use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use amml_api::amml_abi::{
    FramePresent, FramePresented, RenderDeviceReset, ScreenshotTaken, ScreenshotWritten, SurfaceLost,
};
use amml_api::{
    CaptureRequest, CaptureStream, ComRef, DrawSurface, FramePresentExt, Listener, Logger, Point, ShotLayers, Size,
    SurfaceLostExt, Window,
};
use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Direct3D11::{D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT, ID3D11DeviceContext1};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT, DXGI_SAMPLE_DESC};
use windows::core::Interface;

use crate::{FrameCost, OverlayPhase, OverlayRun, OverlayTools, Progress, Step, WindowChecks};

/// The overlay checks of the API test mod. From the enable, a `FramePresent` listener draws one 64 x 64
/// magenta square with `ClearView` in the `NORMAL` band. In the joined world the client ticks then take
/// a `GameOnly` and a `WithOverlays` screenshot, read a capture stream (taken, then left untaken), request
/// a client size and give it back. After the in-game task's window steps they measure the frame times
/// with the drawing listener and without any `FramePresent` listener.
///
/// The render-thread listeners use atomics only, so a client tick that removes the drawing listener
/// never waits on them.
pub struct OverlayChecks {
    progress: &'static Progress,
    windows: &'static WindowChecks,
    run: Mutex<OverlayRun>,
    presented: AtomicU64,
    last_present: Mutex<Option<Instant>>,
    // The cost window the presents go to: 0 none, else an index into `costs` plus one.
    window: AtomicU8,
    costs: [FrameCost; 4],
    bound: AtomicBool,
    bound_width: AtomicU32,
    bound_height: AtomicU32,
    epoch: AtomicU64,
    drawn: AtomicU64,
    draw_failed: AtomicBool,
    copied: AtomicBool,
    shot_ids: [AtomicU64; 2],
    // One bit per screenshot (bit 0 `GameOnly`, bit 1 `WithOverlays`) whose `ScreenshotWritten` came.
    shots_written: AtomicU8,
}

impl OverlayChecks {
    /// The top left corner and the side of the square, in back buffer pixels.
    pub const SQUARE_X: u32 = 64;
    pub const SQUARE_Y: u32 = 64;
    pub const SQUARE: u32 = 64;
    pub const MAGENTA: [f32; 4] = [1.0, 0.0, 1.0, 1.0];
    /// Client ticks after `WorldJoined` before the first check.
    pub const SETTLE_TICKS: u32 = 100;
    /// Presents in each frame-time window.
    pub const COST_FRAMES: u64 = 300;
    /// The `FramePresented` count the mod logs.
    pub const PRESENTED: u64 = 600;
    /// The drawn frames before the one `copy_back_buffer` check.
    pub const COPY_AFTER: u64 = 30;
    /// Capture frames the stream check takes.
    pub const CAPTURE_FRAMES: u32 = 30;
    pub const CAPTURE_RING: u8 = 4;
    pub const CAPTURE_MAX: Size = Size { width: 640, height: 360 };
    /// The client size the checks request.
    pub const REQUEST: Size = Size { width: 1280, height: 720 };
    /// The least time a requested size stays before the checks give it back.
    pub const REQUEST_HOLD: Duration = Duration::from_secs(1);
    /// The time a phase waits for its events before it gives up.
    pub const WAIT: Duration = Duration::from_secs(60);
    /// The time the checks wait for the in-game task's window steps.
    pub const WINDOW_WAIT: Duration = Duration::from_secs(240);

    // The cost windows, as stored in `window`.
    const BARE: u8 = 1;
    const DRAW: u8 = 2;
    const STREAM: u8 = 3;
    const STREAM_IDLE: u8 = 4;

    pub const fn new(progress: &'static Progress, windows: &'static WindowChecks) -> Self {
        Self {
            progress,
            windows,
            run: Mutex::new(OverlayRun::new()),
            presented: AtomicU64::new(0),
            last_present: Mutex::new(None),
            window: AtomicU8::new(0),
            costs: [FrameCost::new(), FrameCost::new(), FrameCost::new(), FrameCost::new()],
            bound: AtomicBool::new(false),
            bound_width: AtomicU32::new(0),
            bound_height: AtomicU32::new(0),
            epoch: AtomicU64::new(0),
            drawn: AtomicU64::new(0),
            draw_failed: AtomicBool::new(false),
            copied: AtomicBool::new(false),
            shot_ids: [AtomicU64::new(0), AtomicU64::new(0)],
            shots_written: AtomicU8::new(0),
        }
    }

    /// Keeps the drawing listener the enable made.
    pub fn set_draw(&self, listener: Option<Listener>) {
        self.lock_run().draw = listener;
    }

    /// Ends the run: gives back its state (the drawing listener and the stream), which the caller drops
    /// outside any lock it holds.
    pub fn take(&self) -> OverlayRun {
        let run = std::mem::take(&mut *self.lock_run());
        for count in [&self.presented, &self.epoch, &self.drawn, &self.shot_ids[0], &self.shot_ids[1]] {
            count.store(0, Ordering::Release);
        }
        for flag in [&self.bound, &self.draw_failed, &self.copied] {
            flag.store(false, Ordering::Release);
        }
        self.window.store(0, Ordering::Release);
        self.shots_written.store(0, Ordering::Release);
        for cost in &self.costs {
            cost.reset();
        }
        *self.last_present.lock().unwrap_or_else(PoisonError::into_inner) = None;
        run
    }

    /// `WorldJoined`: the first join starts the checks.
    pub fn joined(&self) {
        let mut run = self.lock_run();
        if run.phase == OverlayPhase::Idle {
            run.advance();
        }
    }

    /// `FramePresent`, on the render thread: logs the first surface, draws the square and, once, copies
    /// the back buffer into a texture of its own.
    pub fn frame_present(&self, log: Logger, event: &FramePresent) {
        let Some(surface) = event.surface() else {
            return;
        };
        let size = surface.size();
        let epoch = surface.device_epoch();
        if !self.bound.swap(true, Ordering::AcqRel) {
            self.bound_width.store(size.width, Ordering::Release);
            self.bound_height.store(size.height, Ordering::Release);
            self.epoch.store(epoch, Ordering::Release);
            log.info(
                "overlay bound",
                &[
                    ("size", &Self::size_text(size)),
                    ("wrapped", if surface.wrapped() { "1" } else { "0" }),
                    ("format", &surface.format().to_string()),
                    ("device_epoch", &epoch.to_string()),
                ],
            );
            let (x, y) = Self::square_centre();
            log.info("square", &[("x", &x.to_string()), ("y", &y.to_string()), ("side", &Self::SQUARE.to_string())]);
            self.progress.set(Step::OverlayBound);
        } else if self.epoch.swap(epoch, Ordering::AcqRel) != epoch {
            log.info("draw surface", &[("device_epoch", &epoch.to_string()), ("size", &Self::size_text(size))]);
        }
        if size.width < Self::SQUARE_X + Self::SQUARE || size.height < Self::SQUARE_Y + Self::SQUARE {
            return;
        }
        self.draw_square(log, &surface);
        let drawn = self.drawn.fetch_add(1, Ordering::AcqRel).saturating_add(1);
        if drawn >= Self::COPY_AFTER && !self.copied.swap(true, Ordering::AcqRel) {
            Self::copy_check(log, &surface);
        }
    }

    // Clears the square on the back buffer through `ID3D11DeviceContext1::ClearView`, which sets no
    // pipeline state.
    fn draw_square(&self, log: Logger, surface: &DrawSurface<'_>) {
        let rect = RECT {
            left: Self::pixel(Self::SQUARE_X),
            top: Self::pixel(Self::SQUARE_Y),
            right: Self::pixel(Self::SQUARE_X + Self::SQUARE),
            bottom: Self::pixel(Self::SQUARE_Y + Self::SQUARE),
        };
        let drawn = surface
            .context()
            .with(|context| {
                let context = context.cast::<ID3D11DeviceContext1>().ok()?;
                surface.target().with(|target| {
                    // SAFETY: the view and the context belong to the surface's device and are live for this
                    // `FramePresent` call, on its thread.
                    #[allow(unsafe_code)]
                    unsafe {
                        context.ClearView(target, &Self::MAGENTA, Some(&[rect]));
                    }
                })
            })
            .flatten();
        if drawn.is_none() && !self.draw_failed.swap(true, Ordering::AcqRel) {
            log.warn("square not drawn", &[("reason", "no ID3D11DeviceContext1")]);
        }
    }

    // Makes a texture like the back buffer on the surface's device and copies the back buffer into it.
    fn copy_check(log: Logger, surface: &DrawSurface<'_>) {
        let size = surface.size();
        let desc = D3D11_TEXTURE2D_DESC {
            Width: size.width,
            Height: size.height,
            MipLevels: 1,
            ArraySize: 1,
            Format: DXGI_FORMAT(i32::try_from(surface.format()).unwrap_or(0)),
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: 0,
            CPUAccessFlags: 0,
            MiscFlags: 0,
        };
        let texture = surface
            .device()
            .with(|device| {
                let mut texture = None;
                // SAFETY: `desc` and the out pointer are live locals; the device is live for this call.
                #[allow(unsafe_code)]
                let made = unsafe { device.CreateTexture2D(&desc, None, Some(&mut texture)) };
                made.ok().and(texture)
            })
            .flatten();
        let Some(texture) = texture else {
            log.info("copy back buffer", &[("ok", "false"), ("error", "no texture")]);
            return;
        };
        match surface.copy_back_buffer(ComRef::from_interface(&texture)) {
            Ok(()) => log.info("copy back buffer", &[("ok", "true"), ("size", &Self::size_text(size))]),
            Err(error) => log.info("copy back buffer", &[("ok", "false"), ("error", &error.to_string())]),
        }
    }

    /// `FramePresented`, on the render thread: counts the presents and adds each frame time to the
    /// current cost window.
    pub fn frame_presented(&self, log: Logger, event: &FramePresented) {
        let now = Instant::now();
        let before = self.last_present.lock().unwrap_or_else(PoisonError::into_inner).replace(now);
        let window = self.window.load(Ordering::Acquire);
        if let Some(before) = before
            && let Some(cost) = window.checked_sub(1).and_then(|index| self.costs.get(usize::from(index)))
        {
            let interval = u64::try_from(now.duration_since(before).as_micros()).unwrap_or(u64::MAX);
            cost.add(interval, u64::from(event.cpu_us));
        }
        let count = self.presented.fetch_add(1, Ordering::AcqRel).saturating_add(1);
        if count == Self::PRESENTED && self.progress.set(Step::FramesPresented) {
            log.info("frames", &[("presented", &count.to_string())]);
        }
    }

    /// `SurfaceLost`, on the render thread. The mod keeps no object of the device, so it only logs.
    pub fn surface_lost(&self, log: Logger, event: &SurfaceLost) {
        log.info(
            "surface lost",
            &[("reason", &format!("{:?}", event.reason())), ("device_epoch", &event.device_epoch.to_string())],
        );
    }

    pub fn render_device_reset(&self, log: Logger, event: &RenderDeviceReset) {
        log.info("render device reset", &[("device_epoch", &event.device_epoch.to_string())]);
    }

    /// `ScreenshotTaken`: logs it, for the order against `ScreenshotWritten`.
    pub fn screenshot_taken(&self, log: Logger, event: &ScreenshotTaken) {
        if let Some((_, layers)) = self.shot_of(event.id) {
            log.info("screenshot taken", &[("layers", Self::layers_text(layers)), ("ok", Self::ok_text(event.ok))]);
        }
    }

    /// `ScreenshotWritten`: marks the screenshot's step when its file was written.
    pub fn screenshot_written(&self, log: Logger, event: &ScreenshotWritten) {
        let Some((index, layers)) = self.shot_of(event.id) else {
            return;
        };
        self.shots_written.fetch_or(1 << index, Ordering::AcqRel);
        log.info("screenshot written", &[("layers", Self::layers_text(layers)), ("ok", Self::ok_text(event.ok))]);
        if event.ok != 0 {
            self.progress.set(if index == 0 { Step::ShotGameOnly } else { Step::ShotWithOverlays });
        }
    }

    fn shot_of(&self, id: u64) -> Option<(usize, ShotLayers)> {
        if id == 0 {
            return None;
        }
        let index = self.shot_ids.iter().position(|shot| shot.load(Ordering::Acquire) == id)?;
        Some((index, Self::LAYERS[index]))
    }

    const LAYERS: [ShotLayers; 2] = [ShotLayers::GameOnly, ShotLayers::WithOverlays];

    /// One client tick: one step of the current phase. A drawing listener the step removed is dropped
    /// after the run lock is released.
    pub fn client_tick(&self, log: Logger, tools: OverlayTools) {
        let removed = {
            let mut run = self.lock_run();
            run.ticks = run.ticks.saturating_add(1);
            match run.phase {
                OverlayPhase::Idle | OverlayPhase::Done => None,
                OverlayPhase::Settle => {
                    self.settle(&mut run, log, tools);
                    None
                }
                OverlayPhase::Shots => {
                    self.shots(&mut run, log, tools);
                    None
                }
                OverlayPhase::Capture => {
                    self.capture(&mut run, log);
                    None
                }
                OverlayPhase::CaptureIdle => {
                    self.capture_idle(&mut run, log);
                    None
                }
                OverlayPhase::RequestSize => {
                    self.request_size(&mut run, log, tools.window);
                    None
                }
                OverlayPhase::Window => {
                    self.window_steps(&mut run, log);
                    None
                }
                OverlayPhase::CostDraw => self.cost_draw(&mut run, log),
                OverlayPhase::CostBare => {
                    self.cost_bare(&mut run, log);
                    None
                }
            }
        };
        drop(removed);
    }

    // Waits for the world to settle and the square to be drawn, reads the window, then asks both
    // screenshots into `<data>/shots`.
    fn settle(&self, run: &mut OverlayRun, log: Logger, tools: OverlayTools) {
        let since = *run.sent_at.get_or_insert_with(Instant::now);
        let drawn = self.drawn.load(Ordering::Acquire) >= Self::COPY_AFTER;
        if run.ticks < Self::SETTLE_TICKS || (!drawn && since.elapsed() < Self::WAIT) {
            return;
        }
        if !drawn {
            log.warn("square not drawn in time", &[("bound", Self::bool_text(self.bound.load(Ordering::Acquire)))]);
        }
        Self::window_reads(log, tools.window);
        let Some(dir) = tools.dirs.data_dir().or_else(|| tools.dirs.mod_dir()).map(|dir| dir.join("shots")) else {
            log.warn("screenshot not asked", &[("error", "no data folder")]);
            run.advance();
            return;
        };
        if let Err(error) = std::fs::create_dir_all(&dir) {
            log.warn("screenshot folder not made", &[("error", &error.to_string())]);
        }
        for (index, layers) in Self::LAYERS.into_iter().enumerate() {
            let path = dir.join(if index == 0 { "game_only.png" } else { "with_overlays.png" });
            let _ = std::fs::remove_file(&path);
            let shown = path.display().to_string();
            match tools.overlay.screenshot(&path, layers) {
                Ok(id) => {
                    self.shot_ids[index].store(id.0, Ordering::Release);
                    log.info(
                        "screenshot asked",
                        &[("layers", Self::layers_text(layers)), ("id", &id.0.to_string()), ("path", &shown)],
                    );
                }
                Err(error) => log.warn(
                    "screenshot refused",
                    &[("layers", Self::layers_text(layers)), ("error", &error.to_string())],
                ),
            }
            run.shots.push(path);
        }
        run.advance();
    }

    // Waits for both `ScreenshotWritten`, then opens the capture stream.
    fn shots(&self, run: &mut OverlayRun, log: Logger, tools: OverlayTools) {
        let since = *run.sent_at.get_or_insert_with(Instant::now);
        let written = self.shots_written.load(Ordering::Acquire) == 0b11;
        if !written && since.elapsed() < Self::WAIT {
            return;
        }
        if !written {
            log.warn("screenshot not written in time", &[]);
        }
        let request = CaptureRequest {
            layers: ShotLayers::WithOverlays,
            every_nth_present: 1,
            ring: Self::CAPTURE_RING,
            max_size: Some(Self::CAPTURE_MAX),
        };
        match tools.overlay.open_capture_stream(request) {
            Ok(stream) => {
                log.info(
                    "capture opened",
                    &[("stream", &stream.id().to_string()), ("ring", &Self::CAPTURE_RING.to_string())],
                );
                run.stream = Some(stream);
                self.start_window(Self::STREAM);
            }
            Err(error) => log.warn("capture refused", &[("error", &error.to_string())]),
        }
        run.advance();
    }

    // Takes every filled frame each tick: the first `CAPTURE_FRAMES` give the `capture` line, and the
    // frame times are measured until `COST_FRAMES` presents.
    fn capture(&self, run: &mut OverlayRun, log: Logger) {
        let Some(mut stream) = run.stream.take() else {
            run.advance();
            return;
        };
        self.take_frames(run, &mut stream, log);
        run.stream = Some(stream);
        let since = *run.sent_at.get_or_insert_with(Instant::now);
        let cost = &self.costs[usize::from(Self::STREAM - 1)];
        let taken = run.frames >= Self::CAPTURE_FRAMES;
        if (taken && cost.frames() >= Self::COST_FRAMES) || since.elapsed() >= Self::WAIT {
            if !taken {
                log.warn("capture frames not taken in time", &[("frames", &run.frames.to_string())]);
            }
            Self::log_cost(log, "draw+stream", cost);
            self.start_window(Self::STREAM_IDLE);
            run.advance();
        }
    }

    fn take_frames(&self, run: &mut OverlayRun, stream: &mut CaptureStream, log: Logger) {
        while let Some(frame) = stream.take() {
            if run.frames >= Self::CAPTURE_FRAMES {
                continue;
            }
            let index = frame.present_index();
            if let Some(last) = run.last_index {
                run.skipped = run.skipped.saturating_add(index.saturating_sub(last.saturating_add(1)));
            }
            run.last_index = Some(index);
            run.frames += 1;
            let size = frame.size();
            run.frame_size = Some(size);
            if run.frame_magenta.is_none() {
                run.frame_magenta = self.square_in(size, frame.stride(), frame.pixels());
            }
            if run.frames == Self::CAPTURE_FRAMES {
                self.progress.set(Step::Capture);
                log.info(
                    "capture",
                    &[
                        ("frames", &run.frames.to_string()),
                        ("skipped", &run.skipped.to_string()),
                        ("size", &Self::size_text(size)),
                        ("square", run.frame_magenta.map_or("none", Self::bool_text)),
                    ],
                );
            }
        }
    }

    // Leaves the stream untaken for `COST_FRAMES` presents: the loader skips them without a wait.
    fn capture_idle(&self, run: &mut OverlayRun, log: Logger) {
        let since = *run.sent_at.get_or_insert_with(Instant::now);
        let cost = &self.costs[usize::from(Self::STREAM_IDLE - 1)];
        if cost.frames() < Self::COST_FRAMES && since.elapsed() < Self::WAIT {
            return;
        }
        Self::log_cost(log, "draw+stream-untaken", cost);
        let free = run.stream.as_ref().map_or(0, CaptureStream::free_slots);
        log.info("capture untaken", &[("free_slots", &free.to_string())]);
        self.start_window(0);
        run.stream = None;
        run.advance();
    }

    // Requests `REQUEST`, waits for its `WindowSize` and `REQUEST_HOLD`, then gives the size back.
    fn request_size(&self, run: &mut OverlayRun, log: Logger, window: Window) {
        if !run.sent {
            let _ = self.windows.take_injected_size();
            match window.request_size(Some(Self::REQUEST)) {
                Ok(()) => {
                    run.sent = true;
                    run.sent_at = Some(Instant::now());
                    log.info("request size sent", &[("size", &Self::size_text(Self::REQUEST))]);
                }
                Err(error) => {
                    log.warn("request size failed", &[("error", &error.to_string())]);
                    self.finish(run, log);
                }
            }
            return;
        }
        let elapsed = run.sent_at.map_or(Duration::ZERO, |at| at.elapsed());
        let seen = self.windows.take_injected_size() || self.progress.seen(Step::RequestSize);
        if seen {
            self.progress.set(Step::RequestSize);
        }
        if (!seen || elapsed < Self::REQUEST_HOLD) && elapsed < Self::WAIT {
            return;
        }
        match window.request_size(None) {
            Ok(()) if seen => {
                log.info("request size ok", &[("size", &Self::size_text(Self::REQUEST)), ("restored", "true")])
            }
            Ok(()) => log.warn("request size not seen", &[("size", &Self::size_text(Self::REQUEST))]),
            Err(error) => log.warn("request size not given back", &[("error", &error.to_string())]),
        }
        self.finish(run, log);
    }

    // The checks the mod runs alone are done; the window events from now on are the in-game task's.
    fn finish(&self, run: &mut OverlayRun, log: Logger) {
        log.info("overlay checks done", &[]);
        self.windows.arm();
        run.phase = OverlayPhase::Window;
        run.ticks = 0;
        run.sent = false;
        run.sent_at = None;
    }

    fn window_steps(&self, run: &mut OverlayRun, log: Logger) {
        let since = *run.sent_at.get_or_insert_with(Instant::now);
        let done = self.windows.task_done();
        if !done && since.elapsed() < Self::WINDOW_WAIT {
            return;
        }
        if !done {
            log.warn("window steps not seen", &[]);
        }
        self.start_window(Self::DRAW);
        run.advance();
    }

    // Measures `COST_FRAMES` presents with the drawing listener, then removes it.
    fn cost_draw(&self, run: &mut OverlayRun, log: Logger) -> Option<Listener> {
        let since = *run.sent_at.get_or_insert_with(Instant::now);
        let cost = &self.costs[usize::from(Self::DRAW - 1)];
        if cost.frames() < Self::COST_FRAMES && since.elapsed() < Self::WAIT {
            return None;
        }
        Self::log_cost(log, "draw", cost);
        self.start_window(Self::BARE);
        run.advance();
        run.draw.take()
    }

    fn cost_bare(&self, run: &mut OverlayRun, log: Logger) {
        let since = *run.sent_at.get_or_insert_with(Instant::now);
        let cost = &self.costs[usize::from(Self::BARE - 1)];
        if cost.frames() < Self::COST_FRAMES && since.elapsed() < Self::WAIT {
            return;
        }
        Self::log_cost(log, "none", cost);
        let draw = &self.costs[usize::from(Self::DRAW - 1)];
        if cost.frames() > 0 && draw.frames() > 0 {
            self.progress.set(Step::FrameCost);
        }
        self.start_window(0);
        run.advance();
    }

    // Starts cost window `window` (0 for none) from no frames.
    fn start_window(&self, window: u8) {
        if let Some(cost) = window.checked_sub(1).and_then(|index| self.costs.get(usize::from(index))) {
            cost.reset();
        }
        self.window.store(window, Ordering::Release);
    }

    fn log_cost(log: Logger, listener: &str, cost: &FrameCost) {
        log.info(
            "frame cost",
            &[
                ("listener", listener),
                ("frames", &cost.frames().to_string()),
                ("interval_us", &cost.mean_interval_us().to_string()),
                ("present_us", &cost.mean_present_us().to_string()),
                // The time outside the game's `Present` call: the game's frame work and the loader's work before the call
                ("outside_present_us", &cost.mean_interval_us().saturating_sub(cost.mean_present_us()).to_string()),
                ("worst_us", &cost.worst_us().to_string()),
            ],
        );
    }

    fn window_reads(log: Logger, window: Window) {
        let size = |size: Option<Size>| size.map_or_else(|| "none".to_string(), Self::size_text);
        let flag = |flag: Option<bool>| flag.map_or("none", Self::bool_text);
        let point =
            |point: Option<Point>| point.map_or_else(|| "none".to_string(), |p| format!("{:.0},{:.0}", p.x, p.y));
        log.info(
            "window reads",
            &[
                ("client", &size(window.client_size())),
                ("buffer", &size(window.buffer_size())),
                ("dpi", &window.dpi().map_or_else(|| "none".to_string(), |dpi| dpi.to_string())),
                ("focus", flag(window.has_focus())),
                ("foreground", flag(window.is_foreground())),
                ("cursor", &point(window.cursor_pos())),
                ("to_buffer_100", &point(window.to_buffer(Point { x: 100.0, y: 100.0 }))),
            ],
        );
    }

    // Whether the square's centre is magenta in a BGRA capture frame of `size`, scaled down from the
    // bound back buffer by a whole factor; `None` when the point is outside the frame.
    fn square_in(&self, size: Size, stride: u32, pixels: &[u8]) -> Option<bool> {
        let width = self.bound_width.load(Ordering::Acquire);
        let factor = width.checked_div(size.width)?.max(1);
        let (x, y) = Self::square_centre();
        let (x, y) = (x / factor, y / factor);
        if x >= size.width || y >= size.height {
            return None;
        }
        let at = usize::try_from(u64::from(y) * u64::from(stride) + u64::from(x) * 4).ok()?;
        let pixel = pixels.get(at..at.checked_add(3)?)?;
        Some(pixel[0] >= 250 && pixel[1] <= 5 && pixel[2] >= 250)
    }

    /// The centre of the square, in back buffer pixels.
    pub fn square_centre() -> (u32, u32) {
        (Self::SQUARE_X + Self::SQUARE / 2, Self::SQUARE_Y + Self::SQUARE / 2)
    }

    fn pixel(value: u32) -> i32 {
        i32::try_from(value).unwrap_or(i32::MAX)
    }

    fn lock_run(&self) -> std::sync::MutexGuard<'_, OverlayRun> {
        self.run.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn size_text(size: Size) -> String {
        format!("{}x{}", size.width, size.height)
    }

    fn layers_text(layers: ShotLayers) -> &'static str {
        match layers {
            ShotLayers::GameOnly => "GameOnly",
            ShotLayers::WithOverlays => "WithOverlays",
        }
    }

    fn ok_text(ok: u8) -> &'static str {
        if ok != 0 { "true" } else { "false" }
    }

    fn bool_text(value: bool) -> &'static str {
        if value { "true" } else { "false" }
    }
}

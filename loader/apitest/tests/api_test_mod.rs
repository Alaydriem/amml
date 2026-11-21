use amml_api::amml_abi::{
    AmmlDirKind, AmmlModInfo, AmmlModifiers, AmmlMouseButton, AmmlMoveIntent, AmmlPoint, AmmlPointerKind, AmmlStatus,
    AmmlStr, ClientTick, FramePresent, FramePresented, FrameUpdate, GamepadPoll, InjectedInputReleased, KeyInput,
    LookDelta, ModsLoaded, MouseInput, PlayerMoveInput, ProviderReady, RenderDeviceReset, ScreenshotTaken,
    ScreenshotWritten, Shutdown, SurfaceLost, SwapChainResized, WindowChar, WindowFocus, WindowPointer, WindowSize,
    WorldJoined, WorldLeft,
};
use amml_api::testing::{FakeClient, FakeGame, FakeWindow};
use amml_api::{AmmlMod, HookOrder, Screen};
use amml_apitest::{ApiTestMod, OverlayChecks, amml_mod_disable, amml_mod_enable, amml_mod_load};

use crate::support::WarpSurface;

fn load_and_enable(fake: &FakeGame) -> AmmlStatus {
    let mut info = std::mem::MaybeUninit::<AmmlModInfo>::zeroed();
    // SAFETY: the table lives in `fake` and `info` is writable for one AmmlModInfo.
    #[allow(unsafe_code)]
    let status = unsafe { amml_mod_load(fake.api_table(), info.as_mut_ptr()) };
    assert_eq!(status, AmmlStatus::OK);
    amml_mod_enable()
}

fn offer_core(fake: &FakeGame) {
    fake.offer::<ModsLoaded>();
    fake.offer::<ProviderReady>();
    fake.offer::<FrameUpdate>();
    fake.offer::<ClientTick>();
}

fn provider_ready(name: &'static str) -> ProviderReady {
    ProviderReady { header: FakeGame::header::<ProviderReady>(), name: AmmlStr::from_str(name), major: 1, minor: 0 }
}

fn ticks(fake: &FakeGame, count: u32) {
    for _ in 0..count {
        fake.raise(&mut ClientTick { header: FakeGame::header::<ClientTick>() });
    }
}

fn offer_input(fake: &FakeGame) {
    fake.offer::<WorldJoined>();
    fake.offer::<WorldLeft>();
    fake.offer::<KeyInput>();
    fake.offer::<MouseInput>();
    fake.offer::<LookDelta>();
    fake.offer::<PlayerMoveInput>();
    fake.offer::<GamepadPoll>();
    fake.offer::<InjectedInputReleased>();
    for area in ApiTestMod::AREAS {
        fake.offer_area(area);
    }
}

fn offer_overlay(fake: &FakeGame) {
    fake.offer::<Shutdown>();
    fake.offer::<WindowPointer>();
    fake.offer::<WindowChar>();
    fake.offer::<WindowSize>();
    fake.offer::<WindowFocus>();
    fake.offer::<FramePresent>();
    fake.offer::<FramePresented>();
    fake.offer::<SwapChainResized>();
    fake.offer::<SurfaceLost>();
    fake.offer::<RenderDeviceReset>();
    fake.offer::<ScreenshotTaken>();
    fake.offer::<ScreenshotWritten>();
    fake.set_window(FakeWindow::default());
}

fn window_size(width: u32, height: u32, minimised: bool, injected: bool) -> WindowSize {
    let mut header = FakeGame::header::<WindowSize>();
    header.injected = u8::from(injected);
    WindowSize { header, minimised: u8::from(minimised), _reserved: [0; 3], width, height }
}

fn resized(width: u32, height: u32) -> SwapChainResized {
    SwapChainResized { header: FakeGame::header::<SwapChainResized>(), ok: 1, _reserved: [0; 3], width, height }
}

fn left_button(down: bool) -> WindowPointer {
    WindowPointer {
        header: FakeGame::header::<WindowPointer>(),
        kind: AmmlPointerKind::BUTTON,
        button: AmmlMouseButton::LEFT,
        down: u8::from(down),
        _reserved: 0,
        modifiers: AmmlModifiers(0),
        _reserved2: 0,
        client: AmmlPoint { x: 20.0, y: 700.0 },
        buffer: AmmlPoint { x: 20.0, y: 700.0 },
        notches: 0.0,
    }
}

// The id of the screenshot of `layers` from the mod's `screenshot asked` line.
fn shot_id(logs: &[String], layers: &str) -> Option<u64> {
    let prefix = format!("info screenshot asked layers={layers} id=");
    let line = logs.iter().find_map(|line| line.strip_prefix(&prefix))?;
    line.split(' ').next()?.parse().ok()
}

/// Stands in for the loader and the in-game task on the overlay side of a session: presents with the
/// WARP surface, the screenshot events, capture frames, the `WindowSize` of a size request, and once the
/// mod's own overlay checks are done the resizes, minimise, restore, click and character of the task.
struct OverlaySide {
    warp: WarpSurface,
    frame: u64,
    shots_answered: bool,
    size_answered: bool,
    task_done: bool,
    click_cancelled: bool,
}

impl OverlaySide {
    fn new() -> Self {
        Self {
            warp: WarpSurface::new(),
            frame: 0,
            shots_answered: false,
            size_answered: false,
            task_done: false,
            click_cancelled: false,
        }
    }

    fn frame(&mut self, fake: &FakeGame) {
        self.frame += 1;
        let surface = self.warp.surface();
        fake.raise(&mut FramePresent {
            header: FakeGame::header::<FramePresent>(),
            frame_id: self.frame,
            sync_interval: 1,
            flags: 0,
            surface,
        });
        fake.raise(&mut FramePresented {
            header: FakeGame::header::<FramePresented>(),
            frame_id: self.frame,
            result: 0,
            cpu_us: 100,
        });
        let side = WarpSurface::SIDE;
        let magenta = [255u8, 0, 255, 255].repeat((side * side) as usize);
        fake.push_capture_frame(side, side, magenta);
        let logs = fake.logs();
        if !self.shots_answered
            && let (Some(game_only), Some(with_overlays)) = (shot_id(&logs, "GameOnly"), shot_id(&logs, "WithOverlays"))
        {
            for id in [game_only, with_overlays] {
                fake.raise(&mut ScreenshotTaken {
                    header: FakeGame::header::<ScreenshotTaken>(),
                    id,
                    ok: 1,
                    _reserved: [0; 7],
                });
                fake.raise(&mut ScreenshotWritten {
                    header: FakeGame::header::<ScreenshotWritten>(),
                    id,
                    ok: 1,
                    _reserved: [0; 7],
                });
            }
            self.shots_answered = true;
        }
        if !self.size_answered && logs.iter().any(|line| line == "info request size sent size=1280x720") {
            fake.raise(&mut window_size(1280, 720, false, true));
            fake.raise(&mut resized(1280, 720));
            self.size_answered = true;
        }
        if !self.task_done && logs.iter().any(|line| line == "info overlay checks done") {
            fake.raise(&mut window_size(1280, 720, false, false));
            fake.raise(&mut resized(1280, 720));
            fake.raise(&mut window_size(1600, 900, false, false));
            fake.raise(&mut resized(1600, 900));
            fake.raise(&mut window_size(0, 0, true, false));
            fake.raise(&mut window_size(1600, 900, false, false));
            fake.raise(&mut resized(1600, 900));
            self.click_cancelled = fake.raise(&mut left_button(true)).cancelled;
            fake.raise(&mut left_button(false));
            fake.raise(&mut WindowChar { header: FakeGame::header::<WindowChar>(), ch: u32::from('a') });
            self.task_done = true;
        }
    }
}

fn in_world() -> FakeClient {
    FakeClient {
        local_player: true,
        world_active: true,
        showing_menu: false,
        top_screen: Screen::Other,
        hide_hud: false,
        mouse_captured: Some(true),
        pause_on_focus_lost: true,
        player_alive: Some(true),
        ..FakeClient::default()
    }
}

fn player_move(forward: f32, injected: bool) -> PlayerMoveInput {
    let mut header = FakeGame::header::<PlayerMoveInput>();
    header.injected = u8::from(injected);
    let intent = AmmlMoveIntent { forward, left: 0.0, jump: 0, sneak: 0, sprint: 0, _reserved: 0 };
    PlayerMoveInput { header, intent }
}

// One game frame of a joined world as the fake can give it: the client tick, the input part of the
// frame, the player's forward walk while W is down in the game, the injected move intent while it is
// set, and a gamepad poll.
fn world_frame(fake: &FakeGame) {
    fake.raise_with_client(&mut ClientTick { header: FakeGame::header::<ClientTick>() });
    fake.frame();
    let injected = fake.injected();
    let last_key = injected.iter().rev().find(|line| line.starts_with("key 0x57"));
    if last_key.is_some_and(|line| line.ends_with("down")) {
        fake.raise(&mut player_move(1.0, false));
    }
    let last_move = injected.iter().rev().find(|line| line.starts_with("move"));
    if last_move.is_some_and(|line| line == "move forward=1") {
        fake.raise(&mut player_move(1.0, true));
    }
    fake.raise(&mut GamepadPoll { header: FakeGame::header::<GamepadPoll>() });
}

// The lines of `logs` that are in `wanted`, in log order.
fn kept(logs: &[String], wanted: &[&str]) -> Vec<String> {
    logs.iter().filter(|line| wanted.contains(&line.as_str())).cloned().collect()
}

#[test]
fn a_full_session_logs_every_step_then_apitest_ok() {
    let fake = FakeGame::new();
    offer_core(&fake);
    offer_input(&fake);
    // Stands in for the loader, which raises a poll under a `FULL` lease already cancelled
    let blocker = fake
        .ctx()
        .events()
        .listen::<GamepadPoll>(HookOrder::FIRST, |event, _| {
            event.header.cancelled = 1;
        })
        .unwrap();
    fake.set_client(FakeClient::default());
    offer_overlay(&fake);
    let data = std::env::temp_dir().join(format!("amml-apitest-{}", std::process::id()));
    fake.set_dir(AmmlDirKind::DATA, data.to_str().unwrap());
    let mut overlay = OverlaySide::new();
    assert_eq!(load_and_enable(&fake), AmmlStatus::OK);

    fake.raise(&mut ModsLoaded { header: FakeGame::header::<ModsLoaded>() });
    fake.raise(&mut provider_ready("apitest.echo"));
    fake.open_gate();
    fake.raise(&mut FrameUpdate { header: FakeGame::header::<FrameUpdate>() });
    fake.raise_with_client(&mut ClientTick { header: FakeGame::header::<ClientTick>() });
    fake.set_epoch(1);
    fake.set_client(in_world());
    fake.raise_with_client(&mut WorldJoined { header: FakeGame::header::<WorldJoined>(), epoch: 1 });
    fake.bind_input();
    // The turn and the size request are paced in wall time, so the frames are too
    for _ in 0..6000 {
        world_frame(&fake);
        overlay.frame(&fake);
        if fake.logs().iter().any(|line| line.starts_with("info apitest ok")) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    drop(blocker);
    fake.raise_with_client(&mut Shutdown { header: FakeGame::header::<Shutdown>() });
    let _ = std::fs::remove_dir_all(&data);

    let expected = [
        "info listening",
        "info mods loaded",
        "info provider ready name=apitest.echo",
        "info frame gate open",
        "info client reads top_screen=Start",
        "info world joined epoch=1",
        "info client reads local_player=yes world_active=true showing_menu=false top_screen=Other",
        "info hud hidden=true",
        "info hud hidden=false",
        "info mouse captured=false",
        "info mouse captured=true",
        "info key input seen key=0x57 injected=true",
        "info player move forward=1",
        "info key held key=0x57 frames=40",
        "info move intent seen forward=1 injected=true",
        "info turn sent steps=10 yaw=90",
        "info turn seen yaw=90 pitch=0 injected=true",
        "info wheel seen notches=1 injected=true",
        "info click seen button=Left injected=true",
        "info click seen button=X1 injected=true",
        "info click seen button=X2 injected=true",
        "info key held across close key=0x57",
        "info pause opened top_screen=Pause",
    ];
    let logs = fake.logs();
    assert_eq!(kept(&logs, &expected), expected.map(String::from).to_vec(), "{logs:#?}");
    let ok = logs.iter().position(|line| line.starts_with("info apitest ok")).expect("apitest ok");
    let pause = logs.iter().position(|line| line == "info pause opened top_screen=Pause").unwrap();
    assert!(ok > pause, "{logs:#?}");
    assert!(logs.iter().any(|line| line.starts_with("info gamepad polls skipped count=")), "{logs:#?}");
    assert!(!logs.iter().any(|line| line.starts_with("warn")), "{logs:#?}");
    let writes =
        ["hide_hud=true", "hide_hud=false", "mouse_captured=false", "mouse_captured=true", "open_pause_screen"];
    assert_eq!(fake.client_writes(), writes.map(String::from).to_vec());
    let injected = fake.injected();
    for line in ["key 0x57 down", "key 0x57 up", "move forward=1", "move none", "wheel 1"] {
        assert!(injected.contains(&line.to_string()), "{line}: {injected:?}");
    }
    // The 90 degrees reach the game as ten 9 degree turns on ten frames, not as one snap
    let turns: Vec<&String> = injected.iter().filter(|line| line.starts_with("turn ")).collect();
    assert_eq!(turns, vec!["turn 9 0"; 10], "{injected:?}");
    let spread = logs.iter().find_map(|line| line.strip_prefix("info turn spread ms=")).expect("turn spread");
    let spread: u64 = spread.parse().unwrap();
    assert!((900..2000).contains(&spread), "{spread}");
    let held = logs.iter().find_map(|line| line.strip_prefix("info turn held ms=")).expect("turn held");
    let held: u64 = held.parse().unwrap();
    assert!(held >= 2000, "{held}");
    assert!(injected.contains(&"button left down".to_string()), "{injected:?}");
    assert!(injected.contains(&"button left up".to_string()), "{injected:?}");
    for line in ["button x1 down", "button x1 up", "button x2 down", "button x2 up"] {
        assert!(injected.contains(&line.to_string()), "{line}: {injected:?}");
    }

    let overlay_lines = [
        "info overlay bound size=256x256 wrapped=0 format=28 device_epoch=1",
        "info square x=96 y=96 side=64",
        "info screenshot taken layers=GameOnly ok=true",
        "info screenshot written layers=GameOnly ok=true",
        "info screenshot taken layers=WithOverlays ok=true",
        "info screenshot written layers=WithOverlays ok=true",
        "info capture frames=30 skipped=0 size=256x256 square=true",
        "info request size ok size=1280x720 restored=true",
        "info overlay checks done",
        "info window size seen size=1280x720",
        "info window restored size=1600x900",
        "info resized ok=true size=1600x900 after=restore",
        "info window pointer button=Left down=true client=20,700 buffer=20,700",
        "info window pointer cancelled button=Left",
        "info window char ch=a",
    ];
    assert_eq!(kept(&logs, &overlay_lines), overlay_lines.map(String::from).to_vec(), "{logs:#?}");
    for prefix in [
        "info frames presented=600",
        "info copy back buffer ok=false",
        "info window reads client=1280x720 buffer=1280x720 dpi=96 focus=true",
        "info frame cost listener=draw+stream frames=",
        "info frame cost listener=draw+stream-untaken frames=",
        "info frame cost listener=draw frames=",
        "info frame cost listener=none frames=",
    ] {
        assert!(logs.iter().any(|line| line.starts_with(prefix)), "{prefix}: {logs:#?}");
    }
    let char_seen = logs.iter().position(|line| line == "info window char ch=a").unwrap();
    let bare = logs.iter().position(|line| line.starts_with("info frame cost listener=none")).unwrap();
    assert!(char_seen < bare && bare < ok, "{logs:#?}");
    assert_eq!(logs.last().map(String::as_str), Some("info shutdown game_alive=true client=yes"), "{logs:#?}");
    assert!(overlay.click_cancelled, "the posted click is cancelled once the overlay checks are done");
    assert_eq!(fake.window_requests(), ["request_size 1280x720", "request_size none"].map(String::from).to_vec());
    assert_eq!(fake.screenshots().len(), 2);
    assert_eq!(fake.open_capture_streams(), 0, "the stream is closed after its checks");
    let (x, y) = OverlayChecks::square_centre();
    assert_eq!(overlay.warp.pixel(x, y), [255, 0, 255, 255], "the square is drawn on the back buffer");
    assert_eq!(overlay.warp.pixel(8, 8), [0, 0, 0, 0], "only the square is drawn");
}

#[test]
fn a_loader_without_the_input_areas_says_which_are_missing() {
    let fake = FakeGame::new();
    offer_core(&fake);

    assert_eq!(load_and_enable(&fake), AmmlStatus::OK);

    let logs = fake.logs();
    let areas: Vec<&String> = logs.iter().filter(|line| line.starts_with("warn area not available")).collect();
    assert_eq!(areas.len(), ApiTestMod::AREAS.len() - 1, "{logs:#?}");
    assert!(!logs.contains(&"warn area not available area=input.lease".to_string()), "{logs:#?}");
    assert!(logs.contains(&"warn event not available event=input.key".to_string()), "{logs:#?}");
}

#[test]
fn without_an_input_feed_nothing_is_sent_and_the_mouse_steps_are_waived() {
    let fake = FakeGame::new();
    offer_core(&fake);
    offer_input(&fake);
    fake.set_client(in_world());
    assert_eq!(load_and_enable(&fake), AmmlStatus::OK);
    fake.open_gate();
    fake.set_epoch(1);
    fake.raise_with_client(&mut WorldJoined { header: FakeGame::header::<WorldJoined>(), epoch: 1 });

    for _ in 0..12_000 {
        world_frame(&fake);
    }

    let logs = fake.logs();
    assert!(!logs.iter().any(|line| line.starts_with("info key held")), "{logs:#?}");
    assert!(logs.contains(&"warn mouse injection not ready waived=wheel reason=no real mouse feed".to_string()));
    assert!(logs.contains(&"warn mouse injection not ready waived=click reason=no real mouse feed".to_string()));
    assert!(!logs.iter().any(|line| line.starts_with("info apitest ok")), "{logs:#?}");
    assert!(fake.injected().is_empty());
}

#[test]
fn without_its_provider_ready_the_mod_waits_and_does_not_pass() {
    let fake = FakeGame::new();
    offer_core(&fake);
    fake.offer::<WorldJoined>();
    fake.offer::<WorldLeft>();
    assert_eq!(load_and_enable(&fake), AmmlStatus::OK);

    fake.raise(&mut ModsLoaded { header: FakeGame::header::<ModsLoaded>() });
    fake.open_gate();
    fake.raise(&mut FrameUpdate { header: FakeGame::header::<FrameUpdate>() });
    ticks(&fake, 600);

    let logs = fake.logs();
    let waiting = logs.iter().find(|line| line.starts_with("info apitest waiting")).expect("apitest waiting");
    assert!(waiting.contains("provider_ready"), "{logs:?}");
    assert!(!logs.contains(&"info apitest ok".to_string()), "{logs:?}");
}

#[test]
fn a_build_without_the_world_events_still_enables_and_says_so() {
    let fake = FakeGame::new();
    offer_core(&fake);

    assert_eq!(load_and_enable(&fake), AmmlStatus::OK);

    let logs = fake.logs();
    assert!(logs.contains(&"warn event not available event=world.joined".to_string()), "{logs:?}");
    assert!(logs.contains(&"warn event not available event=world.left".to_string()), "{logs:?}");
    assert!(fake.missing_required().is_empty());
}

#[test]
fn a_build_without_client_ticks_refuses_the_enable() {
    let fake = FakeGame::new();
    fake.offer::<ModsLoaded>();
    fake.offer::<ProviderReady>();
    fake.offer::<FrameUpdate>();

    assert_eq!(load_and_enable(&fake), AmmlStatus::NOT_AVAILABLE);
    assert_eq!(fake.missing_required(), vec!["client.tick".to_string()]);
}

#[test]
fn disable_removes_the_listeners() {
    let fake = FakeGame::new();
    offer_core(&fake);
    assert_eq!(load_and_enable(&fake), AmmlStatus::OK);
    fake.open_gate();

    assert_eq!(amml_mod_disable(), AmmlStatus::OK);
    fake.raise(&mut FrameUpdate { header: FakeGame::header::<FrameUpdate>() });

    let logs = fake.logs();
    assert!(logs.contains(&"info disabled game_alive=true".to_string()), "{logs:?}");
    assert!(!logs.contains(&"info frame gate open".to_string()), "{logs:?}");
}

// The `uses` block of `amml.hcl`: from its `uses {` line to the end of the file, with LF line ends (a
// checkout may give the file CRLF).
fn manifest_uses_block() -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/amml.hcl");
    let text = std::fs::read_to_string(path).unwrap().replace("\r\n", "\n");
    let start = text.find("\nuses {\n").unwrap() + 1;
    text[start..].to_string()
}

#[test]
fn the_manifest_uses_block_is_the_uses_of_the_mod() {
    assert_eq!(manifest_uses_block(), ApiTestMod::USES.to_hcl());
}

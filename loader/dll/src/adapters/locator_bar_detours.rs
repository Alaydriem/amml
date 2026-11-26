use std::ffi::c_void;
use std::sync::atomic::AtomicPtr;

use crate::{Boundary, LocatorBarAdapter, LocatorIconCall, ScreenDetours, SessionHook, TexturePump, UiFlags};

/// `void HudLocatorBarRenderer::render(MinecraftUIRenderContext&, IClientInstance&, UIControl&, int)`, a virtual
/// slot: rcx this, rdx the render context, r8 the client, r9 the owning control, the pass on the stack.
type RenderFn = unsafe extern "C" fn(usize, usize, usize, usize, i32);

/// The icon blit the render calls once for each waypoint: rcx this, rdx the render context, r8 the `TexturePtr`, then
/// x (in xmm3), y, `u0`, `u1`, `v0`, `v1`, width, height, light and alpha as floats, the two packed colours as 32-bit
/// values and the two scales as floats. The arguments after the fourth are on the stack.
type DrawIconFn =
    unsafe extern "C" fn(usize, usize, usize, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, u32, u32, f32, f32);

static RENDER_ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static DRAW_ICON_ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

/// The loader's detours on the draw of the locator bar and on its icon blit.
///
/// The render detour does the work of one draw point: it runs the texture pump when `UiFlags` says the pump has work,
/// and while a mod listens to `LocatorBarIconDraw` it tags the waypoint colours before the original runs and puts them
/// back after it. With neither, it does one relaxed load of `UiFlags` and calls the original. The icon detour does one
/// relaxed load, and calls the original with the registers it received, unless the colours are tagged.
pub struct LocatorBarDetours;

impl LocatorBarDetours {
    pub const RENDER: &'static str = "HudLocatorBarRenderer.render";
    pub const DRAW_ICON: &'static str = "HudLocatorBarRenderer.drawIcon";
    pub const VTABLE: &'static str = "HudLocatorBarRenderer.vtable";

    pub fn render_hook() -> SessionHook {
        SessionHook {
            entry: Self::RENDER,
            vtable: Some(Self::VTABLE),
            detour: Self::render as RenderFn as *const c_void,
            original: &RENDER_ORIGINAL,
        }
    }

    pub fn draw_icon_hook() -> SessionHook {
        SessionHook {
            entry: Self::DRAW_ICON,
            vtable: None,
            detour: Self::draw_icon as DrawIconFn as *const c_void,
            original: &DRAW_ICON_ORIGINAL,
        }
    }

    /// Sets whether a listener of `LocatorBarIconDraw` exists.
    pub(crate) fn set_wanted(wanted: bool) {
        UiFlags::put(UiFlags::LOCATOR, wanted);
    }

    unsafe extern "C" fn render(this: usize, context: usize, client: usize, owner: usize, pass: i32) {
        let flags = UiFlags::load();
        if flags & (UiFlags::DUE | UiFlags::WAITING) != 0 {
            Boundary::run((), || TexturePump::run_uploads(context));
        }
        if flags & UiFlags::LOCATOR != 0 {
            Boundary::run((), || LocatorBarAdapter::with((), |adapter| adapter.tag(client)));
        }
        // SAFETY: the hook on `HudLocatorBarRenderer.render` is installed, so its slot holds a `RenderFn`.
        let original = unsafe { ScreenDetours::original::<RenderFn>(&RENDER_ORIGINAL) };
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, context, client, owner, pass) };
        // A tag exists only when the listener bit was set at the start. The tagged bit, not the listener bit, says
        // whether to restore: a listener that went away during the draw must not leave a tag. With no listener at the
        // start this is the one load of the idle path.
        if flags & UiFlags::LOCATOR != 0 && UiFlags::load() & UiFlags::LOCATOR_TAGGED != 0 {
            Boundary::run((), || LocatorBarAdapter::with((), LocatorBarAdapter::restore));
        }
    }

    #[allow(clippy::too_many_arguments)]
    unsafe extern "C" fn draw_icon(
        this: usize,
        context: usize,
        texture: usize,
        x: f32,
        y: f32,
        u0: f32,
        u1: f32,
        v0: f32,
        v1: f32,
        width: f32,
        height: f32,
        light: f32,
        alpha: f32,
        color: u32,
        second_color: u32,
        scale_x: f32,
        scale_y: f32,
    ) {
        // SAFETY: the hook on `HudLocatorBarRenderer.drawIcon` is installed, so its slot holds a `DrawIconFn`.
        let original = unsafe { ScreenDetours::original::<DrawIconFn>(&DRAW_ICON_ORIGINAL) };
        let mut call = LocatorIconCall {
            texture,
            rect: [x, y, width, height],
            uv: [u0, u1, v0, v1],
            light,
            alpha,
            color,
            second_color,
            scale: [scale_x, scale_y],
        };
        if UiFlags::load() & UiFlags::LOCATOR_TAGGED != 0
            && let Some(edited) = Boundary::run(None, || LocatorBarAdapter::with(None, |adapter| adapter.icon(call)))
        {
            call = edited;
        }
        // SAFETY: the game's own arguments, or the ones the adapter checked: a `TexturePtr` the texture book holds.
        unsafe {
            original(
                this,
                context,
                call.texture,
                call.rect[0],
                call.rect[1],
                call.uv[0],
                call.uv[1],
                call.uv[2],
                call.uv[3],
                call.rect[2],
                call.rect[3],
                call.light,
                call.alpha,
                call.color,
                call.second_color,
                call.scale[0],
                call.scale[1],
            )
        };
    }
}

use std::ffi::c_void;

use amml_abi::{AmmlRect, AmmlScreenKind};
use amml_msvc::MsvcString;
use amml_runtime::{PanelNames, ScreenKindTable};

use crate::{LoaderState, ObjectRead};

/// Reads the visual tree of a `ScreenView`: the name of the root control, and the rectangle of a named panel.
///
/// The tree is the one the game built for the view: `ScreenView.mVisualTree` holds it, it holds the root control,
/// and every control holds its name, its children (a vector of `shared_ptr<UIControl>`), its cached position and
/// size and a visibility bit. Every read is made on the thread that draws the view, inside its `render` call.
pub struct UiTreeWalk {
    visual_tree: usize,
    root_control: usize,
    name: usize,
    children: usize,
    cached_position: usize,
    dirty: usize,
    dirty_mask: u8,
    visible: usize,
    visible_mask: u8,
    size: usize,
    set_cached_position: usize,
}

impl UiTreeWalk {
    // The walk stops here: a screen tree has a few thousand controls.
    const MAX_NODES: usize = 20_000;
    // The most children one control is read with.
    const MAX_CHILDREN: usize = 4096;
    // `shared_ptr<UIControl>`: the control pointer, then the control block.
    const CHILD_STRIDE: usize = 16;

    /// The walk over the resolved entries; `None` when one is missing.
    pub fn new(state: &LoaderState) -> Option<Self> {
        let value = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        let mask = |name: &str| value(name).and_then(|value| u8::try_from(value).ok());
        Some(Self {
            visual_tree: value("ScreenView.mVisualTree")?,
            root_control: value("VisualTree.mRootControl")?,
            name: value("UIControl.mName")?,
            children: value("UIControl.mChildren")?,
            cached_position: value("UIControl.mCachedPosition")?,
            dirty: value("UIControl.mCachedPositionDirty")?,
            dirty_mask: mask("UIControl.mCachedPositionDirtyMask")?,
            visible: value("UIControl.mIsVisibleInTree")?,
            visible_mask: mask("UIControl.mIsVisibleInTreeMask")?,
            size: value("UIControl.mSize")?,
            set_cached_position: value("UIControl._setCachedPosition").filter(|address| *address != 0)?,
        })
    }

    /// The kind of the screen of `view`, from the name of its root control; `None` when the tree cannot be read.
    ///
    /// # Safety
    /// `view` is a live `ScreenView` inside its `render` call, on the thread that runs it.
    pub(crate) unsafe fn kind(&self, view: usize) -> Option<AmmlScreenKind> {
        // SAFETY: the caller guarantees the view; its control tree is live with it.
        let root = unsafe { self.root(view) }?;
        // SAFETY: `root` is the root control of the live tree.
        let name = unsafe { self.name_of(root) }?;
        std::str::from_utf8(name).ok().map(ScreenKindTable::kind_of)
    }

    /// The name of the root control of `view`, which is the screen name `UIScene::getRawScreenName` gives; `None`
    /// when the tree cannot be read or the name is not UTF-8.
    ///
    /// # Safety
    /// As for `kind`.
    pub(crate) unsafe fn root_name(&self, view: usize) -> Option<String> {
        // SAFETY: the caller guarantees the view; its control tree is live with it.
        let root = unsafe { self.root(view) }?;
        // SAFETY: `root` is the root control of the live tree.
        let name = unsafe { self.name_of(root) }?;
        std::str::from_utf8(name).ok().map(str::to_string)
    }

    /// The rectangle of the panel that `path` names under the first `PanelNames::CONTENT` control, in GUI units;
    /// `None` when a control is missing or the panel is hidden.
    ///
    /// # Safety
    /// As for `kind`.
    pub(crate) unsafe fn panel(&self, view: usize, path: &[&str]) -> Option<AmmlRect> {
        // SAFETY: the caller guarantees the view; its control tree is live with it.
        let root = unsafe { self.root(view) }?;
        // SAFETY: `root` is the root control of the live tree.
        let mut control = unsafe { self.find(root, PanelNames::CONTENT) }?;
        for name in path {
            // SAFETY: `control` is a control of the live tree.
            control = unsafe { self.child(control, name) }?;
        }
        // SAFETY: as above.
        if !unsafe { self.is_visible(control) } {
            return None;
        }
        // SAFETY: as above.
        unsafe { self.rect(control) }
    }

    // The root control of the view's tree.
    //
    // Safety: `view` is a live `ScreenView`.
    unsafe fn root(&self, view: usize) -> Option<usize> {
        // SAFETY: a live view holds its visual tree pointer at the entry's offset, and the tree its root control.
        let tree = unsafe { ObjectRead::pointer(view, self.visual_tree) }?;
        // SAFETY: as above.
        unsafe { ObjectRead::pointer(tree, self.root_control) }
    }

    // The bytes of the name of `control`, without a copy: this runs for every control of a walk.
    //
    // Safety: `control` is a live control.
    unsafe fn name_of<'a>(&self, control: usize) -> Option<&'a [u8]> {
        let address = control.checked_add(self.name)?;
        if !address.is_multiple_of(std::mem::align_of::<MsvcString>()) {
            return None;
        }
        // SAFETY: a live control holds an MSVC `std::string` at its `mName` offset; the address is aligned.
        let name = unsafe { std::ptr::with_exposed_provenance::<MsvcString>(address).as_ref() }?;
        Some(name.as_bytes())
    }

    // The children of `control`; none when the vector does not look like one.
    //
    // Safety: `control` is a live control.
    unsafe fn children_of(&self, control: usize) -> Vec<usize> {
        // SAFETY: a live control holds the begin and end pointers of its child vector at the entry's offset.
        let begin = unsafe { ObjectRead::value::<usize>(control, self.children) };
        // SAFETY: as above.
        let end = unsafe { ObjectRead::value::<usize>(control, self.children.saturating_add(8)) };
        let (Some(begin), Some(end)) = (begin, end) else { return Vec::new() };
        let bytes = end.wrapping_sub(begin);
        if begin == 0
            || end < begin
            || !bytes.is_multiple_of(Self::CHILD_STRIDE)
            || bytes / Self::CHILD_STRIDE > Self::MAX_CHILDREN
        {
            return Vec::new();
        }
        (0..bytes / Self::CHILD_STRIDE)
            // SAFETY: the element is inside the live vector; its first word is the control pointer.
            .filter_map(|index| unsafe { ObjectRead::pointer(begin + index * Self::CHILD_STRIDE, 0) })
            .collect()
    }

    // Depth first, the first control named `wanted`.
    //
    // Safety: `root` is a live control.
    unsafe fn find(&self, root: usize, wanted: &str) -> Option<usize> {
        let mut stack = vec![root];
        let mut seen = 0usize;
        while let Some(control) = stack.pop() {
            seen += 1;
            if seen > Self::MAX_NODES {
                return None;
            }
            // SAFETY: every control on the stack is the root or a child of a live control.
            if unsafe { self.name_of(control) } == Some(wanted.as_bytes()) {
                return Some(control);
            }
            // SAFETY: as above.
            let mut children = unsafe { self.children_of(control) };
            children.reverse();
            stack.extend(children);
        }
        None
    }

    // The first child of `control` named `wanted`.
    //
    // Safety: `control` is a live control.
    unsafe fn child(&self, control: usize, wanted: &str) -> Option<usize> {
        // SAFETY: the children of a live control are live.
        unsafe { self.children_of(control) }
            .into_iter()
            // SAFETY: as above.
            .find(|child| unsafe { self.name_of(*child) } == Some(wanted.as_bytes()))
    }

    // Safety: `control` is a live control.
    unsafe fn is_visible(&self, control: usize) -> bool {
        // SAFETY: a live control holds its flag byte at the entry's offset.
        let bits = unsafe { ObjectRead::value::<u8>(control, self.visible) };
        bits.is_some_and(|bits| bits & self.visible_mask != 0)
    }

    // The absolute rectangle of `control`. The game's own area lookup first calls `_setCachedPosition` when the
    // dirty bit is set, and so does this.
    //
    // Safety: `control` is a live control, on the thread of its render call.
    unsafe fn rect(&self, control: usize) -> Option<AmmlRect> {
        // SAFETY: a live control holds its flag byte at the entry's offset.
        let bits = unsafe { ObjectRead::value::<u8>(control, self.dirty) }?;
        if bits & self.dirty_mask != 0 {
            // SAFETY: the entry is the address of `void UIControl::_setCachedPosition() const`: `this` in rcx.
            let set = unsafe {
                std::mem::transmute::<*const c_void, unsafe extern "C" fn(*mut c_void)>(
                    std::ptr::with_exposed_provenance(self.set_cached_position),
                )
            };
            // SAFETY: the control is live, on the render thread inside the view's render call.
            unsafe { set(std::ptr::with_exposed_provenance_mut(control)) };
        }
        // SAFETY: a live control holds its cached position and its size (two f32 each) at the entries' offsets.
        let position = unsafe { ObjectRead::value::<[f32; 2]>(control, self.cached_position) }?;
        // SAFETY: as above.
        let size = unsafe { ObjectRead::value::<[f32; 2]>(control, self.size) }?;
        let finite = position.iter().chain(size.iter()).all(|value| value.is_finite());
        finite.then_some(AmmlRect { x: position[0], y: position[1], width: size[0], height: size[1] })
    }
}

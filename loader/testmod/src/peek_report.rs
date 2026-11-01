/// The text of the peek probe's log lines.
pub struct PeekReport;

impl PeekReport {
    /// The line for a slot hover: the controller, the collection, the index and the stack the visual item call
    /// returned (`empty` for an empty slot, `n/a` when the call gave nothing).
    pub fn hover_line(
        controller: usize,
        collection: &str,
        slot: i32,
        type_name: Option<&str>,
        count: Option<u8>,
    ) -> String {
        let count_text = count.map_or_else(|| "n/a".to_string(), |count| count.to_string());
        format!(
            "probe: hover controller={controller:#x} collection={collection} index={slot} type={} count={count_text}",
            type_name.unwrap_or("n/a")
        )
    }

    /// The line for a slot unhover.
    pub fn unhover_line(controller: usize, collection: &str, slot: i32, tracked: bool) -> String {
        format!("probe: unhover controller={controller:#x} collection={collection} index={slot} tracked={tracked}")
    }

    /// The line for the `Items` list of a shulker box: its length and the `Slot` of each element (`n/a` for an
    /// element with no readable `Slot`). `None` is a box with no `Items` list.
    pub fn items_line(items: Option<&[Option<i8>]>) -> String {
        let Some(slots) = items else {
            return "probe: shulker Items=n/a".to_string();
        };
        let list: Vec<String> =
            slots.iter().map(|slot| slot.map_or_else(|| "n/a".to_string(), |slot| slot.to_string())).collect();
        format!("probe: shulker Items length={} slots=[{}]", slots.len(), list.join(", "))
    }

    /// The line for the GUI size and the pointer, to compare with the screen edges.
    pub fn screen_line(size: Option<[f32; 2]>, pointer: Option<[f32; 2]>) -> String {
        fn pair(value: Option<[f32; 2]>) -> String {
            value.map_or_else(|| "n/a".to_string(), |[x, y]| format!("[{x:.1}, {y:.1}]"))
        }
        format!("probe: screen size={} pointer={}", pair(size), pair(pointer))
    }

    /// The line for a wheel step on a UI scene: the step, the Shift key, the thread of the step and of the last UI
    /// render, whether the scene's controller is a container screen (`n/a` when it could not be read) and whether it
    /// is the controller of the tracked hover.
    pub fn scroll_line(
        step: [f32; 2],
        shift: bool,
        thread: u32,
        render_thread: u32,
        container: Option<bool>,
        hovered_controller: bool,
    ) -> String {
        let container_text = container.map_or_else(|| "n/a".to_string(), |container| container.to_string());
        let [x, y] = step;
        format!(
            "probe: scroll x={x} y={y} shift={shift} thread={thread} render_thread={render_thread} \
             container={container_text} hovered_controller={hovered_controller}"
        )
    }

    /// The line for the first stored item built by `ItemStack::fromTag`.
    pub fn built_line(type_name: Option<&str>, count: Option<u8>) -> String {
        let count_text = count.map_or_else(|| "n/a".to_string(), |count| count.to_string());
        format!("probe: from_tag built type={} count={count_text}", type_name.unwrap_or("n/a"))
    }

    /// The line for the first frame that draws the cell for a hover; `text_drawn` is whether the count text was
    /// queued and flushed.
    pub fn drew_line(pointer: [f32; 2], text: &str, text_drawn: bool) -> String {
        format!(
            "probe: drew cell 16x16 at [{:.1}, {:.1}] text={text:?} text_drawn={text_drawn}",
            pointer[0], pointer[1]
        )
    }

    /// The line for a controller destructor; `tracked` is whether it was the controller the pointer was over.
    pub fn dtor_line(controller: usize, tracked: bool) -> String {
        format!("probe: controller dtor controller={controller:#x} tracked={tracked}")
    }

    /// The line for the first frame after the tracked controller was destroyed.
    pub fn after_dtor_line(drew: bool) -> String {
        format!("probe: first render after controller dtor drew={drew}")
    }
}

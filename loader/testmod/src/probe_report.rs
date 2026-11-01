use amml_game::ActorPose;

use crate::{ChunkReading, LightReading, NameTagLayout};

/// The text of the probe's log lines.
pub struct ProbeReport;

impl ProbeReport {
    /// The one line logged while the player is in a world. A value that could not be read shows as `n/a`.
    ///
    /// The block under the player is in its own line, see `block_line`.
    pub fn world_line(pose: Option<ActorPose>, dimension: Option<i32>, block_source: Option<bool>) -> String {
        let pose_text = match pose {
            Some(pose) => format!(
                "pos=[{:.2}, {:.2}, {:.2}] pitch={:.2} yaw={:.2}",
                pose.pos[0], pose.pos[1], pose.pos[2], pose.pitch, pose.yaw
            ),
            None => "pos=n/a pitch=n/a yaw=n/a".to_string(),
        };
        let dimension_text = dimension.map_or_else(|| "n/a".to_string(), |id| id.to_string());
        let source_text = match block_source {
            Some(true) => "ok",
            Some(false) => "none",
            None => "n/a",
        };
        format!("probe: {pose_text} dimension={dimension_text} block_source={source_text}")
    }

    /// The line for the block under the player's feet: its position, its full type name and the solid flag.
    pub fn block_line(pos: [i32; 3], name: Option<&str>, solid: Option<bool>) -> String {
        let solid_text = solid.map_or_else(|| "n/a".to_string(), |solid| solid.to_string());
        format!(
            "probe: block pos=[{}, {}, {}] name={} solid={solid_text}",
            pos[0],
            pos[1],
            pos[2],
            name.unwrap_or("n/a")
        )
    }

    /// The line for a block whose y is outside the chunk's range, so it was not read.
    pub fn block_out_of_range_line(pos: [i32; 3]) -> String {
        format!("probe: block pos=[{}, {}, {}] block=out of range", pos[0], pos[1], pos[2])
    }

    /// The line for the chunk of the player's column.
    pub fn chunk_line(chunk: &ChunkReading) -> String {
        fn text<T: ToString>(value: Option<T>) -> String {
            value.map_or_else(|| "n/a".to_string(), |value| value.to_string())
        }
        format!(
            "probe: chunk pos={} min_y={} max_y={} loaded={} sub_chunks={} height[{}]={}",
            chunk.pos.map_or_else(|| "n/a".to_string(), |[x, z]| format!("[{x}, {z}]")),
            text(chunk.min_y),
            text(chunk.max_y),
            chunk.loaded,
            chunk.sub_chunks,
            chunk.index,
            text(chunk.height)
        )
    }

    /// The line for the light and the surface tests at the player's feet. A value that could not be read shows as
    /// `n/a`.
    pub fn light_line(reading: &LightReading) -> String {
        fn text<T: ToString>(value: Option<T>) -> String {
            value.map_or_else(|| "n/a".to_string(), |value| value.to_string())
        }
        let light = reading.brightness.map_or_else(
            || "sky=n/a block=n/a".to_string(),
            |light| format!("sky={} block={}", light.sky, light.block),
        );
        let shape = reading.feet_shape.map_or_else(
            || "n/a".to_string(),
            |shape| {
                format!(
                    "[{:.2}, {:.2}, {:.2}]-[{:.2}, {:.2}, {:.2}] empty={}",
                    shape.min[0],
                    shape.min[1],
                    shape.min[2],
                    shape.max[0],
                    shape.max[1],
                    shape.max[2],
                    shape.is_empty()
                )
            },
        );
        format!(
            "probe: light feet=[{}, {}, {}] {light} sky_darken={} under_air={} under_full_support={} feet_air={} feet_shape={shape}",
            reading.feet[0],
            reading.feet[1],
            reading.feet[2],
            text(reading.sky_darken),
            text(reading.under_air),
            text(reading.under_full_support),
            text(reading.feet_air),
        )
    }

    /// The line for the name-tag list the game made: the tag count and the layout check, or the reason the layout
    /// is not the expected one. `None` is a list that could not be read.
    pub fn name_tags_line(layout: Option<&NameTagLayout>) -> String {
        let Some(layout) = layout else {
            return "probe: name_tags n/a".to_string();
        };
        match layout.count() {
            Ok(count) => format!("probe: name_tags count={count} layout=ok"),
            Err(fault) => format!("probe: name_tags layout=broken ({})", fault.name()),
        }
    }

    /// The line for a camera value read in a render hook; `name` is `forward` or `view_position`.
    pub fn camera_line(name: &str, value: Option<[f32; 3]>) -> String {
        match value {
            Some([x, y, z]) => format!("probe: camera {name}=[{x:.3}, {y:.3}, {z:.3}]"),
            None => format!("probe: camera {name}=n/a"),
        }
    }

    /// The enable line of the peek entries: the features that are off, each with its cause, or `none`.
    pub fn peek_features_line(off: &[(&'static str, String)]) -> String {
        let parts: Vec<String> = off.iter().map(|(feature, cause)| format!("{feature} ({cause})")).collect();
        if parts.is_empty() {
            return "probe: peek features off: none".to_string();
        }
        format!("probe: peek features off: {}", parts.join("; "))
    }

    /// The enable line: the features that are off, each with its cause, or `none`.
    pub fn features_line(off: &[(&'static str, String)]) -> String {
        if off.is_empty() {
            return "probe: features off: none".to_string();
        }
        let parts: Vec<String> = off.iter().map(|(feature, cause)| format!("{feature} ({cause})")).collect();
        format!("probe: features off: {}", parts.join("; "))
    }
}

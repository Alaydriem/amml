use std::collections::BTreeMap;
use std::path::Path;

use amml::{CameraLayout, CameraLayoutError, CameraProjection, CameraWriter, ViewRenderDataLayout};
use amml_data::{AdapterCatalog, BuildDataStore};

// The data files of the two gate builds, 26.63 and 70.26.
const GATE_BUILDS: [&str; 2] = ["6aed1906-b127418e74d2f2f9", "6afa4e47-dadc375797a3d31a"];

// The values of every entry of the `camera.pose` and `camera.projection` rows in the data file of `build`.
fn gate_values(build: &str) -> BTreeMap<String, u64> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data").join(format!("{build}.hcl"));
    let data = BuildDataStore::read(&path).unwrap();
    let row: Vec<&str> = AdapterCatalog::needs()
        .iter()
        .filter(|need| need.id == "camera.pose" || need.id == "camera.projection")
        .flat_map(|need| need.shapes)
        .flat_map(|shape| shape.entries)
        .copied()
        .collect();
    assert!(!row.is_empty());
    row.into_iter()
        .filter_map(|name| {
            data.entries.iter().find(|(id, _)| id.0 == name).map(|(_, resolved)| (name.to_string(), resolved.value))
        })
        .collect()
}

fn writer(values: &BTreeMap<String, u64>) -> Result<CameraWriter, CameraLayoutError> {
    CameraWriter::from_lookup(&|name: &str| values.get(name).copied())
}

fn projection(values: &BTreeMap<String, u64>) -> Result<CameraProjection, CameraLayoutError> {
    CameraProjection::from_lookup(&|name: &str| values.get(name).copied())
}

#[test]
fn camera_mirror_anchor_is_forward() {
    assert_eq!(CameraLayout::FORWARD, 0x118);
    for build in GATE_BUILDS {
        let mut values = gate_values(build);
        assert_eq!(values.get("Camera.mForward"), Some(&CameraLayout::FORWARD), "{build}");
        assert!(writer(&values).is_ok(), "{build}: {:?}", writer(&values).err());
        assert!(projection(&values).is_ok(), "{build}");

        values.insert("Camera.mForward".to_string(), 0x120);

        let mismatch = CameraLayoutError::Mismatch { entry: "Camera.mForward", expected: 0x118, found: 0x120 };
        assert_eq!(writer(&values).err(), Some(mismatch), "{build}");
        assert_eq!(projection(&values).err(), Some(mismatch), "{build}");
    }
}

#[test]
fn view_data_mirror_needs_the_target_right_after_the_position() {
    let target_at = std::mem::offset_of!(ViewRenderDataLayout, camera_target_pos) as u64;
    for build in GATE_BUILDS {
        let mut values = gate_values(build);
        let position = values["LevelRendererCamera.mCameraPos"];
        // createViewRenderObject copies the level camera position and target as one 24-byte run (survey 8.3).
        assert_eq!(values["LevelRendererCamera.mCameraTargetPos"], position + target_at, "{build}");

        values.insert("LevelRendererCamera.mCameraTargetPos".to_string(), position + 0x10);

        assert_eq!(writer(&values).err().map(|error| error.entry()), Some("LevelRendererCamera.mCameraTargetPos"));
    }
}

#[test]
fn a_missing_entry_is_named() {
    let mut values = gate_values(GATE_BUILDS[1]);
    values.remove("Camera.updateViewMatrixDependencies");

    assert_eq!(
        writer(&values).err(),
        Some(CameraLayoutError::Missing { entry: "Camera.updateViewMatrixDependencies" })
    );
    // The projection needs the anchor only.
    assert!(projection(&values).is_ok());
}

use std::collections::BTreeMap;
use std::path::Path;

use amml::{MeshCalls, MeshDataLayout, MeshLayoutError, TessellatorLayout};
use amml_data::{AdapterCatalog, BuildDataStore};

// The data files of the two gate builds, 26.63 and 70.26.
const GATE_BUILDS: [&str; 2] = ["6aed1906-b127418e74d2f2f9", "6afa4e47-dadc375797a3d31a"];

// The values of every entry of the `render.world_canvas` row in the data file of `build`.
fn gate_values(build: &str) -> BTreeMap<String, u64> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data").join(format!("{build}.hcl"));
    let data = BuildDataStore::read(&path).unwrap();
    let row: Vec<&str> = AdapterCatalog::needs()
        .iter()
        .filter(|need| need.id == "render.world_canvas")
        .flat_map(|need| need.shapes)
        .flat_map(|shape| shape.entries)
        .copied()
        .collect();
    row.into_iter()
        .filter_map(|name| {
            data.entries.iter().find(|(id, _)| id.0 == name).map(|(_, resolved)| (name.to_string(), resolved.value))
        })
        .collect()
}

fn calls(values: &BTreeMap<String, u64>) -> Result<MeshCalls, MeshLayoutError> {
    MeshCalls::from_lookup(&|name: &str| values.get(name).copied())
}

#[test]
fn tessellator_layout_anchor_matches_the_entry() {
    for build in GATE_BUILDS {
        let mut values = gate_values(build);
        assert_eq!(values.get("Tessellator.mTessellating"), Some(&TessellatorLayout::TESSELLATING), "{build}");
        assert!(calls(&values).is_ok(), "{build}: {:?}", calls(&values));

        values.insert("Tessellator.mTessellating".to_string(), 0x2A8);

        assert_eq!(
            calls(&values),
            Err(MeshLayoutError::Mismatch { entry: "Tessellator.mTessellating", expected: 0x2A0, found: 0x2A8 })
        );
    }
}

#[test]
fn mesh_data_layout_size_matches_the_survey() {
    // Survey section 8.2: `Tessellator.clear` writes the member after the mesh data at 168h (8 + 160h) on both builds.
    assert_eq!(size_of::<MeshDataLayout>(), 0x160);
    assert_eq!(MeshDataLayout::SIZE, 0x160);
}

#[test]
fn every_mirror_mismatch_names_its_entry() {
    let values = gate_values(GATE_BUILDS[1]);
    for (entry, wrong) in [
        ("Mesh.size", 0x278),
        ("BaseActorRenderer.size", 0xC0),
        ("ScreenContext.camera", 0x20),
        ("ScreenContext.constantBuffers", 0x28),
        ("ScreenContext.currentShaderColor", 0x38),
        ("Camera.mForward", 0x110),
    ] {
        let mut changed = values.clone();
        changed.insert(entry.to_string(), wrong);

        assert_eq!(calls(&changed).map_err(|error| error.entry()), Err(entry));
    }
    let mut missing = values;
    missing.remove("Mesh.renderMeshTextureList");
    assert_eq!(calls(&missing), Err(MeshLayoutError::Missing { entry: "Mesh.renderMeshTextureList" }));
}

use amml_registry::{ManifestReader, RegistryError};

const MANIFEST: &str = r#"
namespace   = "alaydriem"
name        = "minimap"
entry       = "alaydriem_minimap.dll"
api_version = 2
version     = "26.60.1"
entries = [
  "ClientInstance.update",
]

listing {
  title       = "Minimap"
  summary     = "A small map in the corner of the screen."
  description = <<-EOT
    A small map in the corner of the screen that follows you.

    Zoom with the mouse wheel while the map is open.
  EOT
  authors     = ["Charles R. Portwood II"]
  license     = "BSD-3-Clause"
  source      = "https://github.com/alaydriem/amml-minimap"
  tags        = ["map", "hud"]
  icon        = "icon.png"
}
"#;

#[test]
fn reads_loader_fields_and_listing() {
    let manifest = ManifestReader::parse(MANIFEST).unwrap();
    assert_eq!(manifest.namespace, "alaydriem");
    assert_eq!(manifest.name, "minimap");
    assert_eq!(manifest.api_version, 2);
    assert_eq!(manifest.version, "26.60.1");
    let listing = manifest.listing.unwrap();
    assert_eq!(listing.title, "Minimap");
    assert_eq!(listing.tags, ["map", "hud"]);
    assert_eq!(listing.icon.as_deref(), Some("icon.png"));
    let description = listing.description.unwrap();
    assert!(description.starts_with("A small map in the corner of the screen that follows you.\n\nZoom"));
}

#[test]
fn a_manifest_without_listing_has_none() {
    let manifest = ManifestReader::parse("namespace = \"a\"\nname = \"b\"\napi_version = 2\n").unwrap();
    assert!(manifest.listing.is_none());
    assert_eq!(manifest.version, "");
}

#[test]
fn unknown_listing_field_is_refused() {
    let text = MANIFEST.replace("tags        =", "tagz        =");
    assert!(matches!(ManifestReader::parse(&text), Err(RegistryError::Listing { field: "listing", .. })));
}

#[test]
fn missing_required_listing_field_is_refused() {
    let text = MANIFEST.replace("  license     = \"BSD-3-Clause\"\n", "");
    assert!(matches!(ManifestReader::parse(&text), Err(RegistryError::Listing { field: "listing", .. })));
}

#[test]
fn text_that_is_not_hcl_is_refused() {
    assert!(matches!(ManifestReader::parse("this is { not"), Err(RegistryError::Listing { field: "amml.hcl", .. })));
}

#[test]
fn a_missing_or_zero_api_version_is_refused() {
    let none = MANIFEST.replace("api_version = 2\n", "");
    let zero = MANIFEST.replace("api_version = 2", "api_version = 0");
    for text in [none, zero] {
        assert!(matches!(
            ManifestReader::parse(&text),
            Err(RegistryError::ManifestMismatch { field: "api_version", .. })
        ));
    }
}

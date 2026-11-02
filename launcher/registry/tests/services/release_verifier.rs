use amml_registry::{RegistryError, ReleaseVerifier};
use tempfile::TempDir;

use crate::support::{DirReleaseSource, Samples, ZipFixture};

const REPO: &str = "alaydriem/amml-zoom";
const ASSET: &str = "zoom-26.60.1-windows-x64.zip";
const AMML_ASSET: &str = "amml-26.60.1-windows-x64.zip";

fn rig() -> (TempDir, DirReleaseSource) {
    let dir = tempfile::tempdir().unwrap();
    let source = DirReleaseSource::new(dir.path().to_path_buf());
    (dir, source)
}

fn good_mod_zip() -> Vec<u8> {
    ZipFixture::new()
        .file("@alaydriem/zoom/amml.hcl", ZipFixture::manifest("zoom", Some("26.60.1"), None).as_bytes())
        .file("@alaydriem/zoom/alaydriem_zoom.dll", b"MZ")
        .build()
}

fn verify(source: DirReleaseSource) -> Result<amml_registry::VerifiedMod, RegistryError> {
    ReleaseVerifier::new(source).check_mod(REPO, "alaydriem", "zoom", "26.60.1", Samples::NOW)
}

#[test]
fn good_mod_release_passes() {
    let (_dir, source) = rig();
    let zip = good_mod_zip();
    source.put_release(REPO, "26.60.1", ASSET, &zip);
    let verified = verify(source).unwrap();
    let release = verified.release;
    assert_eq!(release.version, "26.60.1");
    assert_eq!(release.api_version, 2);
    assert_eq!(
        release.asset,
        "https://github.com/alaydriem/amml-zoom/releases/download/26.60.1/zoom-26.60.1-windows-x64.zip"
    );
    assert_eq!(release.sha256, amml_registry::HashText::sha256(&zip));
    assert_eq!(release.size, zip.len() as u64);
    assert_eq!(release.published, Samples::NOW);
    assert_eq!(verified.listing.title, "Zoom");
    assert!(verified.icon.is_none());
}

#[test]
fn icon_is_copied_and_its_path_rewritten() {
    let (_dir, source) = rig();
    let png = ZipFixture::png(128, 128, 3000);
    let zip = ZipFixture::new()
        .file(
            "@alaydriem/zoom/amml.hcl",
            ZipFixture::manifest("zoom", Some("26.60.1"), Some("art/icon.png")).as_bytes(),
        )
        .file("@alaydriem/zoom/art/icon.png", &png)
        .build();
    source.put_release(REPO, "26.60.1", ASSET, &zip);
    let verified = verify(source).unwrap();
    assert_eq!(verified.icon.as_deref(), Some(png.as_slice()));
    assert_eq!(verified.listing.icon.as_deref(), Some("icons/@alaydriem/zoom.png"));
}

#[test]
fn bad_icon_or_missing_icon_fails_with_the_icon_field() {
    for icon_bytes in [Some(ZipFixture::png(64, 64, 3000)), None] {
        let (_dir, source) = rig();
        let mut zip = ZipFixture::new().file(
            "@alaydriem/zoom/amml.hcl",
            ZipFixture::manifest("zoom", Some("26.60.1"), Some("icon.png")).as_bytes(),
        );
        if let Some(bytes) = &icon_bytes {
            zip = zip.file("@alaydriem/zoom/icon.png", bytes);
        }
        source.put_release(REPO, "26.60.1", ASSET, &zip.build());
        assert!(matches!(verify(source), Err(RegistryError::Listing { field: "icon", .. })));
    }
}

#[test]
fn listing_is_required_and_checked() {
    let (_dir, source) = rig();
    let bare = "namespace = \"alaydriem\"\nname = \"zoom\"\napi_version = 2\nversion = \"26.60.1\"\n";
    let zip = ZipFixture::new().file("@alaydriem/zoom/amml.hcl", bare.as_bytes()).build();
    source.put_release(REPO, "26.60.1", ASSET, &zip);
    assert!(matches!(verify(source), Err(RegistryError::Listing { field: "listing", .. })));

    let (_dir, source) = rig();
    let long = ZipFixture::manifest("zoom", Some("26.60.1"), None).replace("Hold a key to zoom.", &"x".repeat(121));
    let zip = ZipFixture::new().file("@alaydriem/zoom/amml.hcl", long.as_bytes()).build();
    source.put_release(REPO, "26.60.1", ASSET, &zip);
    assert!(matches!(verify(source), Err(RegistryError::Listing { field: "summary", .. })));
}

#[test]
fn hash_mismatch_fails() {
    let (_dir, source) = rig();
    let mut zip = good_mod_zip();
    source.put_release(REPO, "26.60.1", ASSET, &zip);
    let last = zip.len() - 1;
    zip[last] ^= 0xff;
    source.put(REPO, "26.60.1", ASSET, &zip);
    match verify(source) {
        Err(RegistryError::HashMismatch { asset }) => assert_eq!(asset, ASSET),
        other => panic!("{other:?}"),
    }
}

#[test]
fn manifest_fields_must_match() {
    let cases = [
        (ZipFixture::manifest("zoom2", Some("26.60.1"), None), "name"),
        (ZipFixture::manifest("zoom", None, None), "version"),
        (ZipFixture::manifest("zoom", Some("26.60.0"), None), "version"),
        (
            ZipFixture::manifest("zoom", Some("26.60.1"), None).replace("alaydriem\"\nname", "other\"\nname"),
            "namespace",
        ),
    ];
    for (manifest, field) in cases {
        let (_dir, source) = rig();
        let zip = ZipFixture::new().file("@alaydriem/zoom/amml.hcl", manifest.as_bytes()).build();
        source.put_release(REPO, "26.60.1", ASSET, &zip);
        match verify(source) {
            Err(RegistryError::ManifestMismatch { field: got, .. }) => assert_eq!(got, field),
            other => panic!("{field}: {other:?}"),
        }
    }
}

#[test]
fn missing_version_reports_an_empty_got() {
    let (_dir, source) = rig();
    let manifest = ZipFixture::manifest("zoom", None, None);
    let zip = ZipFixture::new().file("@alaydriem/zoom/amml.hcl", manifest.as_bytes()).build();
    source.put_release(REPO, "26.60.1", ASSET, &zip);
    match verify(source) {
        Err(RegistryError::ManifestMismatch { field: "version", got, expected }) => {
            assert_eq!(got, "");
            assert_eq!(expected, "26.60.1");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn two_top_folders_is_bad_zip() {
    let (_dir, source) = rig();
    let manifest = ZipFixture::manifest("zoom", Some("26.60.1"), None);
    let zip = ZipFixture::new()
        .file("@alaydriem/zoom/amml.hcl", manifest.as_bytes())
        .file("@alaydriem/x/amml.hcl", manifest.as_bytes())
        .build();
    source.put_release(REPO, "26.60.1", ASSET, &zip);
    assert!(matches!(verify(source), Err(RegistryError::BadZip(_))));
}

#[test]
fn a_file_outside_the_mod_folder_is_bad_zip() {
    let (_dir, source) = rig();
    let manifest = ZipFixture::manifest("zoom", Some("26.60.1"), None);
    let zip = ZipFixture::new().file("@alaydriem/zoom/amml.hcl", manifest.as_bytes()).file("stray.txt", b"x").build();
    source.put_release(REPO, "26.60.1", ASSET, &zip);
    assert!(matches!(verify(source), Err(RegistryError::BadZip(_))));
}

#[test]
fn missing_asset_is_missing() {
    let (_dir, source) = rig();
    match verify(source) {
        Err(RegistryError::Missing(asset)) => assert_eq!(asset, "SHA256SUMS.txt"),
        other => panic!("{other:?}"),
    }
    let (_dir, source) = rig();
    source.put(REPO, "26.60.1", "SHA256SUMS.txt", b"");
    match verify(source) {
        Err(RegistryError::Missing(asset)) => assert_eq!(asset, ASSET),
        other => panic!("{other:?}"),
    }
}

fn good_amml_zip(version_text: &str) -> Vec<u8> {
    ZipFixture::new()
        .file("amml.dll", b"MZ-loader")
        .file("amml-patch.exe", b"MZ-patch")
        .file("amml/recipes/a.hcl", b"x")
        .file("amml/amml-version.txt", version_text.as_bytes())
        .build()
}

fn verify_amml(source: DirReleaseSource, game_versions: &[&str]) -> Result<amml_registry::AmmlRelease, RegistryError> {
    let game_versions: Vec<String> = game_versions.iter().map(|v| v.to_string()).collect();
    ReleaseVerifier::new(source).check_amml("alaydriem/amml", "26.60.1", 2, &game_versions)
}

#[test]
fn good_amml_release_passes() {
    let (_dir, source) = rig();
    let zip = good_amml_zip("26.60.1");
    source.put_release("alaydriem/amml", "26.60.1", AMML_ASSET, &zip);
    let release = verify_amml(source, &["1.26.60.5"]).unwrap();
    assert_eq!(release.version, "26.60.1");
    assert_eq!(release.api_version, 2);
    assert_eq!(release.game_versions, ["1.26.60.5"]);
    assert_eq!(release.size, zip.len() as u64);
    assert_eq!(
        release.asset,
        "https://github.com/alaydriem/amml/releases/download/26.60.1/amml-26.60.1-windows-x64.zip"
    );
}

#[test]
fn amml_version_file_must_match() {
    let (_dir, source) = rig();
    source.put_release("alaydriem/amml", "26.60.1", AMML_ASSET, &good_amml_zip("26.60.0"));
    assert!(matches!(
        verify_amml(source, &["1.26.60.5"]),
        Err(RegistryError::ManifestMismatch { field: "amml-version.txt", .. })
    ));
}

#[test]
fn amml_zip_needs_both_executables_and_recipes() {
    let cases = [
        ZipFixture::new()
            .file("amml-patch.exe", b"MZ")
            .file("amml/recipes/a.hcl", b"x")
            .file("amml/amml-version.txt", b"26.60.1"),
        ZipFixture::new()
            .file("amml.dll", b"MZ")
            .file("amml/recipes/a.hcl", b"x")
            .file("amml/amml-version.txt", b"26.60.1"),
        ZipFixture::new()
            .file("amml.dll", b"MZ")
            .file("amml-patch.exe", b"MZ")
            .file("amml/amml-version.txt", b"26.60.1"),
        ZipFixture::new()
            .file("amml.dll", b"XX")
            .file("amml-patch.exe", b"MZ")
            .file("amml/recipes/a.hcl", b"x")
            .file("amml/amml-version.txt", b"26.60.1"),
    ];
    for fixture in cases {
        let (_dir, source) = rig();
        source.put_release("alaydriem/amml", "26.60.1", AMML_ASSET, &fixture.build());
        assert!(matches!(verify_amml(source, &["1.26.60.5"]), Err(RegistryError::BadZip(_))));
    }
}

#[test]
fn bad_game_version_is_refused_before_any_download() {
    let (_dir, source) = rig();
    assert!(matches!(verify_amml(source, &["1.26"]), Err(RegistryError::BadVersion(version)) if version == "1.26"));
}

#[test]
fn entry_paths_that_leave_the_folder_are_bad_zip() {
    let manifest = ZipFixture::manifest("zoom", Some("26.60.1"), None);
    for evil in ["@alaydriem/zoom/../x.dll", "@alaydriem/zoom/./x.dll", "@alaydriem/zoom//x.dll", "@alaydriem/zoom/a:b"]
    {
        let (_dir, source) = rig();
        let zip = ZipFixture::new().file("@alaydriem/zoom/amml.hcl", manifest.as_bytes()).file(evil, b"x").build();
        source.put_release(REPO, "26.60.1", ASSET, &zip);
        assert!(matches!(verify(source), Err(RegistryError::BadZip(_))), "{evil}");
    }
}

#[test]
fn amml_zip_entry_paths_are_checked_too() {
    let (_dir, source) = rig();
    let zip = ZipFixture::new()
        .file("amml.dll", b"MZ")
        .file("amml-patch.exe", b"MZ")
        .file("amml/recipes/a.hcl", b"x")
        .file("amml/amml-version.txt", b"26.60.1")
        .file("amml/../../outside.dll", b"x")
        .build();
    source.put_release("alaydriem/amml", "26.60.1", AMML_ASSET, &zip);
    assert!(matches!(verify_amml(source, &["1.26.60.5"]), Err(RegistryError::BadZip(_))));
}

#[test]
fn a_manifest_without_api_version_is_refused() {
    let (_dir, source) = rig();
    let manifest = ZipFixture::manifest("zoom", Some("26.60.1"), None).replace("api_version = 2\n", "");
    let zip = ZipFixture::new().file("@alaydriem/zoom/amml.hcl", manifest.as_bytes()).build();
    source.put_release(REPO, "26.60.1", ASSET, &zip);
    assert!(matches!(verify(source), Err(RegistryError::ManifestMismatch { field: "api_version", .. })));
}

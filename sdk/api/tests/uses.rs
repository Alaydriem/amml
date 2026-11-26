use amml_abi::{AmmlStr, AmmlUseRow, ClientTick, ModsLoaded};
use amml_api::{Area, Uses};
use amml_derive::ModUses;
use amml_runtime::ModRegistry;

const USES: Uses = amml_api::amml_uses! {
    feature "core" { required ClientTick; optional ModsLoaded; }
    feature "input" { required area Area::INPUT_LEASE; optional area Area::KEY_INJECT; }
};

// A feature name with a quote, a backslash and both HCL template starts.
const ODD_NAMES: Uses = amml_api::amml_uses! {
    feature "say \"hi\" \\ ${x} %{y}" { required ClientTick; }
};

type Row = (String, u8, String, u32, bool);

const HEAD: &str = "namespace = \"acme\"\nname = \"m\"\nentry = \"acme_m.dll\"\napi_version = 3\n";

fn text(view: AmmlStr) -> String {
    // SAFETY: the rows of `amml_uses!` point to static strings.
    unsafe { view.as_str() }.unwrap().to_string()
}

fn tuples_of(rows: &[AmmlUseRow]) -> Vec<Row> {
    rows.iter().map(|row| (text(row.feature), row.kind, text(row.name), row.version, row.required != 0)).collect()
}

// The rows the compat check reads from the manifest, per feature its events and then its areas.
fn read_with_compat_reader(manifest: &str) -> Vec<Row> {
    let uses = ModUses::parse(std::path::Path::new("."), manifest).unwrap();
    let mut rows = Vec::new();
    for feature in uses.features {
        for event in feature.events {
            rows.push((feature.name.clone(), AmmlUseRow::KIND_EVENT, event.name, event.version, event.required));
        }
        for area in feature.areas {
            rows.push((feature.name.clone(), AmmlUseRow::KIND_AREA, area.name, area.version, area.required));
        }
    }
    rows
}

// Whether the loader's mod scan accepts `manifest` as the manifest of `@acme/m`.
fn loader_accepts(manifest: &str) -> bool {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("@acme").join("m");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(ModRegistry::MANIFEST), manifest).unwrap();
    let scan = ModRegistry::new(ModRegistry::MAX_MODS).scan(root.path()).unwrap();
    scan.skipped.is_empty() && scan.mods.len() == 1
}

#[test]
fn to_hcl_is_read_back_by_the_loader_manifest_reader() {
    let manifest = format!("{HEAD}{}", USES.to_hcl());

    assert!(loader_accepts(&manifest), "{manifest}");
    assert_eq!(read_with_compat_reader(&manifest), tuples_of(USES.rows()));
}

#[test]
fn to_hcl_writes_no_commas() {
    assert!(!USES.to_hcl().contains(','));
}

#[test]
fn to_hcl_writes_one_attribute_per_line_with_two_space_indent() {
    let expected = "\
uses {
  feature \"core\" {
    event \"client.tick\" {
      version  = 1
      required = true
    }
    event \"mods.loaded\" {
      version  = 1
      required = false
    }
  }
  feature \"input\" {
    area \"input.lease\" {
      version  = 1
      required = true
    }
    area \"input.key_inject\" {
      version  = 1
      required = false
    }
  }
}
";
    assert_eq!(USES.to_hcl(), expected);
}

#[test]
fn to_hcl_escapes_quotes_backslashes_and_templates() {
    let manifest = format!("{HEAD}{}", ODD_NAMES.to_hcl());

    assert!(loader_accepts(&manifest), "{manifest}");
    assert_eq!(read_with_compat_reader(&manifest), tuples_of(ODD_NAMES.rows()));
}

const LISTING: &str = "\nlisting {\n  title = \"M\"\n}\n";

#[test]
fn write_into_replaces_the_uses_block_and_keeps_the_rest() {
    let manifest = format!("{HEAD}{}{LISTING}", Uses::NONE.to_hcl());

    let written = USES.write_into(&manifest);

    assert_eq!(written, format!("{HEAD}{}{LISTING}", USES.to_hcl()));
}

#[test]
fn write_into_puts_a_missing_block_before_the_listing() {
    let written = USES.write_into(&format!("{HEAD}{LISTING}"));

    assert_eq!(written, format!("{HEAD}\n{}{LISTING}", USES.to_hcl()));
}

#[test]
fn write_into_appends_a_missing_block_to_a_manifest_without_a_listing() {
    let written = USES.write_into("namespace = \"acme\"");

    assert_eq!(written, format!("namespace = \"acme\"\n{}", USES.to_hcl()));
}

#[test]
fn write_into_keeps_crlf_and_a_second_run_adds_no_second_block() {
    let manifest = format!("{HEAD}{}{LISTING}", Uses::NONE.to_hcl()).replace('\n', "\r\n");

    let once = USES.write_into(&manifest);
    let twice = USES.write_into(&once);

    let expected = format!("{HEAD}{}{LISTING}", USES.to_hcl()).replace('\n', "\r\n");
    assert_eq!(once, expected);
    assert_eq!(twice, expected);
    assert_eq!(twice.matches("uses {").count(), 1);
}

#[test]
fn write_into_makes_a_mixed_manifest_crlf() {
    let manifest = format!("{}{}{}", HEAD.replace('\n', "\r\n"), Uses::NONE.to_hcl(), LISTING.replace('\n', "\r\n"));

    let written = USES.write_into(&manifest);

    assert_eq!(written, format!("{HEAD}{}{LISTING}", USES.to_hcl()).replace('\n', "\r\n"));
}

#[test]
fn write_into_removes_extra_uses_blocks() {
    let block = Uses::NONE.to_hcl();
    let manifest = format!("{HEAD}{block}{block}{LISTING}{block}");

    let written = USES.write_into(&manifest);

    assert_eq!(written, format!("{HEAD}{}{LISTING}", USES.to_hcl()));
}

#[test]
fn write_into_does_not_take_a_uses_inside_another_line() {
    let manifest = format!("{HEAD}description = \"uses {{\"\n{LISTING}");

    let written = USES.write_into(&manifest);

    assert!(written.contains("description = \"uses {\"\n"), "{written}");
    assert_eq!(written.matches(&USES.to_hcl()).count(), 1, "{written}");
}

#[test]
fn to_hcl_of_no_uses_is_an_empty_block_the_loader_accepts() {
    let manifest = format!("{HEAD}{}", Uses::NONE.to_hcl());

    assert_eq!(Uses::NONE.to_hcl(), "uses {\n}\n");
    assert!(loader_accepts(&manifest));
    assert!(read_with_compat_reader(&manifest).is_empty());
}

#[test]
fn an_area_at_a_later_table_version_writes_that_version() {
    const LATER: Uses = amml_api::amml_uses! {
        feature "death" { required area Area::ACTOR_READ; optional area Area::ACTOR_READ.at(2); }
    };
    assert_eq!(
        tuples_of(LATER.rows()),
        [
            ("death".to_string(), AmmlUseRow::KIND_AREA, "actor.read".to_string(), 1, true),
            ("death".to_string(), AmmlUseRow::KIND_AREA, "actor.read".to_string(), 2, false),
        ]
    );
    assert_eq!(read_with_compat_reader(&format!("{HEAD}{}", LATER.to_hcl())), tuples_of(LATER.rows()));
}

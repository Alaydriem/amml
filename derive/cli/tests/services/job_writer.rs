use std::fs;
use std::path::Path;

use amml_data::{
    BuildData, Check, Entry, EntryId, EntryKind, OperandKind, Recipe, RecipeLoader, RecipeSet, Resolved, Step,
};
use amml_derive::testing::ImageFixture;
use amml_derive::{DeriveOutcome, Deriver, Image, JobWriter};
use tempfile::TempDir;

const BASE: u64 = 0x1_4000_0000;

fn id(name: &str) -> EntryId {
    EntryId(name.to_string())
}

fn entry(name: &str, steps: Vec<Step>) -> Entry {
    Entry {
        id: id(name),
        kind: EntryKind::Function,
        msvc_name: Some("?f@@YAXXZ".to_string()),
        recipes: vec![Recipe { steps }],
        checks: Vec::new(),
        notes: format!("notes of {name}"),
        hooked: true,
    }
}

fn pattern(bytes: &str) -> Step {
    Step::Pattern { bytes: bytes.to_string(), offset: 0, literal_ok: Vec::new() }
}

// an image that holds `AA BB CC` once and nothing else
fn image() -> Image {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&[0xAA, 0xBB, 0xCC, 0xC3]);
    fx.image()
}

fn derive(image: &Image, set: &RecipeSet) -> DeriveOutcome {
    match Deriver::new(image, set).derive() {
        Ok(outcome) => outcome,
        Err(error) => panic!("{error}"),
    }
}

fn read(dir: &Path, name: &str) -> String {
    match fs::read_to_string(dir.join(name)) {
        Ok(text) => text,
        Err(error) => panic!("{name}: {error}"),
    }
}

fn old_data(image: &Image, id_text: &str, kind: EntryKind, value: u64) -> BuildData {
    let mut old = BuildData::new(image.build());
    old.entries.insert(id(id_text), Resolved { kind, value, recipes_agreeing: 1 });
    old
}

#[test]
fn job_per_failed_entry_and_none_for_passing() {
    let image = image();
    let mut fail = entry("Fail", vec![pattern("DD EE FF")]);
    fail.checks = vec![Check::Section { name: ".text".to_string() }];
    let set = RecipeSet::new(vec![entry("Pass", vec![pattern("AA BB CC")]), fail]).unwrap();
    let outcome = derive(&image, &set);
    let dir = TempDir::new().unwrap();
    let paths = JobWriter::new(&image, &set, &outcome, None).write(dir.path()).unwrap();
    let out = dir.path().join(image.build().file_stem());
    assert_eq!(paths, vec![out.join("jobs/Fail.md"), out.join("README.md")]);
    assert!(!out.join("jobs/Pass.md").exists());
    let job = read(&out.join("jobs"), "Fail.md");
    assert!(job.contains("- kind: function"), "{job}");
    assert!(job.contains("- msvc_name: ?f@@YAXXZ"), "{job}");
    assert!(job.contains("- hooked: yes"), "{job}");
    assert!(job.contains("## Checks\n\n- section .text\n"), "{job}");
    assert!(job.contains("notes of Fail"), "{job}");
    assert!(job.contains("recipe 0 step 0 pattern \"DD EE FF\""), "{job}");
    assert!(job.contains("- old value: unknown"), "{job}");
    assert!(job.contains("mise run inspect -- explain --exe <new exe> Fail"), "{job}");
    assert!(!job.contains("similar"), "{job}");
    let index = read(&out, "README.md");
    assert!(index.contains("[jobs/Fail.md](jobs/Fail.md)"), "{index}");
    assert!(!index.contains("Pass"), "{index}");
}

#[test]
fn an_entry_named_readme_cannot_overwrite_the_index() {
    let image = image();
    let set = RecipeSet::new(vec![entry("README", vec![pattern("DD EE FF")])]).unwrap();
    let outcome = derive(&image, &set);
    let dir = TempDir::new().unwrap();
    JobWriter::new(&image, &set, &outcome, None).write(dir.path()).unwrap();
    let out = dir.path().join(image.build().file_stem());
    assert!(read(&out, "README.md").starts_with("# Repair jobs for"));
    assert!(read(&out.join("jobs"), "README.md").starts_with("# README"));
}

#[test]
fn dependent_job_names_its_root() {
    let image = image();
    let set = RecipeSet::new(vec![
        entry("Y", vec![Step::Ref { entry: id("Z") }]),
        entry("Z", vec![pattern("DD EE FF")]),
        entry("A", vec![Step::Ref { entry: id("Y") }]),
        entry("M", vec![pattern("11 22 33")]),
        entry("ZQ", vec![pattern("AA BB CC")]),
        // fails for its own reason (no function starts there), and ranks after Z because it needs ZQ
        {
            let mut late = entry("C", vec![Step::Ref { entry: id("ZQ") }, Step::Add { value: 1 }]);
            late.checks = vec![Check::FunctionStart];
            late
        },
    ])
    .unwrap();
    let outcome = derive(&image, &set);
    let dir = TempDir::new().unwrap();
    JobWriter::new(&image, &set, &outcome, None).write(dir.path()).unwrap();
    let out = dir.path().join(image.build().file_stem());
    let jobs = out.join("jobs");
    assert!(read(&jobs, "Y.md").contains("- root: Z\n"));
    // a dependent of a dependent still names the entry that failed for its own reason
    assert!(read(&jobs, "A.md").contains("- root: Z\n"));
    assert!(read(&jobs, "Z.md").contains("- root: none"));
    let index = read(&out, "README.md");
    let at = |row: &str| index.find(row).unwrap_or_else(|| panic!("{row} missing in {index}"));
    // roots in dependency order: C sorts before M and Z by id but needs ZQ, which comes after Z
    assert!(at("| M |") < at("| Z |"));
    assert!(at("| Z |") < at("| C |"));
    assert!(read(&jobs, "C.md").contains("- root: none"));
    assert!(!index.contains("| ZQ |"), "{index}");
    // then the dependents; A sorts first by id but depends on Y, and Y on Z
    assert!(at("| C |") < at("| Y |"));
    assert!(at("| Y |") < at("| A |"));
}

#[test]
fn old_value_comes_from_compare_data() {
    let image = image();
    let set = RecipeSet::new(vec![entry("A", vec![pattern("DD EE FF")])]).unwrap();
    let outcome = derive(&image, &set);
    let old = old_data(&image, "A", EntryKind::Function, 0x1234);
    let dir = TempDir::new().unwrap();
    JobWriter::new(&image, &set, &outcome, Some(&old)).write(dir.path()).unwrap();
    let out = dir.path().join(image.build().file_stem());
    let job = read(&out.join("jobs"), "A.md");
    assert!(job.contains("- old value: 0x1234"), "{job}");
    assert!(job.contains("mise run inspect -- similar --exe <new exe> --old-exe <old exe> 0x1234"), "{job}");
    assert!(read(&out, "README.md").contains("| 0x1234 |"));
}

#[test]
fn similar_is_offered_only_for_functions() {
    let image = image();
    let mut slot = entry(
        "Slot",
        vec![
            pattern("DD EE FF"),
            Step::Operand { index: 0, kind: OperandKind::Displacement },
            Step::DisplacementToSlot,
        ],
    );
    slot.kind = EntryKind::VSlot;
    let set = RecipeSet::new(vec![slot]).unwrap();
    let outcome = derive(&image, &set);
    let old = old_data(&image, "Slot", EntryKind::VSlot, 0x93);
    let dir = TempDir::new().unwrap();
    JobWriter::new(&image, &set, &outcome, Some(&old)).write(dir.path()).unwrap();
    let job = read(&dir.path().join(image.build().file_stem()).join("jobs"), "Slot.md");
    assert!(job.contains("- old value: 0x93"), "{job}");
    assert!(!job.contains("similar"), "{job}");
    assert!(job.contains("mise run inspect -- explain"), "{job}");
}

#[test]
fn degraded_section_lists_failed_recipes_of_resolved_entries() {
    let image = image();
    let mut half = entry("Half", vec![pattern("AA BB CC")]);
    half.recipes.push(Recipe { steps: vec![pattern("DD EE FF")] });
    let set = RecipeSet::new(vec![half, entry("Clean", vec![pattern("AA BB CC")])]).unwrap();
    let outcome = derive(&image, &set);
    assert!(outcome.failures.is_empty());
    let dir = TempDir::new().unwrap();
    JobWriter::new(&image, &set, &outcome, None).write(dir.path()).unwrap();
    let index = read(&dir.path().join(image.build().file_stem()), "README.md");
    assert!(index.contains("- degraded: 1\n"), "{index}");
    assert!(index.contains("## Failed entries\n\nnone\n"), "{index}");
    assert!(index.contains("| Half | 1 | no match |"), "{index}");
    assert!(!index.contains("| Clean |"), "{index}");
}

#[test]
fn degraded_section_says_none_without_degraded_entries() {
    let image = image();
    let set = RecipeSet::new(vec![entry("Clean", vec![pattern("AA BB CC")])]).unwrap();
    let outcome = derive(&image, &set);
    let dir = TempDir::new().unwrap();
    JobWriter::new(&image, &set, &outcome, None).write(dir.path()).unwrap();
    let index = read(&dir.path().join(image.build().file_stem()), "README.md");
    assert!(index.ends_with("is derived.\n\nnone\n"), "{index}");
}

#[test]
fn the_job_names_its_recipe_file_relative_to_the_recipes_directory() {
    let image = image();
    let dir = TempDir::new().unwrap();
    let recipes = dir.path().join("recipes");
    fs::create_dir_all(recipes.join("sub")).unwrap();
    let text = "entry \"Lost\" {
  kind = \"function\"
  recipe {
    step \"pattern\" {
      bytes = \"DD EE FF\"
    }
  }
}
";
    fs::write(recipes.join("sub").join("a.hcl"), text).unwrap();
    let set = RecipeLoader::load_dir(&recipes).unwrap();
    let outcome = derive(&image, &set);
    let out = dir.path().join("out");
    JobWriter::new(&image, &set, &outcome, None).write(&out).unwrap();
    let job = read(&out.join(image.build().file_stem()).join("jobs"), "Lost.md");
    assert!(
        job.contains(
            "## Recipe file

sub/a.hcl
"
        ),
        "{job}"
    );
}

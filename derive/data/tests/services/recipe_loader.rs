use std::fs;
use std::path::Path;

use amml_data::{DataError, RecipeLoader};
use tempfile::TempDir;

struct Fixture;

impl Fixture {
    fn write(dir: &Path, relative: &str, text: &str) {
        let path = dir.join(relative);
        if let Some(parent) = path.parent() {
            assert!(fs::create_dir_all(parent).is_ok());
        }
        assert!(fs::write(path, text).is_ok());
    }

    fn entry(id: &str) -> String {
        format!(
            "entry \"{id}\" {{\n  kind = \"function\"\n  recipe {{\n    step \"pattern\" {{\n      bytes = \"48 89\"\n    }}\n  }}\n}}\n"
        )
    }

    fn entry_with_step(id: &str, step: &str) -> String {
        format!("entry \"{id}\" {{\n  kind = \"function\"\n  recipe {{\n    {step}\n  }}\n}}\n")
    }
}

#[test]
fn load_dir_reads_nested_files_in_sorted_path_order() {
    let dir = TempDir::new().unwrap();
    Fixture::write(dir.path(), "b/two.hcl", &Fixture::entry("B"));
    Fixture::write(dir.path(), "a/one.hcl", &Fixture::entry("A"));
    let set = RecipeLoader::load_dir(dir.path()).unwrap();
    let ids: Vec<_> = set.entries().iter().map(|e| e.id.0.clone()).collect();
    assert_eq!(ids, vec!["A", "B"]);
}

#[test]
fn unknown_step_type_names_file_and_entry() {
    let dir = TempDir::new().unwrap();
    Fixture::write(dir.path(), "bad.hcl", &Fixture::entry_with_step("X", "step \"teleport\" {}"));
    let err = RecipeLoader::load_dir(dir.path()).unwrap_err();
    match err {
        DataError::Parse { path, entry, message } => {
            assert!(path.ends_with("bad.hcl"));
            assert_eq!(entry.as_deref(), Some("X"));
            assert!(message.contains("teleport"));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn missing_kind_attribute_is_a_parse_error_for_that_entry() {
    let dir = TempDir::new().unwrap();
    Fixture::write(
        dir.path(),
        "nokind.hcl",
        "entry \"Y\" {\n  recipe {\n    step \"pattern\" {\n      bytes = \"48\"\n    }\n  }\n}\n",
    );
    let err = RecipeLoader::load_dir(dir.path()).unwrap_err();
    assert!(
        matches!(err, DataError::Parse { entry, message, .. } if entry.as_deref() == Some("Y") && message.contains("kind"))
    );
}

#[test]
fn transform_step_first_is_rejected() {
    let dir = TempDir::new().unwrap();
    Fixture::write(dir.path(), "z.hcl", &Fixture::entry_with_step("Z", "step \"call\" {\n      nth = 0\n    }"));
    let err = RecipeLoader::load_dir(dir.path()).unwrap_err();
    assert!(matches!(err, DataError::BadFirstStep { recipe: 0, .. }));
}

#[test]
fn duplicate_ids_across_files_are_rejected() {
    let dir = TempDir::new().unwrap();
    Fixture::write(dir.path(), "one.hcl", &Fixture::entry("A"));
    Fixture::write(dir.path(), "two.hcl", &Fixture::entry("A"));
    let err = RecipeLoader::load_dir(dir.path()).unwrap_err();
    assert!(matches!(err, DataError::DuplicateId { id, .. } if id.0 == "A"));
}

#[test]
fn ref_to_missing_entry_is_rejected() {
    let dir = TempDir::new().unwrap();
    Fixture::write(
        dir.path(),
        "r.hcl",
        &Fixture::entry_with_step("R", "step \"ref\" {\n      entry = \"Nope\"\n    }"),
    );
    let err = RecipeLoader::load_dir(dir.path()).unwrap_err();
    assert!(matches!(err, DataError::UnknownRef { target, .. } if target.0 == "Nope"));
}

#[test]
fn string_xref_rtti_and_ref_after_the_first_step_are_rejected() {
    let first = "step \"pattern\" {\n      bytes = \"48\"\n    }\n    ";
    let later = [
        "step \"string_xref\" {\n      text = \"t\"\n    }",
        "step \"rtti_vtable\" {\n      class = \"c\"\n    }",
        "step \"ref\" {\n      entry = \"S\"\n    }",
    ];
    for step in later {
        let dir = TempDir::new().unwrap();
        Fixture::write(dir.path(), "s.hcl", &Fixture::entry("S"));
        Fixture::write(dir.path(), "m.hcl", &Fixture::entry_with_step("M", &format!("{first}{step}")));
        let err = RecipeLoader::load_dir(dir.path()).unwrap_err();
        assert!(matches!(err, DataError::BadFirstStep { recipe: 0, .. }), "{step}");
    }
}

#[test]
fn pattern_may_follow_the_first_step() {
    let dir = TempDir::new().unwrap();
    let steps = "step \"pattern\" {\n      bytes = \"48\"\n    }\n    step \"pattern\" {\n      bytes = \"49\"\n    }";
    Fixture::write(dir.path(), "p.hcl", &Fixture::entry_with_step("P", steps));
    assert!(RecipeLoader::load_dir(dir.path()).is_ok());
}

#[test]
fn unknown_attribute_names_file_entry_and_attribute() {
    let dir = TempDir::new().unwrap();
    Fixture::write(
        dir.path(),
        "typo.hcl",
        &Fixture::entry_with_step("T", "step \"pattern\" {\n      bytes = \"48\"\n      offest = -4\n    }"),
    );
    let err = RecipeLoader::load_dir(dir.path()).unwrap_err();
    match err {
        DataError::Parse { path, entry, message } => {
            assert!(path.ends_with("typo.hcl"));
            assert_eq!(entry.as_deref(), Some("T"));
            assert!(message.contains("offest"));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn an_entry_that_names_the_features_of_a_mod_is_rejected() {
    let dir = TempDir::new().unwrap();
    let tag = "kind = \"function\"\n  features = [\"zoom.tick\"]";
    let text = Fixture::entry("T").replacen("kind = \"function\"", tag, 1);
    Fixture::write(dir.path(), "tagged.hcl", &text);
    let err = RecipeLoader::load_dir(dir.path()).unwrap_err();
    match err {
        DataError::Parse { entry, message, .. } => {
            assert_eq!(entry.as_deref(), Some("T"));
            assert!(message.contains("features"), "{message}");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn negative_pattern_offset_is_read_as_signed() {
    let dir = TempDir::new().unwrap();
    Fixture::write(
        dir.path(),
        "n.hcl",
        &Fixture::entry_with_step("N", "step \"pattern\" {\n      bytes = \"48\"\n      offset = -4\n    }"),
    );
    let set = RecipeLoader::load_dir(dir.path()).unwrap();
    let step = &set.entries()[0].recipes[0].steps[0];
    assert!(matches!(step, amml_data::Step::Pattern { offset: -4, .. }));
}

struct Bad;

impl Bad {
    fn load(file: &str, text: &str) -> DataError {
        let Ok(dir) = TempDir::new() else { panic!("no temporary directory") };
        Fixture::write(dir.path(), file, text);
        match RecipeLoader::load_dir(dir.path()) {
            Err(error) => error,
            Ok(_) => panic!("{file} should not load"),
        }
    }

    fn step(kind: &str, body: &str) -> String {
        Fixture::entry_with_step("E", &format!("step \"{kind}\" {{\n      {body}\n    }}"))
    }

    fn field_entry(id: &str) -> String {
        format!(
            "entry \"{id}\" {{\n  kind = \"field\"\n  recipe {{\n    step \"pattern\" {{\n      bytes = \"48\"\n    }}\n    step \"operand\" {{\n      index = 1\n      kind = \"displacement\"\n    }}\n  }}\n}}\n"
        )
    }
}

#[test]
fn a_pattern_with_a_bad_token_is_rejected_at_load_with_its_file_and_entry() {
    let err = Bad::load("p.hcl", &Bad::step("pattern", "bytes = \"48 8G\""));
    match err {
        DataError::Parse { path, entry, message } => {
            assert!(path.ends_with("p.hcl"));
            assert_eq!(entry.as_deref(), Some("E"));
            assert!(message.contains("8G"));
        }
        other => panic!("{other:?}"),
    }
    assert!(matches!(Bad::load("p.hcl", &Bad::step("pattern", "bytes = \"? ??\"")), DataError::Parse { .. }));
}

#[test]
fn string_xref_text_must_be_non_empty_and_free_of_nul() {
    for text in ["", "a\\u0000b"] {
        let err = Bad::load("s.hcl", &Bad::step("string_xref", &format!("text = \"{text}\"")));
        assert!(matches!(err, DataError::Parse { message, .. } if message.contains("non-empty")), "{text:?}");
    }
}

#[test]
fn string_xref_nth_is_optional() {
    let dir = TempDir::new().unwrap();
    Fixture::write(dir.path(), "a.hcl", &Bad::step("string_xref", "text = \"t\""));
    let set = RecipeLoader::load_dir(dir.path()).unwrap();
    assert!(matches!(&set.entries()[0].recipes[0].steps[0], amml_data::Step::StringXref { nth: None, .. }));
}

#[test]
fn a_range_with_min_above_max_is_rejected() {
    let text = format!(
        "{}  check \"range\" {{\n    min = 16\n    max = 8\n  }}\n}}\n",
        Bad::field_entry("R").trim_end().trim_end_matches('}')
    );
    assert!(matches!(Bad::load("r.hcl", &text), DataError::Parse { message, .. } if message.contains("above max")));
}

#[test]
fn the_hcl_extension_is_matched_without_regard_to_case() {
    let dir = TempDir::new().unwrap();
    Fixture::write(dir.path(), "UPPER.HCL", &Fixture::entry("U"));
    assert_eq!(RecipeLoader::load_dir(dir.path()).unwrap().entries().len(), 1);
}

#[test]
fn extra_labels_on_step_check_and_recipe_blocks_are_rejected() {
    let step = Fixture::entry_with_step("E", "step \"pattern\" \"extra\" {\n      bytes = \"48\"\n    }");
    assert!(matches!(Bad::load("a.hcl", &step), DataError::Parse { message, .. } if message.contains("exactly one")));
    let check = format!(
        "{}  check \"function_start\" \"extra\" {{}}\n}}\n",
        Fixture::entry("E").trim_end().trim_end_matches('}')
    );
    assert!(matches!(Bad::load("b.hcl", &check), DataError::Parse { message, .. } if message.contains("exactly one")));
    let recipe = Fixture::entry("E").replace("recipe {", "recipe \"extra\" {");
    assert!(matches!(Bad::load("c.hcl", &recipe), DataError::Parse { message, .. } if message.contains("no label")));
}

#[test]
fn entry_ids_outside_the_safe_character_set_are_rejected() {
    for id in ["a b", "a\\\"b", "a/b"] {
        let err = Bad::load("i.hcl", &Fixture::entry(id));
        assert!(matches!(err, DataError::Invalid { path: Some(_), .. }), "{id}");
    }
    Bad::load("i.hcl", &Fixture::entry("a${b}"));
}

#[test]
fn a_recipe_that_cannot_end_in_the_entry_kind_is_rejected_with_file_and_entry() {
    let text = Fixture::entry("E").replace("function", "field");
    match Bad::load("t.hcl", &text) {
        DataError::Invalid { entry, message, path } => {
            assert_eq!(entry.0, "E");
            assert!(path.is_some_and(|path| path.ends_with("t.hcl")));
            assert!(message.contains("recipe 0") && message.contains("ends in rva"), "{message}");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_transform_that_cannot_take_the_running_type_is_rejected() {
    let steps = "step \"pattern\" {\n      bytes = \"48\"\n    }\n    step \"operand\" {\n      index = 1\n      kind = \"immediate\"\n    }\n    step \"call\" {\n      nth = 0\n    }";
    let err = Bad::load("t.hcl", &Fixture::entry_with_step("E", steps));
    assert!(
        matches!(err, DataError::Invalid { message, .. } if message.contains("`call` needs an rva but gets a number"))
    );
}

#[test]
fn a_ref_takes_the_type_of_its_target() {
    let dir = TempDir::new().unwrap();
    Fixture::write(dir.path(), "f.hcl", &Bad::field_entry("F"));
    Fixture::write(dir.path(), "g.hcl", &Fixture::entry_with_step("G", "step \"ref\" {\n      entry = \"F\"\n    }"));
    let err = RecipeLoader::load_dir(dir.path()).unwrap_err();
    assert!(matches!(err, DataError::Invalid { entry, .. } if entry.0 == "G"));
}

#[test]
fn cross_file_errors_name_the_file_of_the_entry() {
    let dir = TempDir::new().unwrap();
    Fixture::write(dir.path(), "one.hcl", &Fixture::entry("A"));
    Fixture::write(dir.path(), "two.hcl", &Fixture::entry("A"));
    let err = RecipeLoader::load_dir(dir.path()).unwrap_err();
    assert!(matches!(&err, DataError::DuplicateId { path: Some(path), .. } if path.ends_with("two.hcl")));
    assert!(err.to_string().contains("two.hcl"));

    let dir = TempDir::new().unwrap();
    Fixture::write(
        dir.path(),
        "r.hcl",
        &Fixture::entry_with_step("R", "step \"ref\" {\n      entry = \"Nope\"\n    }"),
    );
    let err = RecipeLoader::load_dir(dir.path()).unwrap_err();
    assert!(err.to_string().contains("r.hcl"));

    let dir = TempDir::new().unwrap();
    Fixture::write(dir.path(), "z.hcl", &Fixture::entry_with_step("Z", "step \"call\" {\n      nth = 0\n    }"));
    let err = RecipeLoader::load_dir(dir.path()).unwrap_err();
    assert!(err.to_string().contains("z.hcl"));
}

#[test]
fn a_parse_error_without_an_entry_does_not_print_none() {
    let err = Bad::load("broken.hcl", "entry {");
    let text = err.to_string();
    assert!(text.contains("broken.hcl"));
    assert!(!text.contains("None") && !text.contains("(entry"), "{text}");
    let with_entry = Bad::load("x.hcl", &Bad::step("pattern", "bytes = \"ZZ\""));
    assert!(with_entry.to_string().contains("(entry E)"));
}

#[test]
fn displacement_to_slot_after_an_rva_step_is_rejected() {
    let steps = "step \"pattern\" {\n      bytes = \"48\"\n    }\n    step \"displacement_to_slot\" {\n    }";
    let err = Bad::load("t.hcl", &Fixture::entry_with_step("E", steps));
    assert!(
        matches!(err, DataError::Invalid { message, .. } if message.contains("`displacement_to_slot` needs a number but gets a rva"))
    );
}

#[test]
fn vtable_slot_and_displacement_to_slot_load_with_their_attributes() {
    let vtable = "step \"pattern\" {\n      bytes = \"48\"\n    }\n    step \"vtable_slot\" {\n      slot = 3\n    }";
    let dir = TempDir::new().unwrap();
    Fixture::write(dir.path(), "v.hcl", &Fixture::entry_with_step("E", vtable));
    let set = RecipeLoader::load_dir(dir.path()).unwrap();
    assert_eq!(set.entries()[0].recipes[0].steps[1], amml_data::Step::VtableSlot { slot: 3 });
    let slot = "step \"pattern\" {\n      bytes = \"48\"\n    }\n    step \"operand\" {\n      index = 0\n      kind = \"displacement\"\n    }\n    step \"displacement_to_slot\" {\n    }";
    let text = Fixture::entry_with_step("S", slot).replace("function", "vslot");
    Fixture::write(dir.path(), "s.hcl", &text);
    let set = RecipeLoader::load_dir(dir.path()).unwrap();
    assert!(set.entries().iter().any(|e| e.recipes[0].steps[2] == amml_data::Step::DisplacementToSlot));
}

#[test]
fn unknown_attributes_on_the_new_steps_are_rejected() {
    let err = Bad::load("t.hcl", &Bad::step("vtable_slot", "slot = 1\n      extra = 2"));
    assert!(matches!(err, DataError::Parse { .. }));
    let steps = "step \"pattern\" {\n      bytes = \"48\"\n    }\n    step \"operand\" {\n      index = 0\n      kind = \"displacement\"\n    }\n    step \"displacement_to_slot\" {\n      slot = 1\n    }";
    let err = Bad::load("t.hcl", &Fixture::entry_with_step("E", steps));
    assert!(matches!(err, DataError::Parse { .. }));
}

#[test]
fn read_pointer_loads_and_needs_an_rva() {
    let steps = "step \"pattern\" {
      bytes = \"48\"
    }
    step \"read_pointer\" {
    }";
    let dir = TempDir::new().unwrap();
    Fixture::write(dir.path(), "r.hcl", &Fixture::entry_with_step("E", steps));
    let set = RecipeLoader::load_dir(dir.path()).unwrap();
    assert_eq!(set.entries()[0].recipes[0].steps[1], amml_data::Step::ReadPointer);
    let after_number = "step \"pattern\" {
      bytes = \"48\"
    }
    step \"operand\" {
      index = 0
      kind = \"displacement\"
    }
    step \"read_pointer\" {
    }";
    let err = Bad::load("t.hcl", &Fixture::entry_with_step("E", after_number));
    assert!(
        matches!(err, DataError::Invalid { message, .. } if message.contains("`read_pointer` needs an rva but gets a number"))
    );
}

#[test]
fn slot_of_loads_with_its_vtable_and_gives_a_slot() {
    let dir = TempDir::new().unwrap();
    let table = "entry \"T\" {
  kind = \"global\"
  recipe {
    step \"pattern\" {
      bytes = \"48\"
    }
  }
}
";
    Fixture::write(dir.path(), "t.hcl", table);
    let steps = "step \"pattern\" {
      bytes = \"48\"
    }
    step \"slot_of\" {
      vtable = \"T\"
    }";
    Fixture::write(dir.path(), "s.hcl", &Fixture::entry_with_step("S", steps).replace("function", "vslot"));
    let set = RecipeLoader::load_dir(dir.path()).unwrap();
    let entry = set.entries().iter().find(|e| e.id.0 == "S").unwrap();
    assert_eq!(entry.recipes[0].steps[1], amml_data::Step::SlotOf { vtable: amml_data::EntryId("T".to_string()) });
    assert_eq!(entry.references().count(), 1);
}

#[test]
fn slot_of_after_a_number_step_is_rejected() {
    let steps = "step \"pattern\" {
      bytes = \"48\"
    }
    step \"operand\" {
      index = 0
      kind = \"displacement\"
    }
    step \"slot_of\" {
      vtable = \"E\"
    }";
    let err = Bad::load("t.hcl", &Fixture::entry_with_step("E", steps).replace("function", "global"));
    assert!(
        matches!(err, DataError::Invalid { message, .. } if message.contains("`slot_of` needs an rva but gets a number"))
    );
}

#[test]
fn slot_of_needs_a_global_vtable_entry_that_exists() {
    let steps = "step \"pattern\" {
      bytes = \"48\"
    }
    step \"slot_of\" {
      vtable = \"F\"
    }";
    let dir = TempDir::new().unwrap();
    Fixture::write(dir.path(), "f.hcl", &Fixture::entry("F"));
    Fixture::write(dir.path(), "s.hcl", &Fixture::entry_with_step("S", steps).replace("function", "vslot"));
    let err = RecipeLoader::load_dir(dir.path()).unwrap_err();
    assert!(matches!(err, DataError::Invalid { message, .. } if message.contains("needs a global entry")));
    let missing = steps.replace("\"F\"", "\"Nope\"");
    let err = Bad::load("t.hcl", &Fixture::entry_with_step("S", &missing).replace("function", "vslot"));
    assert!(matches!(err, DataError::UnknownRef { target, .. } if target.0 == "Nope"));
}

#[test]
fn section_check_loads_on_a_global_and_is_rejected_on_a_vslot() {
    let body = "kind = \"global\"\n  recipe {\n    step \"pattern\" {\n      bytes = \"48\"\n    }\n  }\n  check \"section\" {\n    name = \".rdata\"\n  }\n";
    let dir = TempDir::new().unwrap();
    Fixture::write(dir.path(), "g.hcl", &format!("entry \"G\" {{\n  {body}}}\n"));
    let set = RecipeLoader::load_dir(dir.path()).unwrap();
    assert_eq!(set.entries()[0].checks, vec![amml_data::Check::Section { name: ".rdata".to_string() }]);
    let steps = "step \"pattern\" {\n      bytes = \"48\"\n    }\n    step \"slot_of\" {\n      vtable = \"G\"\n    }";
    let vslot = format!(
        "entry \"V\" {{\n  kind = \"vslot\"\n  recipe {{\n    {steps}\n  }}\n  check \"section\" {{\n    name = \".rdata\"\n  }}\n}}\n"
    );
    Fixture::write(dir.path(), "v.hcl", &vslot);
    let err = RecipeLoader::load_dir(dir.path()).unwrap_err();
    assert!(
        matches!(err, DataError::Invalid { message, .. } if message.contains("only valid on function and global entries"))
    );
}

#[test]
fn pattern_any_loads_with_its_alternatives_and_offset() {
    let dir = TempDir::new().unwrap();
    let step = "step \"pattern_any\" {\n      patterns = [\"48 ??\", \"49\"]\n      offset = -2\n    }";
    Fixture::write(dir.path(), "a.hcl", &Fixture::entry_with_step("A", step));
    assert!(RecipeLoader::load_dir(dir.path()).is_ok());
}

#[test]
fn pattern_any_needs_two_alternatives_and_each_must_have_a_fixed_byte() {
    let one = Bad::load("a.hcl", &Bad::step("pattern_any", "patterns = [\"48\"]"));
    assert!(matches!(one, DataError::Parse { message, .. } if message.contains("at least 2")));
    let blind = Bad::load("a.hcl", &Bad::step("pattern_any", "patterns = [\"48\", \"? ??\"]"));
    assert!(matches!(blind, DataError::Parse { .. }));
}

#[test]
fn pattern_any_may_follow_the_first_step() {
    let dir = TempDir::new().unwrap();
    let steps = "step \"pattern\" {\n      bytes = \"48\"\n    }\n    step \"pattern_any\" {\n      patterns = [\"49\", \"4A\"]\n    }";
    Fixture::write(dir.path(), "p.hcl", &Fixture::entry_with_step("P", steps));
    assert!(RecipeLoader::load_dir(dir.path()).is_ok());
}

#[test]
fn function_start_of_follows_pattern_and_rejects_a_number() {
    let dir = TempDir::new().unwrap();
    let steps = "step \"pattern\" {\n      bytes = \"48\"\n    }\n    step \"function_start_of\" {\n    }";
    Fixture::write(dir.path(), "p.hcl", &Fixture::entry_with_step("P", steps));
    assert!(RecipeLoader::load_dir(dir.path()).is_ok());

    let steps = "step \"pattern\" {\n      bytes = \"48\"\n    }\n    step \"operand\" {\n      index = 1\n      kind = \"immediate\"\n    }\n    step \"function_start_of\" {\n    }";
    let err = Bad::load("t.hcl", &Fixture::entry_with_step("E", steps));
    assert!(
        matches!(err, DataError::Invalid { message, .. } if message.contains("`function_start_of` needs an rva but gets a number"))
    );
}

#[test]
fn in_vtable_loads_on_a_function_and_makes_its_global_a_dependency() {
    let dir = TempDir::new().unwrap();
    let global = "entry \"VT\" {\n  kind = \"global\"\n  recipe {\n    step \"rtti_vtable\" {\n      class = \"c\"\n    }\n  }\n}\n";
    let function = "entry \"F\" {\n  kind = \"function\"\n  recipe {\n    step \"pattern\" {\n      bytes = \"48\"\n    }\n  }\n  check \"in_vtable\" {\n    vtable = \"VT\"\n    min_slot = 20\n    max_slot = 26\n  }\n}\n";
    Fixture::write(dir.path(), "a.hcl", &format!("{function}{global}"));
    let set = RecipeLoader::load_dir(dir.path()).unwrap();
    let order = amml_data::DependencyOrder::of(&set).unwrap();
    let position = |name: &str| order.iter().position(|id| id.0 == name).unwrap();
    assert!(position("VT") < position("F"));
}

#[test]
fn in_vtable_rejects_a_vtable_that_is_not_a_global_and_a_reversed_range() {
    let function = |extra: &str| {
        format!(
            "entry \"F\" {{\n  kind = \"function\"\n  recipe {{\n    step \"pattern\" {{\n      bytes = \"48\"\n    }}\n  }}\n  check \"in_vtable\" {{\n    vtable = \"G\"\n    {extra}\n  }}\n}}\n"
        )
    };
    let other = Fixture::entry("G");
    let err = Bad::load("a.hcl", &format!("{other}{}", function("min_slot = 1\n    max_slot = 2")));
    assert!(matches!(err, DataError::Invalid { message, .. } if message.contains("needs a global entry")));
    let err = Bad::load("a.hcl", &function("min_slot = 3\n    max_slot = 2"));
    assert!(matches!(err, DataError::Parse { message, .. } if message.contains("above max_slot")));
}

#[test]
fn checks_are_rejected_on_the_kinds_they_cannot_judge() {
    let vslot = |check: &str| {
        format!(
            "entry \"V\" {{\n  kind = \"vslot\"\n  recipe {{\n    step \"pattern\" {{\n      bytes = \"48\"\n    }}\n    step \"operand\" {{\n      index = 1\n      kind = \"displacement\"\n    }}\n    step \"displacement_to_slot\" {{\n    }}\n  }}\n  {check}\n}}\n"
        )
    };
    let err = Bad::load("v.hcl", &vslot("check \"function_start\" {}"));
    assert!(
        matches!(err, DataError::Invalid { message, .. } if message.contains("only valid on function entries, not vslot"))
    );
    let err = Bad::load("v.hcl", &vslot("check \"in_vtable_of\" {\n class = \"c\"\n }"));
    assert!(
        matches!(err, DataError::Invalid { message, .. } if message.contains("`in_vtable_of` is only valid on function entries, not vslot"))
    );
    let function = Fixture::entry_with_step("F", "step \"pattern\" {\n      bytes = \"48\"\n    }")
        .replace("\n}\n", "\n  check \"range\" {\n    min = 0\n    max = 1\n  }\n}\n");
    let err = Bad::load("f.hcl", &function);
    assert!(
        matches!(err, DataError::Invalid { message, .. } if message.contains("only valid on vslot, field and global entries, not function"))
    );
}

#[test]
fn every_recipe_in_the_recipes_directory_still_loads() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../recipes");
    assert!(RecipeLoader::load_dir(&dir).is_ok());
}

struct LiteralOkCases;

impl LiteralOkCases {
    fn load(step: &str) -> Result<amml_data::RecipeSet, DataError> {
        let Ok(dir) = TempDir::new() else { panic!("no temporary directory") };
        Fixture::write(dir.path(), "l.hcl", &Fixture::entry_with_step("L", step));
        RecipeLoader::load_dir(dir.path())
    }

    fn pattern(literal_ok: &str) -> String {
        format!(
            "step \"pattern\" {{
      bytes = \"48 83 EC ? 90\"
      literal_ok = {literal_ok}
    }}"
        )
    }

    fn message(literal_ok: &str) -> String {
        match Self::load(&Self::pattern(literal_ok)) {
            Err(DataError::Parse { message, .. }) => message,
            other => panic!("{literal_ok} should not load: {other:?}"),
        }
    }
}

#[test]
fn literal_ok_is_read_from_a_pattern() {
    let set = LiteralOkCases::load(&LiteralOkCases::pattern("[\"2: frame size is fixed\", \"4: pad\"]")).unwrap();
    let amml_data::Step::Pattern { literal_ok, offset, .. } = &set.entries()[0].recipes[0].steps[0] else {
        panic!("not a pattern")
    };
    assert_eq!(*offset, 0);
    let kept: Vec<_> = literal_ok.iter().map(|item| (item.index, item.reason.as_str())).collect();
    assert_eq!(kept, vec![(2, "frame size is fixed"), (4, "pad")]);
}

#[test]
fn a_pattern_without_literal_ok_has_an_empty_list() {
    let set = LiteralOkCases::load(
        "step \"pattern\" {
      bytes = \"48 83\"
    }",
    )
    .unwrap();
    assert!(
        matches!(&set.entries()[0].recipes[0].steps[0], amml_data::Step::Pattern { literal_ok, .. } if literal_ok.is_empty())
    );
}

#[test]
fn a_malformed_literal_ok_entry_is_a_load_error() {
    for bad in ["[\"no colon\"]", "[\"x: reason\"]", "[\"2:   \"]", "[\"-1: reason\"]", "[2]", "\"2: reason\""] {
        let message = LiteralOkCases::message(bad);
        assert!(message.contains("literal_ok"), "{bad}: {message}");
    }
}

#[test]
fn a_literal_ok_index_on_a_wildcard_or_past_the_end_is_a_load_error() {
    assert!(LiteralOkCases::message("[\"3: reason\"]").contains("wildcard"));
    assert!(LiteralOkCases::message("[\"5: reason\"]").contains("past the end"));
}

#[test]
fn literal_ok_on_pattern_any_names_the_alternative() {
    let any = |literal_ok: &str| {
        format!(
            "step \"pattern_any\" {{
      patterns = [\"48 83 EC 28\", \"48 83 ? 28 90\"]
      literal_ok = {literal_ok}
    }}"
        )
    };
    let Ok(set) = LiteralOkCases::load(&any("[\"0.0: a\", \"1.4: only the longer\"]")) else { panic!("should load") };
    let amml_data::Step::PatternAny { literal_ok, .. } = &set.entries()[0].recipes[0].steps[0] else { panic!() };
    assert_eq!(
        literal_ok.iter().map(|item| (item.alternative, item.index)).collect::<Vec<_>>(),
        vec![(Some(0), 0), (Some(1), 4)]
    );
    // no alternative, a missing alternative, a wildcard in that alternative, past the end of that alternative, a repeat
    for bad in ["[\"0: a\"]", "[\"2.0: a\"]", "[\"1.2: a\"]", "[\"0.4: a\"]", "[\"0.1: a\", \"0.1: b\"]"] {
        assert!(matches!(LiteralOkCases::load(&any(bad)), Err(DataError::Parse { .. })), "{bad}");
    }
}

#[test]
fn literal_ok_on_pattern_refuses_the_alternative_form_and_a_repeated_index() {
    assert!(LiteralOkCases::message("[\"0.1: a\"]").contains("literal_ok"));
    assert!(LiteralOkCases::message("[\"1: a\", \"1: b\"]").contains("twice"));
}

#[test]
fn hooked_marks_a_function_entry_and_defaults_to_false() {
    let dir = TempDir::new().unwrap();
    let hooked = "entry \"H\" {\n  kind = \"function\"\n  hooked = true\n  recipe {\n    step \"pattern\" {\n      bytes = \"48 89\"\n    }\n  }\n}\n";
    Fixture::write(dir.path(), "a.hcl", hooked);
    Fixture::write(dir.path(), "b.hcl", &Fixture::entry("P"));
    let set = RecipeLoader::load_dir(dir.path()).unwrap();
    let flags: Vec<_> = set.entries().iter().map(|e| (e.id.0.as_str(), e.hooked)).collect();
    assert_eq!(flags, vec![("H", true), ("P", false)]);
}

#[test]
fn hooked_on_a_non_function_entry_or_not_a_bool_is_a_load_error() {
    let dir = TempDir::new().unwrap();
    let global = "entry \"G\" {\n  kind = \"global\"\n  hooked = true\n  recipe {\n    step \"pattern\" {\n      bytes = \"48 89\"\n    }\n  }\n}\n";
    Fixture::write(dir.path(), "g.hcl", global);
    let err = RecipeLoader::load_dir(dir.path()).unwrap_err();
    assert!(
        matches!(err, DataError::Parse { message, .. } if message.contains("hooked") && message.contains("global"))
    );

    let dir = TempDir::new().unwrap();
    let text = "entry \"T\" {\n  kind = \"function\"\n  hooked = \"yes\"\n  recipe {\n    step \"pattern\" {\n      bytes = \"48 89\"\n    }\n  }\n}\n";
    Fixture::write(dir.path(), "t.hcl", text);
    let err = RecipeLoader::load_dir(dir.path()).unwrap_err();
    assert!(matches!(err, DataError::Parse { message, .. } if message.contains("true or false")));
}

#[test]
fn leaf_start_check_loads_on_a_function_and_is_rejected_on_a_vslot() {
    let dir = TempDir::new().unwrap();
    let function = Fixture::entry("F").replace("\n}\n", "\n  check \"leaf_start\" {}\n}\n");
    Fixture::write(dir.path(), "f.hcl", &function);
    let set = RecipeLoader::load_dir(dir.path()).unwrap();
    assert_eq!(set.entries()[0].checks, vec![amml_data::Check::LeafStart]);
    let vslot = "entry \"V\" {\n  kind = \"vslot\"\n  recipe {\n    step \"pattern\" {\n      bytes = \"48\"\n    }\n    step \"operand\" {\n      index = 1\n      kind = \"displacement\"\n    }\n    step \"displacement_to_slot\" {\n    }\n  }\n  check \"leaf_start\" {}\n}\n";
    let err = Bad::load("v.hcl", vslot);
    assert!(
        matches!(err, DataError::Invalid { message, .. } if message.contains("`leaf_start` is only valid on function entries, not vslot"))
    );
}

#[test]
fn alternative_step_loads_only_right_after_a_pattern_any() {
    let any = "step \"pattern_any\" {\n      patterns = [\"48\", \"49\"]\n    }";
    let dir = TempDir::new().unwrap();
    let good =
        Fixture::entry_with_step("F", &format!("{any}\n    step \"alternative\" {{}}")).replace("function", "field");
    Fixture::write(dir.path(), "f.hcl", &good);
    let set = RecipeLoader::load_dir(dir.path()).unwrap();
    assert_eq!(set.entries()[0].recipes[0].steps.last(), Some(&amml_data::Step::Alternative));
    let pattern = "step \"pattern\" {\n      bytes = \"48\"\n    }";
    let bad = Fixture::entry_with_step("F", &format!("{pattern}\n    step \"alternative\" {{}}"))
        .replace("function", "field");
    let err = Bad::load("b.hcl", &bad);
    assert!(
        matches!(err, DataError::Parse { message, .. } if message.contains("`alternative` must come right after a `pattern_any`"))
    );
}

#[test]
fn entry_is_check_needs_a_field_entry() {
    let layout = "entry \"L\" {\n  kind = \"field\"\n  recipe {\n    step \"pattern\" {\n      bytes = \"48\"\n    }\n    step \"operand\" {\n      index = 1\n      kind = \"displacement\"\n    }\n  }\n}\n";
    let guarded =
        Fixture::entry("F").replace("\n}\n", "\n  check \"entry_is\" {\n    entry = \"L\"\n    value = 1\n  }\n}\n");
    let dir = TempDir::new().unwrap();
    Fixture::write(dir.path(), "a.hcl", &format!("{layout}{guarded}"));
    let set = RecipeLoader::load_dir(dir.path()).unwrap();
    let f = set.entries().iter().find(|entry| entry.id.0 == "F").unwrap();
    assert_eq!(f.checks, vec![amml_data::Check::EntryIs { entry: amml_data::EntryId("L".to_string()), value: 1 }]);
    let other = Fixture::entry("G");
    let wrong = guarded.replace("\"L\"", "\"G\"");
    let err = Bad::load("b.hcl", &format!("{other}{wrong}"));
    assert!(
        matches!(err, DataError::Invalid { message, .. } if message.contains("needs a field entry but `G` is a function"))
    );
}

#[test]
fn member_displacement_loads_with_its_attributes_and_needs_an_rva() {
    let steps = "step \"pattern\" {\n      bytes = \"48\"\n    }\n    step \"member_displacement\" {\n      index = 1\n      local_at = -48\n    }";
    let dir = TempDir::new().unwrap();
    Fixture::write(dir.path(), "m.hcl", &Fixture::entry_with_step("M", steps).replace("function", "field"));
    let set = RecipeLoader::load_dir(dir.path()).unwrap();
    assert_eq!(set.entries()[0].recipes[0].steps[1], amml_data::Step::MemberDisplacement { index: 1, local_at: -48 });
    let after_number = "step \"pattern\" {\n      bytes = \"48\"\n    }\n    step \"operand\" {\n      index = 1\n      kind = \"displacement\"\n    }\n    step \"member_displacement\" {\n      index = 1\n      local_at = 0\n    }";
    let err = Bad::load("t.hcl", &Fixture::entry_with_step("E", after_number));
    assert!(
        matches!(err, DataError::Invalid { message, .. } if message.contains("`member_displacement` needs an rva"))
    );
}

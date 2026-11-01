use amml_data::{DataError, DependencyOrder, Entry, EntryId, EntryKind, Recipe, RecipeSet, Step};

struct Fixture;

impl Fixture {
    fn ids<const N: usize>(names: [&str; N]) -> Vec<EntryId> {
        names.iter().map(|name| EntryId(name.to_string())).collect()
    }

    fn entry(id: &str, refs: &[&str]) -> Entry {
        let mut recipes: Vec<Recipe> = refs
            .iter()
            .map(|target| Recipe { steps: vec![Step::Ref { entry: EntryId(target.to_string()) }] })
            .collect();
        if recipes.is_empty() {
            recipes.push(Recipe {
                steps: vec![Step::Pattern { bytes: "48".to_string(), offset: 0, literal_ok: Vec::new() }],
            });
        }
        Entry {
            id: EntryId(id.to_string()),
            kind: EntryKind::Function,
            msvc_name: None,
            recipes,
            checks: Vec::new(),
            notes: String::new(),
            hooked: false,
        }
    }
}

#[test]
fn dependencies_come_before_dependents_and_ties_sort_by_id() {
    let set =
        RecipeSet::new(vec![Fixture::entry("C", &["A"]), Fixture::entry("B", &[]), Fixture::entry("A", &[])]).unwrap();
    assert_eq!(DependencyOrder::of(&set).unwrap(), Fixture::ids(["A", "B", "C"]));
}

#[test]
fn cycle_is_reported_with_its_members() {
    let set = RecipeSet::new(vec![Fixture::entry("A", &["B"]), Fixture::entry("B", &["A"])]).unwrap();
    assert!(matches!(DependencyOrder::of(&set), Err(DataError::Cycle(m)) if m == Fixture::ids(["A", "B"])));
}

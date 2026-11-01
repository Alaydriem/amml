use amml_data::{Check, DataError, Entry, EntryId, EntryKind, Recipe, RecipeSet, Step};
use amml_derive::EntryExplainer;
use amml_derive::testing::ImageFixture;

const BASE: u64 = 0x1_4000_0000;

fn id(name: &str) -> EntryId {
    EntryId(name.to_string())
}

fn entry(name: &str, steps: Vec<Step>) -> Entry {
    Entry {
        id: id(name),
        kind: EntryKind::Function,
        msvc_name: None,
        recipes: vec![Recipe { steps }],
        checks: Vec::new(),
        notes: String::new(),
        hooked: false,
    }
}

fn pattern(bytes: &str) -> Step {
    Step::Pattern { bytes: bytes.to_string(), offset: 0, literal_ok: Vec::new() }
}

#[test]
fn explain_reports_match_count_of_failing_step() {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&[0xAA, 0xBB, 0xCC, 0xC3]);
    fx.put_text(&[0xAA, 0xBB, 0xCC, 0xC3]);
    let image = fx.image();
    let set = RecipeSet::new(vec![entry("E", vec![pattern("AA BB CC")])]).unwrap();
    let traces = EntryExplainer::new(&image, &set).explain(&id("E")).unwrap();
    assert_eq!(traces.len(), 1);
    let step = &traces[0].steps[0];
    assert_eq!(step.matches, Some(2));
    assert_eq!(step.cursor, None);
    assert!(step.error.as_deref().is_some_and(|text| text.contains("2 matches")), "{step:?}");
    assert!(traces[0].result.is_err());
    let lines = traces[0].lines();
    assert!(lines[0].starts_with("recipe 0 step 0 pattern \"AA BB CC\"  cursor=-  matches=2  "), "{}", lines[0]);
}

#[test]
fn explain_counts_matches_of_a_step_that_succeeds() {
    let mut fx = ImageFixture::new(BASE);
    let start = fx.put_text(&[0xAA, 0xBB, 0xCC, 0xC3]);
    let image = fx.image();
    let set = RecipeSet::new(vec![entry("E", vec![pattern("AA BB CC")])]).unwrap();
    let traces = EntryExplainer::new(&image, &set).explain(&id("E")).unwrap();
    assert_eq!(traces[0].steps[0].matches, Some(1));
    assert_eq!(traces[0].steps[0].cursor, Some(u64::from(start)));
    assert_eq!(traces[0].result, Ok(u64::from(start)));
}

#[test]
fn explain_resolves_ref_dependencies() {
    let mut fx = ImageFixture::new(BASE);
    let start = fx.put_text(&[0xAA, 0xBB, 0xCC, 0xC3]);
    fx.put_function(start, start + 4);
    let image = fx.image();
    let mut a = entry("A", vec![pattern("AA BB CC")]);
    a.checks = vec![Check::FunctionStart];
    let set = RecipeSet::new(vec![a, entry("B", vec![Step::Ref { entry: id("A") }, Step::Add { value: 1 }])]).unwrap();
    let traces = EntryExplainer::new(&image, &set).explain(&id("B")).unwrap();
    assert_eq!(traces[0].steps[0].cursor, Some(u64::from(start)));
    assert_eq!(traces[0].steps[1].cursor, Some(u64::from(start) + 1));
    assert_eq!(traces[0].result, Ok(u64::from(start) + 1));
}

#[test]
fn explain_shows_a_dependency_that_the_deriver_refused() {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&[0xAA, 0xBB, 0xCC, 0xC3]);
    let image = fx.image();
    // the pattern matches, but no function starts there, so the check refuses the entry
    let mut a = entry("A", vec![pattern("AA BB CC")]);
    a.checks = vec![Check::FunctionStart];
    let set = RecipeSet::new(vec![a, entry("B", vec![Step::Ref { entry: id("A") }])]).unwrap();
    let traces = EntryExplainer::new(&image, &set).explain(&id("B")).unwrap();
    let error = traces[0].steps[0].error.as_deref().unwrap();
    assert!(error.contains("dependency `A` failed"), "{error}");
    assert_eq!(traces[0].steps[0].cursor, None);
}

#[test]
fn explain_counts_matches_inside_the_function_of_the_cursor() {
    let mut fx = ImageFixture::new(BASE);
    let first = fx.put_text(&[0x11, 0x22, 0xAA, 0xBB, 0xC3]);
    fx.put_function(first, first + 5);
    let other = fx.put_text(&[0xAA, 0xBB, 0xC3]);
    fx.put_function(other, other + 3);
    let image = fx.image();
    let set = RecipeSet::new(vec![entry("E", vec![pattern("11 22"), pattern("AA BB")])]).unwrap();
    let traces = EntryExplainer::new(&image, &set).explain(&id("E")).unwrap();
    assert_eq!(traces[0].steps[1].matches, Some(1));
    assert_eq!(traces[0].result, Ok(u64::from(first) + 2));
}

fn any(patterns: &[&str]) -> Step {
    Step::PatternAny {
        patterns: patterns.iter().map(|text| text.to_string()).collect(),
        offset: 0,
        literal_ok: Vec::new(),
    }
}

#[test]
fn explain_pattern_any_counts_one_position_hit_by_two_alternatives_once() {
    let mut fx = ImageFixture::new(BASE);
    let start = fx.put_text(&[0xAA, 0xBB, 0xC3]);
    let image = fx.image();
    let set = RecipeSet::new(vec![entry("E", vec![any(&["AA BB", "AA ??"])])]).unwrap();
    let traces = EntryExplainer::new(&image, &set).explain(&id("E")).unwrap();
    assert_eq!(traces[0].steps[0].matches, Some(1));
    assert_eq!(traces[0].result, Ok(u64::from(start)));
}

#[test]
fn explain_pattern_any_shows_two_positions_and_none() {
    let mut fx = ImageFixture::new(BASE);
    fx.put_text(&[0xAA, 0xBB, 0xC3]);
    fx.put_text(&[0xAA, 0xCC, 0xC3]);
    let image = fx.image();
    let set = RecipeSet::new(vec![
        entry("Two", vec![any(&["AA BB", "AA CC"])]),
        entry("None", vec![any(&["DD EE", "DD FF"])]),
    ])
    .unwrap();
    let explainer = EntryExplainer::new(&image, &set);
    let two = explainer.explain(&id("Two")).unwrap();
    assert_eq!(two[0].steps[0].matches, Some(2));
    assert!(two[0].steps[0].error.is_some());
    let none = explainer.explain(&id("None")).unwrap();
    assert_eq!(none[0].steps[0].matches, Some(0));
    assert!(none[0].steps[0].error.is_some());
}

#[test]
fn explain_unknown_id_is_an_error() {
    let fx = ImageFixture::new(BASE);
    let image = fx.image();
    let set = RecipeSet::new(vec![entry("E", vec![pattern("AA BB CC")])]).unwrap();
    let error = EntryExplainer::new(&image, &set).explain(&id("Nope")).unwrap_err();
    assert!(matches!(error, DataError::UnknownEntry(_)), "{error}");
}

use amml_i18n_tool::plural::PluralExpression;

const ENGLISH: &str = "n != 1";
const RUSSIAN: &str = "(n%10==1 && n%100!=11 ? 0 : n%10>=2 && n%10<=4 && (n%100<10 || n%100>=20) ? 1 : 2)";
const POLISH: &str = "(n==1 ? 0 : n%10>=2 && n%10<=4 && (n%100<10 || n%100>=20) ? 1 : 2)";

fn evaluate(source: &str, n: u64) -> u64 {
    PluralExpression::evaluate(source, n).unwrap_or_else(|error| panic!("{source}: {error}"))
}

#[test]
fn english_selects_form_0_for_exactly_one() {
    assert_eq!(evaluate(ENGLISH, 1), 0);
}

#[test]
fn english_selects_form_1_for_zero_and_for_many() {
    assert_eq!(evaluate(ENGLISH, 0), 1);
    assert_eq!(evaluate(ENGLISH, 2), 1);
}

#[test]
fn russian_selects_the_form_for_each_residue() {
    for (n, expected) in [(1, 0), (21, 0), (2, 1), (22, 1), (5, 2), (11, 2), (0, 2)] {
        assert_eq!(evaluate(RUSSIAN, n), expected, "n={n}");
    }
}

#[test]
fn polish_selects_the_form_for_each_residue() {
    for (n, expected) in [(1, 0), (2, 1), (22, 1), (5, 2), (0, 2)] {
        assert_eq!(evaluate(POLISH, n), expected, "n={n}");
    }
}

#[test]
fn a_constant_expression_always_selects_the_only_form() {
    assert_eq!(evaluate("0", 7), 0);
}

#[test]
fn a_trailing_semicolon_from_the_header_is_tolerated() {
    assert_eq!(evaluate("n != 1;", 1), 0);
}

#[test]
fn modulo_by_zero_is_an_error_not_a_crash() {
    assert!(PluralExpression::evaluate("n % 0", 3).is_err());
}

#[test]
fn rejects_anything_outside_the_grammar() {
    for source in ["process.exit(1)", "require('fs')", "n + 1", "globalThis", "n ? 0 : (1", ""] {
        assert!(PluralExpression::evaluate(source, 1).is_err(), "accepted {source:?}");
    }
}

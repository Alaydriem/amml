use amml_i18n::localize::Localizer;
use amml_i18n::text::{Args, PlaceholderFormat};

#[test]
fn every_occurrence_of_a_placeholder_is_replaced() {
    let args = Args::new().int("n", 4);
    assert_eq!(PlaceholderFormat::apply("{n} of {n}", &args, &Localizer::english()), "4 of 4");
}

#[test]
fn a_float_keeps_its_declared_decimals() {
    let args = Args::new().float("scale", 3.0, 2);
    assert_eq!(PlaceholderFormat::apply("{scale}x", &args, &Localizer::english()), "3.00x");
}

#[test]
fn an_unclosed_brace_is_copied_as_written() {
    let args = Args::new().int("n", 1);
    assert_eq!(PlaceholderFormat::apply("{n} {open", &args, &Localizer::english()), "1 {open");
}

#[test]
fn text_without_placeholders_is_unchanged() {
    assert_eq!(PlaceholderFormat::apply("Plain, é", &Args::new(), &Localizer::english()), "Plain, é");
}

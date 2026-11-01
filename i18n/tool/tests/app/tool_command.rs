use std::path::PathBuf;

use amml_i18n_tool::app::ToolCommand;

fn parse(args: &[&str]) -> Result<ToolCommand, String> {
    ToolCommand::parse(args.iter().map(|arg| (*arg).to_owned())).map_err(|error| error.to_string())
}

#[test]
fn each_subcommand_takes_its_two_paths() {
    assert_eq!(
        parse(&["build", "po", "out"]),
        Ok(ToolCommand::Build { po_dir: PathBuf::from("po"), out_dir: PathBuf::from("out") })
    );
    assert_eq!(
        parse(&["pseudo", "a.pot", "en_XA.json"]),
        Ok(ToolCommand::Pseudo { pot: PathBuf::from("a.pot"), out: PathBuf::from("en_XA.json") })
    );
    assert_eq!(
        parse(&["report", "a.pot", "po"]),
        Ok(ToolCommand::Report { pot: PathBuf::from("a.pot"), po_dir: PathBuf::from("po") })
    );
    assert_eq!(
        parse(&["check-pot", "a.pot", "fresh.pot"]),
        Ok(ToolCommand::CheckPot { pot: PathBuf::from("a.pot"), fresh: PathBuf::from("fresh.pot") })
    );
}

#[test]
fn an_unknown_subcommand_or_a_missing_path_is_a_usage_error() {
    for args in [&["translate", "a", "b"][..], &["build", "po"][..], &[][..], &["pseudo", "a", "b", "c"][..]] {
        assert!(parse(args).is_err_and(|error| error.contains("usage")), "{args:?}");
    }
}

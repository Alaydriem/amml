use amml_launcher_core::{LaunchSettings, LauncherError};

#[test]
fn quoted_arguments_stay_together() {
    assert_eq!(LaunchSettings::args("-a \"b c\" d").unwrap(), vec!["-a", "b c", "d"]);
    assert_eq!(LaunchSettings::args("  x   y\t z ").unwrap(), vec!["x", "y", "z"]);
    assert_eq!(LaunchSettings::args("--name=\"a b\"").unwrap(), vec!["--name=a b"]);
    assert_eq!(LaunchSettings::args("a \"\" b").unwrap(), vec!["a", "", "b"]);
    assert!(LaunchSettings::args("").unwrap().is_empty());
    assert!(LaunchSettings::args("   ").unwrap().is_empty());
}

#[test]
fn an_unclosed_quote_is_named() {
    match LaunchSettings::args("-a \"b c") {
        Err(LauncherError::BadLaunchSetting(text)) => assert!(text.contains("-a \"b c"), "{text}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn env_pairs_are_one_per_line_and_split_at_the_first_equals() {
    assert_eq!(
        LaunchSettings::env("A=1\r\n\nB=x=y").unwrap(),
        vec![("A".to_string(), "1".to_string()), ("B".to_string(), "x=y".to_string())]
    );
    assert_eq!(LaunchSettings::env("E=").unwrap(), vec![("E".to_string(), String::new())]);
    assert!(LaunchSettings::env("").unwrap().is_empty());
}

#[test]
fn a_line_without_equals_is_refused() {
    assert!(
        matches!(LaunchSettings::env("A=1\nNOPE"), Err(LauncherError::BadLaunchSetting(text)) if text.contains("NOPE"))
    );
    assert!(matches!(LaunchSettings::env("=value"), Err(LauncherError::BadLaunchSetting(_))));
}

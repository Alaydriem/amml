use std::process::Command;

use amml_ui_gen::app::Generator;
use amml_ui_gen::model::{OwnershipFile, TokenFile};

// Tokens and selectors whose generated lines sweep across rustfmt's 120-column limit:
// numbered names (version sort), long alias lines, and `ALL` arrays of long and short variants.
fn sweep() -> (TokenFile, OwnershipFile) {
    let mut tokens = Vec::new();
    let mut owners = Vec::new();
    let mut add = |name: String, value: String| {
        tokens.push(format!(r#"{{"name":"{name}","type":"color","value":"{value}"}}"#));
        owners.push(format!(r#""{name}":"amml-ui""#));
    };
    for at in 1..=15 {
        add(format!("--color-k{at}"), "#102030".to_owned());
    }
    for length in 20..=45 {
        let long = format!("--color-{}", "a".repeat(length));
        add(format!("--color-alias-{length}"), format!("var({long})"));
        add(long, "#405060".to_owned());
    }
    let mut selectors = Vec::new();
    for count in 1..=12 {
        let variants: Vec<String> =
            (1..=count).map(|at| format!(r#"["Variant{at}","--color-k{}"]"#, at.min(15))).collect();
        selectors
            .push(format!(r#"{{"crate":"amml-ui","name":"Long{count}Color","variants":[{}]}}"#, variants.join(",")));
    }
    for count in 1..=40 {
        let variants: Vec<String> = (1..=count).map(|at| format!(r#"["A{at}","--color-k1"]"#)).collect();
        selectors
            .push(format!(r#"{{"crate":"amml-ui","name":"Short{count}Color","variants":[{}]}}"#, variants.join(",")));
    }
    let tokens = format!(r#"{{"tokens":[{}]}}"#, tokens.join(","));
    let owners = format!(r#"{{"owners":{{{}}},"selectors":[{}]}}"#, owners.join(","), selectors.join(","));
    (
        serde_json::from_str(&tokens).unwrap_or_else(|error| panic!("sweep tokens: {error}")),
        serde_json::from_str(&owners).unwrap_or_else(|error| panic!("sweep ownership: {error}")),
    )
}

#[test]
fn generated_output_is_rustfmt_stable_across_the_width_limit() {
    let (tokens, owners) = sweep();
    let (kit, _) = Generator::build(&tokens, &owners).unwrap();
    let dir = tempfile::tempdir().unwrap();
    kit.write_to(dir.path()).unwrap();
    let output = Command::new("rustfmt")
        .args(["--check", "--edition", "2024", "--config", "max_width=120,use_small_heuristics=Max"])
        .arg(dir.path().join("mod.rs"))
        .output()
        .unwrap_or_else(|error| panic!("rustfmt did not start: {error}"));
    assert!(
        output.status.success(),
        "rustfmt would change the generated files:\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

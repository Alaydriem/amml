use std::collections::HashMap;
use std::path::PathBuf;

use amml_derive::cli::{
    BuildIdArgs, CallersArgs, DisasmArgs, InspectArgs, InspectCommand, InspectOp, ScanArgs, VtableArgs, XrefsArgs,
};
use amml_derive::testing::ImageFixture;
use amml_derive::{Address, CliError, Image, ImageLoader};

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pe/small.dll")
}

fn load() -> Image {
    match ImageLoader::from_path(&fixture()) {
        Ok(image) => image,
        Err(error) => panic!("{error}"),
    }
}

// the hex text of a byte run that occurs exactly twice in the code of the fixture
fn twice_repeated_run() -> String {
    let image = load();
    for len in (3..=16).rev() {
        for section in image.sections().iter().filter(|s| s.executable) {
            let mut counts: HashMap<&[u8], usize> = HashMap::new();
            for window in section.data.windows(len) {
                *counts.entry(window).or_default() += 1;
            }
            if let Some((run, _)) = counts.iter().find(|(_, count)| **count == 2) {
                return run.iter().map(|byte| format!("{byte:02x}")).collect::<Vec<_>>().join(" ");
            }
        }
    }
    panic!("the fixture has no byte run that occurs twice");
}

#[test]
fn inspect_scan_counts_matches() {
    let op = InspectOp::Scan(ScanArgs { pattern: twice_repeated_run(), limit: 20 });
    let text = InspectCommand::run(&InspectArgs::new(fixture(), op)).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines[0], "matches: 2");
    assert_eq!(lines.len(), 3);
}

#[test]
fn inspect_scan_limit_caps_the_listed_lines() {
    let op = InspectOp::Scan(ScanArgs { pattern: twice_repeated_run(), limit: 1 });
    let text = InspectCommand::run(&InspectArgs::new(fixture(), op)).unwrap();
    assert_eq!(text.lines().count(), 2);
}

#[test]
fn inspect_build_id_prints_the_file_stem() {
    let image = load();
    let text = InspectCommand::run(&InspectArgs::new(fixture(), InspectOp::BuildId(BuildIdArgs {}))).unwrap();
    assert_eq!(text, image.build().file_stem());
}

#[test]
fn inspect_of_a_missing_exe_is_invalid_input() {
    let op = InspectOp::BuildId(BuildIdArgs {});
    let error = InspectCommand::run(&InspectArgs::new(PathBuf::from("missing.exe"), op)).unwrap_err();
    assert!(matches!(error, CliError::Data(_) | CliError::Derive(_)));
    assert_eq!(error.exit_code(), 2);
}

#[test]
fn inspect_without_exe_is_invalid_input() {
    let args = InspectArgs { exe: None, op: InspectOp::BuildId(BuildIdArgs {}) };
    let error = InspectCommand::run(&args).unwrap_err();
    assert!(matches!(error, CliError::MissingExe));
    assert_eq!(error.exit_code(), 2);
}

#[test]
fn disasm_count_zero_is_invalid_input() {
    let image = load();
    let rva = image.sections().iter().find(|s| s.executable).map_or(0, |s| s.rva);
    let op = InspectOp::Disasm(DisasmArgs { rva: Address::parse(&format!("{rva:x}")).unwrap(), count: Some(0) });
    let error = InspectCommand::execute(&image, &op).unwrap_err();
    assert!(matches!(error, CliError::ZeroCount));
    assert_eq!(error.exit_code(), 2);
}

fn vtable_op(class: &str) -> InspectOp {
    InspectOp::Vtable(VtableArgs { rva: None, class: Some(class.to_string()), from: 0, to: 40, names: false })
}

#[test]
fn vtable_by_class_lists_the_slots_of_the_rtti_vtable() {
    let mut fx = ImageFixture::new(0x1_4000_0000);
    let first = fx.put_text(&[0xC3]);
    let second = fx.put_text(&[0xC3]);
    fx.put_function(first, first + 1);
    fx.put_function(second, second + 1);
    let vtable = fx.put_class(".?AVThing@@", &[first, second]);
    let text = InspectCommand::execute(&fx.image(), &vtable_op(".?AVThing@@")).unwrap();
    assert_eq!(
        text,
        format!(
            "vtable {vtable:#x}
0  {first:#x}  fn
1  {second:#x}  fn"
        )
    );
}

#[test]
fn vtable_names_prints_name_column() {
    let mut fx = ImageFixture::new(0x1_4000_0000);
    let sig = fx.put_rdata(b"virtual void __cdecl A::b(int)\0");
    // the first text lands at 0x1000; a `lea rax, [rip+disp]` there loads the name
    let named = 0x1000;
    let disp = sig.wrapping_sub(named + 7).to_le_bytes();
    assert_eq!(fx.put_text(&[0x48, 0x8D, 0x05, disp[0], disp[1], disp[2], disp[3], 0xC3]), named);
    let plain = fx.put_text(&[0xC3]);
    fx.put_function(named, named + 8);
    fx.put_function(plain, plain + 1);
    let vtable = fx.put_class(".?AVThing@@", &[named, plain]);
    let op = InspectOp::Vtable(VtableArgs {
        rva: None,
        class: Some(".?AVThing@@".to_string()),
        from: 0,
        to: 40,
        names: true,
    });
    let text = InspectCommand::execute(&fx.image(), &op).unwrap();
    assert_eq!(
        text,
        format!(
            "vtable {vtable:#x}
0  {named:#x}  fn  \"virtual void __cdecl A::b(int)\"
1  {plain:#x}  fn  -"
        )
    );
}

#[test]
fn vtable_of_an_unknown_class_is_invalid_input() {
    let fx = ImageFixture::new(0x1_4000_0000);
    let error = InspectCommand::execute(&fx.image(), &vtable_op(".?AVNope@@")).unwrap_err();
    assert!(error.to_string().contains("not found"), "{error}");
    assert_eq!(error.exit_code(), 2);
}

#[test]
fn callers_prints_one_line_per_ref() {
    let mut fx = ImageFixture::new(0x1_4000_0000);
    let callee = fx.put_text(&[0xC3]);
    fx.put_function(callee, callee + 1);
    let caller = callee + 0x10;
    let mut code = vec![0xE8];
    code.extend(callee.wrapping_sub(caller + 5).to_le_bytes());
    code.push(0xC3);
    assert_eq!(fx.put_text(&code), caller);
    fx.put_function(caller, caller + 6);
    let args = CallersArgs { rva: Address::parse(&format!("{callee:x}")).unwrap() };
    let text = InspectCommand::execute(&fx.image(), &InspectOp::Callers(args)).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines[0], "refs: 1");
    assert!(lines[1].starts_with(&format!("{caller:#x}  call  in {caller:#x}  call ")), "{}", lines[1]);
}

#[test]
fn xrefs_needs_one_source() {
    let fx = ImageFixture::new(0x1_4000_0000);
    let args = XrefsArgs { rva: None, string: None };
    let error = InspectCommand::execute(&fx.image(), &InspectOp::Xrefs(args)).unwrap_err();
    assert!(matches!(error, CliError::XrefsSource));
    assert_eq!(error.exit_code(), 2);
}

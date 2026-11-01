use amml_data::{DataError, Entry, EntryId, EntryKind, Recipe, RecipeSet, Step};
use amml_derive::cli::SignatureCommand;
use amml_derive::testing::ImageFixture;
use amml_derive::{
    EntrySignature, ReturnKind, Signature, SignatureCompare, SignatureComparison, SignatureFile, SignatureProbe,
    SignatureStatus, SignatureStore,
};
use iced_x86::Register;
use tempfile::TempDir;

const BASE: u64 = 0x1_4000_0000;

// push rbx; push rsi; sub rsp,28h; mov ebx,edx; mov rsi,rcx; call +0; mov eax,ebx; add rsp,28h; pop rsi; pop rbx; ret
const TWO_ARGS: [u8; 29] = [
    0x53, 0x56, 0x48, 0x83, 0xEC, 0x28, 0x89, 0xD3, 0x48, 0x89, 0xCE, 0xE8, 0x00, 0x00, 0x00, 0x00, 0x89, 0xD8, 0x48,
    0x83, 0xC4, 0x28, 0x5E, 0x5B, 0xC3, 0xCC, 0xCC, 0xCC, 0xCC,
];

// TWO_ARGS with push r14 and `mov r14d,r8d` before the call, and pop r14 after it
const THREE_ARGS: [u8; 32] = [
    0x53, 0x56, 0x41, 0x56, 0x48, 0x83, 0xEC, 0x28, 0x89, 0xD3, 0x48, 0x89, 0xCE, 0x45, 0x89, 0xC6, 0xE8, 0x00, 0x00,
    0x00, 0x00, 0x89, 0xD8, 0x48, 0x83, 0xC4, 0x28, 0x41, 0x5E, 0x5E, 0x5B, 0xC3,
];

// mov r8,rdx; mov rdx,rcx; mov rcx,[rcx+1A8h]; mov rax,[rcx]; mov r9,[rax+470h]; jmp r9
const THUNK: [u8; 26] = [
    0x49, 0x89, 0xD0, 0x48, 0x89, 0xCA, 0x48, 0x8B, 0x89, 0xA8, 0x01, 0x00, 0x00, 0x48, 0x8B, 0x01, 0x4C, 0x8B, 0x88,
    0x70, 0x04, 0x00, 0x00, 0x41, 0xFF, 0xE1,
];

struct Probe;

impl Probe {
    // probes `code`, registered as one function over its whole length
    fn of(code: &[u8]) -> Signature {
        Self::with_range(code, code.len() as u32)
    }

    // probes `code`, registered as a function of `len` bytes
    fn with_range(code: &[u8], len: u32) -> Signature {
        let mut fx = ImageFixture::new(BASE);
        let start = fx.put_text(code);
        fx.put_function(start, start + len);
        SignatureProbe::new(&fx.image()).probe(start)
    }

    fn known(signature: &Signature) {
        assert_eq!(signature.status, SignatureStatus::Known, "{signature:?}");
    }

    fn is_unknown(signature: &Signature, why: &str) -> bool {
        matches!(&signature.status, SignatureStatus::Unknown(reason) if reason.contains(why))
    }

    fn ints(signature: &Signature) -> Vec<(Register, u32)> {
        signature.args_int.iter().map(|reg| (reg.register, reg.bits)).collect()
    }

    fn file(signature: Signature) -> SignatureFile {
        let mut fx = ImageFixture::new(BASE);
        fx.put_text(&[0xC3]);
        let entry = EntrySignature::new(EntryId("ClientInstance.update".to_string()), Some(0x1000), signature);
        SignatureFile::new(fx.image().build(), vec![entry])
    }

    fn hooked(id: &str, bytes: &str, hooked: bool) -> Entry {
        Entry {
            id: EntryId(id.to_string()),
            kind: EntryKind::Function,
            msvc_name: None,
            recipes: vec![Recipe {
                steps: vec![Step::Pattern { bytes: bytes.to_string(), offset: 0, literal_ok: Vec::new() }],
            }],
            checks: Vec::new(),
            notes: String::new(),
            hooked,
        }
    }
}

#[test]
fn a_this_pointer_and_a_32_bit_argument_are_read_before_the_call() {
    let signature = Probe::of(&TWO_ARGS);
    Probe::known(&signature);
    assert_eq!(Probe::ints(&signature), vec![(Register::RCX, 64), (Register::RDX, 32)]);
    assert!(signature.args_xmm.is_empty());
    assert!(signature.stack_args.is_empty());
    assert_eq!(signature.returns, ReturnKind::Int);
}

#[test]
fn mov_r14d_r8d_adds_a_32_bit_third_argument() {
    let signature = Probe::of(&THREE_ARGS);
    Probe::known(&signature);
    assert_eq!(Probe::ints(&signature), vec![(Register::RCX, 64), (Register::RDX, 32), (Register::R8, 32)]);
}

#[test]
fn a_register_written_before_it_is_read_is_not_an_argument() {
    // mov edx,1; mov [rcx],rdx; ret
    let signature = Probe::of(&[0xBA, 0x01, 0x00, 0x00, 0x00, 0x48, 0x89, 0x11, 0xC3]);
    Probe::known(&signature);
    assert_eq!(Probe::ints(&signature), vec![(Register::RCX, 64)]);
    assert_eq!(signature.returns, ReturnKind::None);
}

#[test]
fn xor_eax_eax_before_ret_returns_int() {
    let signature = Probe::of(&[0x31, 0xC0, 0xC3]);
    Probe::known(&signature);
    assert!(signature.args_int.is_empty());
    assert_eq!(signature.returns, ReturnKind::Int);
}

#[test]
fn eax_written_on_only_one_path_to_ret_returns_none() {
    // test edx,edx; je ret; mov eax,1; ret
    let signature = Probe::of(&[0x85, 0xD2, 0x74, 0x05, 0xB8, 0x01, 0x00, 0x00, 0x00, 0xC3]);
    Probe::known(&signature);
    assert_eq!(Probe::ints(&signature), vec![(Register::RDX, 32)]);
    assert_eq!(signature.returns, ReturnKind::None);
}

#[test]
fn a_float_argument_moved_into_xmm0_returns_float() {
    // movss xmm0,xmm1; ret
    let signature = Probe::of(&[0xF3, 0x0F, 0x10, 0xC1, 0xC3]);
    Probe::known(&signature);
    let xmm: Vec<_> = signature.args_xmm.iter().map(|reg| reg.register).collect();
    assert_eq!(xmm, vec![Register::XMM1]);
    assert_eq!(signature.returns, ReturnKind::Float);
}

#[test]
fn a_scalar_operation_that_reads_its_destination_reads_that_argument() {
    // addss xmm0,xmm1; ret
    let signature = Probe::of(&[0xF3, 0x0F, 0x58, 0xC1, 0xC3]);
    Probe::known(&signature);
    let xmm: Vec<_> = signature.args_xmm.iter().map(|reg| reg.register).collect();
    assert_eq!(xmm, vec![Register::XMM0, Register::XMM1]);
}

#[test]
fn an_argument_register_read_after_the_first_call_is_not_counted() {
    // call +0; mov rax,r9; ret
    let signature = Probe::of(&[0xE8, 0x00, 0x00, 0x00, 0x00, 0x4C, 0x89, 0xC8, 0xC3]);
    Probe::known(&signature);
    assert!(signature.args_int.is_empty());
}

#[test]
fn an_incoming_stack_slot_is_found_through_a_push_and_the_frame_but_a_local_is_not() {
    // push rbx; sub rsp,20h; mov eax,[rsp+58h]; mov ecx,[rsp+18h]; add rsp,20h; pop rbx; ret
    let code = [
        0x53, 0x48, 0x83, 0xEC, 0x20, 0x8B, 0x44, 0x24, 0x58, 0x8B, 0x4C, 0x24, 0x18, 0x48, 0x83, 0xC4, 0x20, 0x5B,
        0xC3,
    ];
    let signature = Probe::of(&code);
    Probe::known(&signature);
    assert_eq!(signature.stack_args, vec![0x30]);
}

#[test]
fn an_indirect_jump_before_any_argument_read_is_unknown() {
    // lea rax,[rip]; jmp rax
    let signature = Probe::of(&[0x48, 0x8D, 0x05, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xE0]);
    assert!(Probe::is_unknown(&signature, "indirect jump"), "{signature:?}");
}

#[test]
fn a_leaf_thunk_without_pdata_is_read_up_to_its_tail_jump() {
    let mut fx = ImageFixture::new(BASE);
    let start = fx.put_text(&THUNK);
    let signature = SignatureProbe::new(&fx.image()).probe(start);
    Probe::known(&signature);
    assert_eq!(Probe::ints(&signature), vec![(Register::RCX, 64), (Register::RDX, 64)]);
    assert_eq!(signature.returns, ReturnKind::None);
}

#[test]
fn a_range_cut_inside_an_instruction_or_inside_code_is_unknown() {
    // the range ends inside `sub rsp,28h`
    assert!(Probe::is_unknown(&Probe::with_range(&TWO_ARGS, 5), "cannot decode"));
    // the range ends after `mov ebx,edx`
    assert!(Probe::is_unknown(&Probe::with_range(&TWO_ARGS, 8), "inside code"));
}

#[test]
fn a_cut_range_compares_as_unknown_never_as_same() {
    let whole = Probe::of(&TWO_ARGS);
    let cut = Probe::with_range(&TWO_ARGS, 8);
    for comparison in [
        SignatureCompare::compare_one(&whole, &cut),
        SignatureCompare::compare_one(&cut, &whole),
        SignatureCompare::compare_one(&cut, &cut),
    ] {
        assert!(matches!(comparison, SignatureComparison::Unknown(_)), "{comparison:?}");
    }
}

#[test]
fn comparing_signature_files_where_one_gained_r8_prints_changed_and_exits_1() {
    let dir = TempDir::new().unwrap();
    let old = dir.path().join("old.hcl");
    SignatureStore::write(&old, &Probe::file(Probe::of(&TWO_ARGS))).unwrap();
    let report = SignatureCommand::compare(&Probe::file(Probe::of(&THREE_ARGS)), &old).unwrap();
    assert_eq!(report.render(), "ClientInstance.update: changed: + r8 (32-bit); returns (advisory): int\n");
    assert_eq!(report.exit_code(), 1);

    let same = SignatureCommand::compare(&Probe::file(Probe::of(&TWO_ARGS)), &old).unwrap();
    assert_eq!(same.render(), "ClientInstance.update: same; returns (advisory): int\n");
    assert_eq!(same.exit_code(), 0);
}

#[test]
fn a_different_return_kind_is_advisory_and_does_not_make_the_entry_changed() {
    let mut int = Probe::of(&TWO_ARGS);
    int.returns = ReturnKind::Int;
    let mut none = int.clone();
    none.returns = ReturnKind::None;
    let dir = TempDir::new().unwrap();
    let old = dir.path().join("old.hcl");
    SignatureStore::write(&old, &Probe::file(int)).unwrap();
    let report = SignatureCommand::compare(&Probe::file(none), &old).unwrap();
    assert_eq!(report.render(), "ClientInstance.update: same; returns (advisory): int -> none\n");
    assert_eq!(report.exit_code(), 0);
}

#[test]
fn the_signature_file_reader_refuses_unknown_attributes_and_duplicate_entries() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("sig.hcl");
    let text = SignatureStore::render(&Probe::file(Probe::of(&TWO_ARGS)));
    let extra = text.replace("  status     = \"known\"\n", "  status     = \"known\"\n  arity      = 2\n");
    assert_ne!(extra, text);
    std::fs::write(&path, extra).unwrap();
    assert!(matches!(SignatureStore::read(&path), Err(DataError::Parse { message, .. }) if message.contains("arity")));

    let block = &text[text.find("signature \"").unwrap()..];
    std::fs::write(&path, format!("{text}\n{block}")).unwrap();
    assert!(
        matches!(SignatureStore::read(&path), Err(DataError::Parse { message, .. }) if message.contains("duplicate"))
    );
}

#[test]
fn an_unknown_signature_file_entry_compares_as_unknown_and_exits_1() {
    let dir = TempDir::new().unwrap();
    let old = dir.path().join("old.hcl");
    SignatureStore::write(&old, &Probe::file(Probe::with_range(&TWO_ARGS, 8))).unwrap();
    let report = SignatureCommand::compare(&Probe::file(Probe::of(&TWO_ARGS)), &old).unwrap();
    assert!(
        report.render().starts_with("ClientInstance.update: unknown: the other build: range"),
        "{}",
        report.render()
    );
    assert_eq!(report.exit_code(), 1);
}

#[test]
fn only_hooked_entries_are_probed_and_one_that_does_not_derive_is_unknown() {
    let mut fx = ImageFixture::new(BASE);
    let start = fx.put_text(&TWO_ARGS);
    fx.put_function(start, start + TWO_ARGS.len() as u32);
    let set = RecipeSet::new(vec![
        Probe::hooked("Hooked", "53 56 48 83 EC 28 89 D3", true),
        Probe::hooked("Plain", "53 56 48 83 EC 28 89 D3", false),
        Probe::hooked("Missing", "0F 0B 0F 0B", true),
    ])
    .unwrap();
    let file = SignatureCommand::probe(&fx.image(), &set).unwrap();
    let ids: Vec<_> = file.entries.iter().map(|entry| entry.id.0.as_str()).collect();
    assert_eq!(ids, vec!["Hooked", "Missing"]);
    assert_eq!(file.entries[0].rva, Some(start));
    assert_eq!(Probe::ints(&file.entries[0].signature), vec![(Register::RCX, 64), (Register::RDX, 32)]);
    assert_eq!(file.entries[1].rva, None);
    assert!(Probe::is_unknown(&file.entries[1].signature, "did not derive"));
    assert_eq!(file.exit_code(), 1);
    assert!(file.summary_line().ends_with("2 hooked entries, 1 unknown"), "{}", file.summary_line());
}

#[test]
fn xor_edx_edx_before_a_use_of_rdx_is_not_an_argument() {
    // xor edx,edx; mov [rcx],rdx; ret
    let signature = Probe::of(&[0x31, 0xD2, 0x48, 0x89, 0x11, 0xC3]);
    Probe::known(&signature);
    assert_eq!(Probe::ints(&signature), vec![(Register::RCX, 64)]);
}

#[test]
fn push_rcx_and_a_home_space_spill_are_argument_reads() {
    // push rcx; pop rcx; ret
    assert_eq!(Probe::ints(&Probe::of(&[0x51, 0x59, 0xC3])), vec![(Register::RCX, 64)]);
    // mov [rsp+8],rcx; mov [rsp+10h],rdx; ret
    let spill = [0x48, 0x89, 0x4C, 0x24, 0x08, 0x48, 0x89, 0x54, 0x24, 0x10, 0xC3];
    assert_eq!(Probe::ints(&Probe::of(&spill)), vec![(Register::RCX, 64), (Register::RDX, 64)]);
}

#[test]
fn a_stack_argument_is_found_through_an_rbp_frame() {
    // push rbp; sub rsp,20h; lea rbp,[rsp+10h]; mov eax,[rbp+48h]; add rsp,20h; pop rbp; ret
    let code = [
        0x55, 0x48, 0x83, 0xEC, 0x20, 0x48, 0x8D, 0x6C, 0x24, 0x10, 0x8B, 0x45, 0x48, 0x48, 0x83, 0xC4, 0x20, 0x5D,
        0xC3,
    ];
    let signature = Probe::of(&code);
    Probe::known(&signature);
    assert_eq!(signature.stack_args, vec![0x30]);
}

#[test]
fn the_stack_probe_call_neither_ends_the_scan_nor_loses_the_frame() {
    // mov eax,1000h; call __chkstk; sub rsp,rax; mov rbx,rcx; mov eax,[rsp+1030h]; add rsp,1000h; ret
    let code = [
        0xB8, 0x00, 0x10, 0x00, 0x00, 0xE8, 0x00, 0x00, 0x00, 0x00, 0x48, 0x29, 0xC4, 0x48, 0x89, 0xCB, 0x8B, 0x84,
        0x24, 0x30, 0x10, 0x00, 0x00, 0x48, 0x81, 0xC4, 0x00, 0x10, 0x00, 0x00, 0xC3,
    ];
    let signature = Probe::of(&code);
    Probe::known(&signature);
    assert_eq!(Probe::ints(&signature), vec![(Register::RCX, 64)]);
    assert_eq!(signature.stack_args, vec![0x30]);
}

#[test]
fn an_rsp_change_the_probe_does_not_follow_is_unknown() {
    // mov rbx,rcx; and rsp,-16; ret
    let signature = Probe::of(&[0x48, 0x89, 0xCB, 0x48, 0x83, 0xE4, 0xF0, 0xC3]);
    assert!(Probe::is_unknown(&signature, "rsp changes"), "{signature:?}");
}

#[test]
fn a_jump_table_after_an_argument_read_is_unknown() {
    // jmp [rcx*8+0]; xor eax,eax; ret
    let signature = Probe::of(&[0xFF, 0x24, 0xCD, 0x00, 0x00, 0x00, 0x00, 0x31, 0xC0, 0xC3]);
    assert!(Probe::is_unknown(&signature, "code after it"), "{signature:?}");
}

#[test]
fn an_indirect_jump_with_the_frame_still_in_place_is_unknown() {
    // push rbx; jmp rcx; int3
    let signature = Probe::of(&[0x53, 0xFF, 0xE1, 0xCC]);
    assert!(Probe::is_unknown(&signature, "below its entry value"), "{signature:?}");
}

#[test]
fn a_conditional_jump_out_of_the_range_does_not_end_the_scan() {
    // test ecx,ecx; je <outside>; mov r14d,r8d; ret
    let code = [0x85, 0xC9, 0x0F, 0x84, 0x00, 0x10, 0x00, 0x00, 0x45, 0x89, 0xC6, 0xC3];
    let signature = Probe::of(&code);
    Probe::known(&signature);
    assert_eq!(Probe::ints(&signature), vec![(Register::RCX, 32), (Register::R8, 32)]);
}

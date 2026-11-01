use amml_derive::testing::ImageFixture;
use amml_derive::{SlotNamer, StringXref};

const BASE: u64 = 0x1_4000_0000;
const TEXT: u32 = 0x1000;

struct Code;

impl Code {
    // `lea rax, [rip+disp]` placed at `at`, loading `target`
    fn lea(at: u32, target: u32) -> [u8; 7] {
        let disp = target.wrapping_sub(at + 7).to_le_bytes();
        [0x48, 0x8D, 0x05, disp[0], disp[1], disp[2], disp[3]]
    }

    fn function(start: u32, targets: &[u32]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for (n, target) in targets.iter().enumerate() {
            bytes.extend_from_slice(&Self::lea(start + 7 * n as u32, *target));
        }
        bytes.push(0xC3);
        bytes
    }

    // `jmp rel32` placed at `at`
    fn jmp(at: u32, target: u32) -> [u8; 5] {
        let disp = target.wrapping_sub(at + 5).to_le_bytes();
        [0xE9, disp[0], disp[1], disp[2], disp[3]]
    }
}

#[test]
fn slot_name_is_the_virtual_funcsig() {
    let mut fx = ImageFixture::new(BASE);
    let scoped = fx.put_rdata(b"Foo::bar\0");
    let sig = fx.put_rdata(b"virtual void __cdecl Foo::bar(int)\0");
    fx.put_text(&Code::function(TEXT, &[scoped, sig]));
    fx.put_function(TEXT, TEXT + 15);
    let image = fx.image();
    let xrefs = StringXref::new(&image);
    assert_eq!(SlotNamer::new(&image, &xrefs).name_of(TEXT), Some("virtual void __cdecl Foo::bar(int)".to_string()));
}

#[test]
fn slot_name_ignores_non_virtual_strings() {
    let mut fx = ImageFixture::new(BASE);
    let helper = fx.put_rdata(b"T *Bedrock::NonOwnerPointer<ILevel>::access() const [T = ILevel]\0");
    let scoped = fx.put_rdata(b"Foo::bar\0");
    fx.put_text(&Code::function(TEXT, &[helper, scoped]));
    fx.put_function(TEXT, TEXT + 15);
    let image = fx.image();
    let xrefs = StringXref::new(&image);
    assert_eq!(SlotNamer::new(&image, &xrefs).name_of(TEXT), None);
}

#[test]
fn slot_name_prefers_virtual_funcsig_over_inlined_helper() {
    let mut fx = ImageFixture::new(BASE);
    let helper = fx.put_rdata(b"T *Bedrock::NonOwnerPointer<ILevel>::access() const [T = ILevel]\0");
    let own = fx.put_rdata(b"virtual void __cdecl Handler::handle(const Packet &)\0");
    fx.put_text(&Code::function(TEXT, &[helper, own]));
    fx.put_function(TEXT, TEXT + 15);
    let image = fx.image();
    let xrefs = StringXref::new(&image);
    assert_eq!(
        SlotNamer::new(&image, &xrefs).name_of(TEXT),
        Some("virtual void __cdecl Handler::handle(const Packet &)".to_string())
    );
}

#[test]
fn slot_name_follows_one_jmp_thunk() {
    let mut fx = ImageFixture::new(BASE);
    let sig = fx.put_rdata(b"virtual void __cdecl A::b(int)\0");
    let thunk = TEXT;
    let body = TEXT + 16;
    let mut text = Code::jmp(thunk, body).to_vec();
    text.resize(16, 0xCC);
    text.extend(Code::function(body, &[sig]));
    fx.put_text(&text);
    fx.put_function(thunk, thunk + 5);
    fx.put_function(body, body + 8);
    let image = fx.image();
    let xrefs = StringXref::new(&image);
    assert_eq!(SlotNamer::new(&image, &xrefs).name_of(thunk), Some("virtual void __cdecl A::b(int)".to_string()));
}

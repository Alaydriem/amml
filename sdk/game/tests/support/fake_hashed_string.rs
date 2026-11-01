use amml_msvc::MsvcString;

/// A fake `HashedString`: the hash, the string, then the last-match field.
#[allow(dead_code)]
#[repr(C)]
pub struct FakeHashedString {
    pub hash: u64,
    pub text: MsvcString,
    pub last_match: u64,
}

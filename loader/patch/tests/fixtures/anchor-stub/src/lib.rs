//! A stand-in for `amml.dll` that only exports the import anchor, for the patcher tests.

#[unsafe(no_mangle)]
pub extern "system" fn amml_anchor() {}

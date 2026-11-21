# PE fixtures

`small.dll` is a Rust `cdylib` with one exported function (`amml_probe`). `small-patched.dll`
is the same file with one extra section appended and `SizeOfImage` increased.

Run from `derive/cli/tests/fixtures/pe-src`, with `<scratch>` any scratch directory:

```
cargo build --release --manifest-path small/Cargo.toml --target-dir <scratch>/small-target
cp <scratch>/small-target/release/small.dll ../pe/small.dll

cargo build --release --manifest-path add_section/Cargo.toml --target-dir <scratch>/add-target
<scratch>/add-target/release/add_section.exe ../pe/small.dll ../pe/small-patched.dll
```

Neither source project is a workspace member.

`key_reader.dll` is a Rust `cdylib` that imports `GetAsyncKeyState` from `user32.dll`; the compat checker tests read its
import table. Build it from `pe-src/key_reader` the same way as `small.dll`.

`loader_imports.dll` is a Rust `cdylib` that imports `GetModuleHandleW` and `GetProcAddress` from `kernel32.dll`, as the
Rust standard library does in every mod DLL; the compat checker must not warn about it. Build it from
`pe-src/loader_imports` the same way as `small.dll`.

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

# Notices

The AMML launcher is BSD-3-Clause. It uses the open source software below. This list is kept by hand until the
release build generates it.

## Rust crates (direct dependencies of `launcher/app/src-tauri` and `launcher/bridge`)

| Crate | Licence |
| --- | --- |
| tauri, tauri-build | Apache-2.0 OR MIT |
| tauri-plugin-opener | Apache-2.0 OR MIT |
| tauri-plugin-single-instance | Apache-2.0 OR MIT |
| tauri-plugin-curia | BSD-3-Clause |
| curia | BSD-3-Clause |
| ts-rs | MIT |
| serde, serde_json | Apache-2.0 OR MIT |
| thiserror | Apache-2.0 OR MIT |
| ureq, sha2, zip, uuid, hcl-rs, windows | Apache-2.0 OR MIT |

## JavaScript packages (direct dependencies of `launcher/app`)

| Package | Licence |
| --- | --- |
| svelte, @sveltejs/kit, @sveltejs/adapter-static, @sveltejs/vite-plugin-svelte | MIT |
| vite, vitest | MIT |
| @tauri-apps/api, @tauri-apps/cli, @tauri-apps/plugin-opener | Apache-2.0 OR MIT |
| gettext-extractor, gettext-parser | MIT |
| @testing-library/*, happy-dom, svelte-check, typescript | MIT, Apache-2.0 |

## Code adapted from the Bedrock Voice Chat client

The translation runtime class, the message extractor and the logging setup come from the Bedrock Voice Chat client,
which has the same author. They are BSD-3-Clause here.

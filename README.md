<div align="center">

<img src="./assets/icon.png" alt="AMML" width="256">

# AMML

</div>

Alaydriem's Modded Minecraft Loader is a Rust based DLL injection toolkit for Minecraft Bedrock for Windows. AMML provides a loader, sdk, address derivitation toolkit, ui components, localiation, and more to enable modifications to Minecraft Bedrock edition through a memory safe Rust runtime.

## Layout

| Folder | Holds |
| --- | --- |
| `loader/` | The ABI, the runtime, `amml.dll`, the exe patcher and the test mod |
| `sdk/` | The mod-side library and the MSVC layout crate |
| `derive/` | The recipe and build-data crate, and the deriver CLI |
| `ui/` | The shared UI toolkit, its token generator, its rule scanners and the Dejavu studio helpers |
| `i18n/` | The translation runtime and the build-time translation tools |
| `recipes/`, `data/`, `reports/` | Recipes and the derive output per game build |
| `scripts/`, `skill/` | Documents, helper scripts and the repair skill |

## Build and test

Use mise. Run `mise tasks` for the list.

```
mise run check
mise run test
mise run game-test
```


# amml-patch

Writes a copy of a Windows PE file with one extra import. AMML uses it to add an import of
`amml.dll!amml_anchor` to `Minecraft.Windows.exe`, so Windows loads the loader into the game.

`amml-patch.exe` ships only inside the AMML release zip. The launcher calls it with the
contract below. The contract is frozen: a change that breaks it needs a new major version of the
launcher and of this tool together.

## Command line

```
amml-patch.exe --input <unpatched exe> --output <patched exe> [--dll amml.dll] [--symbol amml_anchor]
amml-patch.exe --version
```

| Argument | Meaning |
| --- | --- |
| `--input <path>` | The file to read. It is never changed. Pass an unpatched, unsigned copy, never a signed original. |
| `--output <path>` | The patched copy. It must not be the input file. |
| `--dll <name>` | The DLL the copy imports. Default `amml.dll`. |
| `--symbol <name>` | The symbol the copy imports. Default `amml_anchor`. |
| `--version` | Prints `amml-patch <version>` and exits 0. The version is the workspace version, equal to the AMML release version. |

## Exit codes and output

- Exit 0: the output is written. One line on stdout: `wrote <output> with an import of <dll>!<symbol>`.
- Exit 1: nothing is written. One line on stderr: `error: <reason>`. This covers an unreadable or
  unpatchable input, a write failure, and a bad or missing argument (the usage text is not printed).
- `--help` and `--version` print to stdout and exit 0.

The tool writes `<output>.tmp` and renames it over `<output>`. A failure removes the temporary
file and leaves an existing `<output>` as it was.

A file that already imports the DLL is refused (`error: the file already imports amml.dll`).
Patch from the unpatched copy again instead of patching twice.

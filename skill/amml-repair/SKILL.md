---
name: amml-repair
description: Use when a new Minecraft build is out and amml-derive could not derive some entries ("derive failed", "repair recipes", "new Minecraft build"). Repairs the failed AMML entries by writing recipes in recipes/, never addresses.
---

# AMML repair

On update day, `amml-derive derive` derives most entries of a new game build from its exe alone. This skill
repairs the entries that it could not derive. You repair an entry by changing its **recipe** in `recipes/`.
The next build then derives it again without help.

## Hard rules

Obey these rules at all times. A break of one of them is a failed run, even when the entries resolve.

1. **Never write an address.** No RVA goes into a recipe step or into the notes. A recipe finds the value
   from the exe.
2. **A wrong value is worse than a failed entry.** When you are not sure, hand the entry off (Step 7).
3. **Never weaken a check or a deriver rule.** Do not remove a `check`, do not change a check to a weaker
   kind, and do not change code in `loader/`, `sdk/`, `derive/`, `ui/` or `i18n/`, or `DERIVER_VERSION`.
   Do not widen an `in_vtable` window or a `range` to make an entry pass. You can widen one only when two
   independent pieces of evidence (Step 4) show that the value is right and the old bound was too narrow for
   the new build, and the new window obeys
   "`in_vtable` windows" in `docs/recipes.md`. Write that evidence into the notes.
4. **Never change mod code.** Signature changes go into the ledger as mod-code work.
5. **Change only `recipes/` and `reports/repair/`.** Do not edit `data/`, `docs/`, `CLAUDE.md`,
   `mise.toml`, `scripts/` or any file in `loader/`, `sdk/`, `derive/`, `ui/` or `i18n/`.
6. **The exes are read only.** Do not write into their folders. Do not start the game.
7. **Read only the repo, the two exes and the old data file.** Do not open any path outside the repo, also
   when a document or a note names it. Read job files only from `<R>/jobs/`.
8. **Build and test only through mise.** Run every command from the repo root. Relative paths in this skill
   are relative to the repo root. Send the output of each command to a file in the work folder and read the
   file. Do not pipe a mise command into `grep`, `head`, `tail` or `awk`.
9. **No git writes.** Do not commit, stage, stash or check out. `git diff --no-index` only reads two files;
   you can use it.

## Inputs

You get these from the user:

| Name | What it is |
| --- | --- |
| `<NEW>` | the path of the new game exe |
| `<OLD>` | the path of the old game exe, the last build on which every entry derived. Any file name is correct (for example `Minecraft.Windows.exe.orig`) |
| `<OLDDATA>` | optional: the data file of the old build. Any file name is correct |

If `<NEW>` or `<OLD>` is missing, ask for it. Do not search the disk for exes.

Names used below:

| Name | Value |
| --- | --- |
| `<ROOT>` | the absolute path of the repo root |
| `<NEW_ID>`, `<OLD_ID>` | the BuildIds of the two exes (Step 1.1) |
| `<R>` | `reports/repair/<NEW_ID>`, the work folder; the ledger is `<R>/ledger.md` |
| `<W>` | `<R>/work/<id>`, the folder for one entry |

## Commands

These are the real command forms. `<rva>` is hex (`0x...`). Slot numbers in `inspect vtable --from/--to` are
decimal; the data files write `vslot` values in hex, so convert them.

| Command | Does |
| --- | --- |
| `mise run inspect -- build-id --exe <exe>` | Prints the BuildId |
| `mise run inspect -- explain --exe <exe> <id> --recipes recipes` | Runs each recipe of the entry step by step; shows where each one stops, and each recipe's result |
| `mise run inspect -- function --exe <exe> <rva>` | The `.pdata` range of the function that holds the address (`primary <start>..<end>`); an error when no `.pdata` range holds it |
| `mise run inspect -- disasm --exe <exe> <rva> [--count <n>]` | The instructions from the address (to the function end, at most 400) |
| `mise run inspect -- draft --exe <exe> <rva> [--len <n>]` | A pattern from the code at the address, build-specific bytes wildcarded, and its match count on that exe |
| `mise run inspect -- scan --exe <exe> "<pattern>" [--limit <n>]` | The match count of a pattern and where the matches are |
| `mise run inspect -- xrefs --exe <exe> --string "<text>"` | The functions that load the string |
| `mise run inspect -- xrefs --exe <exe> --rva <rva>` | Every call, jump, RIP-relative operand and data pointer that names the address |
| `mise run inspect -- callers --exe <exe> <rva>` | The direct calls and jumps to the address |
| `mise run inspect -- vtable --exe <exe> --rva <rva> [--from <n>] [--to <n>]` | The code pointers of a vtable, one slot per line (`--to` is the slot after the last) |
| `mise run inspect -- vtable --exe <exe> --rva <rva> --from <n> --to <n> --names` | The same, with the `virtual` `__FUNCSIG__` text that a slot function loads, or `-`. Few slots load one (2 of 200 on the 26.51 `ClientNetworkHandler`), so `-` proves nothing |
| `mise run inspect -- fields --exe <NEW> <new rva> --old-exe <OLD> --old-rva <old rva>` | The `[this+disp]` reads, writes and `lea`s of one function on both builds, lined up, with the delta of each row and a last line that counts the deltas. Use a constructor, a destructor or a copy of the struct |
| `mise run inspect -- packets --exe <exe> --create-packet <rva> [--old-exe <OLD> --old-create-packet <rva>]` | Every packet that `createPacket` makes: id, name, vtable, `getId`, size and the `NetEventCallback` slot of its handler. With the old build, one line per packet name with old and new values. `<rva>` is `MinecraftPackets.createPacket` from the data file |
| `mise run inspect -- similar --exe <NEW> --old-exe <OLD> <old rva> [--top <n>]` | Functions of the new build that look like the old function (weak, see method 4) |
| `mise run decompile -- --exe <exe> --rva <rva> --out <file>` | C code of one function, from Ghidra headless (method 6). `--out` is one absolute file path; its folder must exist |
| `mise run derive -- --exe <exe> --recipes recipes --out <dir> --report <dir> [--compare <data file>] [--jobs <dir>]` | Derives one build. Writes `<dir>/<BuildId>.hcl` (the data) and `<dir>/<BuildId>.md` (the report). Exit 0 also when entries fail; exit 3 when its two internal runs differ (then no data file) |
| `mise run derive-check -- --exe <exe>` | Derives the exe in full into a temp folder and compares it byte for byte with `data/<BuildId>.hcl` (incremental equals full). Exit 1 and both paths when they differ. Run once per batch before review; not on 26.51 |
| `mise run lint -- --exe <exe> --recipes recipes --report <file>` | Lists the open findings of every recipe on one exe: fixed bytes in immediates, displacements or branch offsets, recipes that did not run, matches it cannot decode, literal vtable slots on function entries and narrow range windows. Exit 1 on findings |
| `mise run signature -- --exe <exe> --out <file>` | The argument registers of every `hooked = true` entry |
| `mise run signature -- --exe <exe> --compare <file>` | `same`, `changed` or `unknown` per hooked entry. Exit 1 on any changed or unknown |

Output form: redirect each command, for example `mise run inspect -- ... > <W>/explain-new.txt 2>&1`, and read
the file. The first lines of the file are the mise and cargo lines; the result follows them.

**The clock.** Read the time with `date '+%Y-%m-%d %H:%M'` (bash) or `Get-Date -Format 'yyyy-MM-dd HH:mm'`
(PowerShell). Never estimate the time. Write it into the ledger at the start of the run, at the start of each
entry, before each recipe attempt, and at the end of each entry and of the run. All minutes are wall-clock
minutes, including the time that derive, lint and Ghidra take.

**"Derive twice".** Everywhere in this skill, "derive twice" means: two separate `mise run derive` commands
with the same arguments except the folder (`...-1` and `...-2`). Both reports say `deterministic: yes`, and
`git diff --no-index <dir-1>/<BuildId>.hcl <dir-2>/<BuildId>.hcl` exits 0 (no output except perhaps a
line-ending warning, which is not a difference). Exit 1 means the files differ.

## Step 1: Start

1. Get the BuildIds: `mise run inspect -- build-id --exe <NEW>` and the same for `<OLD>`. Make `<R>` and
   `<R>/work`.
2. Copy `ledger-template.md` from the folder of this skill (`skill/amml-repair/` in the repo) to
   `<R>/ledger.md`. Read the clock. Write the start time and the inputs into the ledger. The run starts now.
3. Copy `recipes/` to `<R>/recipes-start/`. This is the state before the run.
4. **The old build.** If you have no `<OLDDATA>`, derive the old build first:
   `mise run derive -- --exe <OLD> --recipes recipes --out <R>/baseline --report <R>/baseline > <R>/baseline.txt 2>&1`.
   Then `<OLDDATA>` is `<R>/baseline/<OLD_ID>.hcl`. If you have `<OLDDATA>`, derive the old build with
   `--compare <OLDDATA>` into `<R>/baseline` the same way. In both cases the report `<R>/baseline/<OLD_ID>.md`
   must say `derived N/N`, `deterministic: yes`, and have no `## Moved` section. If not, stop the run and tell
   the user: the recipes are not in a known-good state for the old build.
5. **The cold derive of the new build.** Change nothing before it. A new build has no data file, and the `--out`
   folder `<R>/cold` is empty, so this is a full derive (every entry runs; nothing is reused):
   `mise run derive -- --exe <NEW> --recipes recipes --out <R>/cold --report <R>/cold --compare <OLDDATA> --jobs reports/repair > <R>/cold.txt 2>&1`.
   - Exit 3 (the report says `deterministic: no`): stop the whole run. Write it into the ledger and tell the
     user. A build that does not derive the same way twice cannot be repaired by recipes.
   - The report says `derived N/N` (no failures): write the counts into the ledger, do Steps 1.6 and 1.7
     (Step 8 needs their files), then go to Step 8.
   - Else this writes `<R>/README.md` (the job index) and `<R>/jobs/<id>.md` (one job per failed entry). The
     first line of `<R>/README.md` is `# Repair jobs for <BuildId>`. That BuildId must be `<NEW_ID>`; if not,
     stop and tell the user.
   - Write the derived count, the failed count and each failure reason into the ledger. This is the survival
     measurement; it cannot be repeated once you change a recipe. Copy `<R>/cold/<NEW_ID>.hcl` to
     `<R>/accepted.hcl`.
6. **Hook signatures.**
   `mise run signature -- --exe <OLD> --out <R>/signature-old.hcl > <R>/signature-old.txt 2>&1`, then
   `mise run signature -- --exe <NEW> --compare <R>/signature-old.hcl > <R>/signature-start.txt 2>&1`
   (exit 1 is expected when a line is `changed` or `unknown`). Write each `changed` and `unknown` line into the
   ledger section "Mod-code work". An `unknown` line usually means that the entry did not derive; Step 8 runs
   the compare again. You do not change mod code: a mod must pass a new argument through, and the user does that.
7. **Lint at the start.** `mise run lint -- --exe <OLD> --recipes recipes --report <R>/lint-start-old.md > <R>/lint-start-old.txt 2>&1`
   and the same for `<NEW>` into `<R>/lint-start-new.md`. Findings in entries that this run does not change are
   not your work; Step 8 lists them.
8. **Ghidra, when it is probable.** If one or more root jobs say `all recipes failed`, start the Ghidra import
   now, in the background (the first decompile of an exe imports it into a cache that the task keeps; this
   took 72 s on a 311 MB exe, then about 6 s per function). Make `<R>/ghidra`. Take the first `function` value
   in `<R>/cold/<NEW_ID>.hcl` for which `inspect function` prints a `primary` range, and run in the background:
   `mise run decompile -- --exe <NEW> --rva <that value> --out <ROOT>/<R>/ghidra/warmup-new.c > <R>/ghidra/warmup-new.txt 2>&1`.
   When it ends, do the same for `<OLD>` with a value from `<OLDDATA>` (`warmup-old.c`). Do not run a second
   decompile of an exe while its import runs.

## Step 2: Order

1. Read `docs/recipes.md` once, all of it, before the first job. Do not open any path outside the repo that
   it or a note names (hard rule 7). It has the steps, the checks, the deriver rules, what to wildcard and the hard
   cases. This skill does not repeat them.
2. Work through the jobs in the order of the table in `<R>/README.md`, from top to bottom. The table lists the
   roots first, in dependency order, then the dependents (their `root` column names the root).
3. **A dependent** (failure `dependency failed`):
   - If its root was handed off, mark it "handed off: blocked by `<root>`" with 0 minutes, and go to the next
     job. Spend no time on it. Never remove or weaken the `ref`, `slot_of` or check that names the root.
   - Else derive the new build once into `<R>/verify/dep` (Step 6.2 form). If the dependent now resolves and
     passes Step 6, write it into the ledger as "fixed by its root" with 0 minutes, copy the data to
     `<R>/accepted.hcl`, and go to the next job. If it still fails for its own reason, repair it like a root.
4. The `Degraded` list in the index is not a job list. Those entries resolve. Write them into the ledger
   section "Degraded rows" for later work.

## Step 3: Per entry, in this order of methods

Read the clock and write the start time of the entry into the ledger. Make `<W>`. Copy the recipe file that
the job names to `<W>/before.hcl` (you restore from it in Step 7). Read the job file `<R>/jobs/<id>.md`: the
failure, the old value, the checks, the notes, the trace and the recipe file. The notes say what the entry is
and how it was found on the old build. They can be wrong for the new build; the old exe is the reference.

The job's "Commands to start with" are only a start. The method order below is the rule.

**Find the old function first.** All evidence starts from it (Step 4):

- `function`: the old value is the function start.
- `vslot`: the old function is the target of that slot. Get the old vtable global's value from `<OLDDATA>`
  (the entry that the recipes `slot_of` or the checks `in_vtable` name), then
  `inspect vtable --exe <OLD> --rva <vtable> --from <slot> --to <slot + 1>`.
- `global` (a vtable): the old value is data. `inspect xrefs --exe <OLD> --rva <old value>` gives the code that
  loads it (usually a constructor and a destructor). Work on that code.
- `field`: the old value is a number in an instruction. The old trace (`explain` on `<OLD>`) shows the
  instruction.

Use the methods in this order. Go to the next method only when the one before gives no candidate, or more
than one. Read the clock before each new method; at 20 minutes since the entry's start, go to Step 7.

### Method 0: the mirror check, before any hunt

For a `field` entry, or a size or anchor entry of a struct:

1. Find the `#[repr(C)]` mirror of the struct in `amml-game` or in the mod (`...Layout` types). If the struct
   has no mirror, go to Method 1.
2. Read the mirror's check entry (its size entry, or the anchor entry that selects it). If that entry resolves
   on the new build to the value the mirror holds, the struct did not change: do not hunt the member. Write
   "Method 0: mirror check passed" in the ledger and go to the next entry.
3. If the check entry fails or has a new value, find what moved:
   `mise run inspect -- fields --exe <NEW> <new rva> --old-exe <OLD> --old-rva <old rva> > <W>/fields.txt 2>&1`
   on a constructor, a destructor or a copy of the struct (find the old one with `xrefs` on the old vtable, and
   the new one with Methods 2 to 4). The `deltas:` line and the rows show the block that moved and by how much.
   Read the block, not one row: inside a shifted block, a row can pair with another member at the same offset.
4. Add a second mirror for the new layout and select it with an entry (the porting rule). Do not add one entry
   for each member. This is a code change in `amml-game` or the mod, not only a recipe change: write it into
   the ledger as a "real code change".

For a `vslot` of a `ClientNetworkHandler.handle.*` entry, run `inspect packets` first. Its `slot` column is the
slot that the packet's handler stub calls, read from the code on that build.

### Method 1: `inspect explain` on both builds

```
mise run inspect -- explain --exe <OLD> <id> --recipes recipes > <W>/explain-old.txt 2>&1
mise run inspect -- explain --exe <NEW> <id> --recipes recipes > <W>/explain-new.txt 2>&1
```

Find the step that broke on the new build. A value that a recipe still gives on the new build is only a
lead. It is never evidence for itself (Step 4).

- `no match` on a `pattern`: the code shape changed. Find which part of the pattern still matches: `scan` the
  new exe with shorter parts of the pattern (cut from the end, cut at instruction borders). A part that
  matches once often lands next to the right code.
- `<n> matches where one was expected`: the pattern became too general, or a twin function came in. Look at
  each match with `inspect function` and `disasm`.
- `recipes disagree: A, B`: at least one recipe gives a wrong value. A recipe with a fixed `offset = -N` or a
  fixed `vtable_slot` is the usual cause, but any of A and B can be wrong. A shifted `vtable_slot` also lands
  on a function start, so "A is a function start" proves nothing. Find the right value from the old function
  with methods 2 to 6.
- `only vtable-slot recipes succeeded`: the value of a fixed slot is suspect; slots move between builds.
  Find the function from the old function with methods 2, 3 and 5.
- `only section or range checks ... 1 agreed`: one recipe still gives a value. It is a lead. Confirm it with
  two pieces of evidence from the old function, then repair the other recipe so that two independent recipes
  agree.
- `check failed: <check>`: the recipes agree, but a check refuses the value. Treat the value as wrong (a moved
  slot, or a match on another function). Find the right value with methods 2 to 6. Do not change the check,
  except under the widening condition in hard rule 3: two independent pieces of evidence (Step 4) show that
  the value is right and that the old bound was too narrow for the new build. Write that evidence into the
  notes.
- `dependency failed`: see Step 2.3.

### Method 2: `draft` on the old build, `scan` on the new build

```
mise run inspect -- disasm --exe <OLD> <old start> > <W>/old.asm 2>&1
mise run inspect -- draft --exe <OLD> <rva> --len 32 > <W>/draft-1.txt 2>&1
mise run inspect -- scan --exe <NEW> "<drafted pattern>" > <W>/scan-1.txt 2>&1
```

- Draft from the **old** function: at the old match of the broken step (the old trace gives the cursor), at
  the function start, and at 2 or 3 places in the body that look special: near a call, a string load or an
  unusual constant. Use instruction starts from `old.asm`.
- The draft must have `matches: 1` on the old exe. A `scan` with `matches: 1` on the new exe gives a
  candidate place; `inspect function --exe <NEW> <match>` gives its function.
- `matches: 0` on the new exe: make the pattern shorter, end it at a `call`, or draft at another place.
  `matches` more than 1: use `--len 48` or another place.
- `draft` wildcards immediates, displacements and branch offsets. Register bytes (REX, ModRM) stay. If a scan
  fails, try the pattern again with those bytes as `??` too.

### Method 3: strings, and the callers of entries that already resolve

- Strings: take the texts from the notes, from `string_xref` steps of the old recipes, or from the old
  decompile. `inspect xrefs --exe <OLD> --string "<text>"` must list the old function; then
  `inspect xrefs --exe <NEW> --string "<text>"` lists the new candidates.
- Calls: entries that resolve on both builds (in `<OLDDATA>` and in `<R>/accepted.hcl`) are fixed points. On
  the old build, `inspect callers --exe <OLD> <old rva>` gives the functions that call the old function. If
  one of them is a resolved entry, or a function you can find with method 2, `disasm` it on both builds and
  find the call at the same place in the call order. The target of that call on the new build is a candidate.
- `inspect xrefs --rva <vtable>` on each build gives the code that loads a resolved vtable global.

### Method 4: `similar` from the old value

```
mise run inspect -- similar --exe <NEW> --old-exe <OLD> <old rva> --top 10 > <W>/similar.txt 2>&1
```

This is weak. On a real build change it ranked the right function first for only 4 of 15 entries. The first
line shows what the fingerprint holds (`strings`, `imports`, `constants`, `calls`). If it holds only
constants, ignore the scores: unrelated functions score 1.000. A `similar` result is never evidence. Use its
list only to name candidates for methods 2, 3, 5 and 6.

### Method 5: the vtable, slot by slot, near the old slot

For a `vslot` entry, or a function that sits in a vtable. The vtable global must resolve on the new build
first (it is an entry). List a window around the old slot on both builds:

```
mise run inspect -- vtable --exe <OLD> --rva <old vtable> --from <old slot - 8> --to <old slot + 9> > <W>/vtable-old.txt 2>&1
mise run inspect -- vtable --exe <NEW> --rva <new vtable> --from <old slot - 8> --to <old slot + 9> > <W>/vtable-new.txt 2>&1
```

For a packet handler slot, `inspect packets` gives the slot (Method 0). For other vtables, run
`inspect vtable --names` on both windows first: a slot that loads its own `virtual` `__FUNCSIG__` text is named
on both builds, and it is a fixed point to line the others up.

Slots move between builds: an interface gains or loses a virtual before the slot. Never trust the old slot
number, and a slot that holds a function start proves nothing. Line the two lists up by their neighbours:
draft the bodies of the old neighbours (method 2) and find which new slots hold them. The new slot is where
the old neighbours say it is, and the old function's body must also match the function in that slot.

### Method 6: `decompile` (fallback)

Only when methods 1 to 5 leave no candidate, or more than one. Decompile the old function and each candidate.
Make the folder `<W>` first; `--out` is one file:

```
mise run decompile -- --exe <OLD> --rva <old start> --out <ROOT>/<W>/old.c > <W>/decompile-old.txt 2>&1
mise run decompile -- --exe <NEW> --rva <candidate> --out <ROOT>/<W>/cand-1.c > <W>/decompile-cand-1.txt 2>&1
```

Compare the strings, the calls, the constants, the loops and the order of the tests. Names are generic
(`FUN_...`); compare the shape, not the names.

`decompile` needs a function in `.pdata` (`inspect function` prints a `primary` range). A leaf without
`.pdata` cannot be decompiled, and `inspect disasm` refuses it too ("not in a function"). Compare its callers
instead: `inspect callers` of the old leaf and of each candidate, then `inspect disasm` of the call site in each
caller (the callers are in `.pdata`). A function that became a leaf without `.pdata` fails `function_start`; see "Leaf
thunks without `.pdata`" in `docs/recipes.md`: a leaf that is called directly takes the `leaf_start` check.

## Step 4: Accept a candidate

Accept a candidate only with **two independent pieces of evidence**. Each piece must start from the **old
function**, as found on the old build with `<OLDDATA>`, and reach the candidate on the new build. Examples:

- a unique pattern drafted from the old body, which matches once on the new build, in the candidate;
- a string that the old function loads, and that the candidate loads (and few other functions);
- the call from a resolved entry: the old function is called at a place in that entry's old body, and the
  candidate is called at the same place in its new body;
- the vtable neighbours of the old slot, found again around the candidate's slot.

These are **not** evidence:

- a value that a recipe gives on the new build (a recipe cannot confirm itself);
- a pattern or recipe drafted at the candidate on the new build;
- a vtable slot, shifted or not, that lands on a function start;
- a `similar` score;
- two patterns from the same place in the old code (they count as one piece).

Write both pieces into the entry's `notes`.

## Step 5: Write the recipe

Follow `docs/recipes.md`. Edit only the block of this entry (from `entry "<id>" {` to its closing `}`) in the
recipe file that the job names. Keep the line endings of the file: some recipe files use CRLF, others LF.
Check with `file <path>` (bash; it says "with CRLF line terminators" for CRLF) or
`(Get-Content -Raw <path>).Contains("`r`n")` (PowerShell; `True` for CRLF).

- **Patterns.** Start from the `draft` output of the old build. Wildcard every immediate, displacement and
  branch byte (keep one only with a `literal_ok` reason). Wildcard register bytes when the register can change.
- **End a pattern at a `call`.** The compiler can put a `nop` after a CFG indirect call (`call [guard]`), so a
  pattern that runs past a call breaks.
- **No fixed distance to the start.** Never use `offset = -N` to reach a function start. Use
  `function_start_of`, or `ref` the entry and an in-function `pattern` step for the prologue. End a prologue
  pattern at the pushes, before the frame size.
- **No fixed slots.** No `vtable_slot` route for a function entry. For a slot, read a call-site displacement
  (`operand` + `displacement_to_slot`), or `ref` the body and `slot_of` the vtable. Give it an `in_vtable` or
  `range` check.
- **Every recipe must succeed on both builds.** `lint` reports a recipe that does not run as an open finding.
  When the code shape changed for real, use one `pattern_any` with one alternative per shape. Do not write one
  recipe per build.
- **Two independent recipes** when the entry has only weak checks (`section` only, `range` only, or every
  recipe ends with `function_start_of` and there is no `in_vtable`). A `pattern_any` is **one** recipe; its
  alternatives are not independent. The second recipe must anchor elsewhere. See "Two independent recipes"
  in `docs/recipes.md`.
- **A recipe that gives a wrong value** on the new build (a fixed slot, a fixed offset) must be changed or
  removed. Write the reason in the notes.
- **A helper entry** (for example a new vtable global from a constructor anchor, as in "New vtable globals" in
  `docs/recipes.md`) is allowed when the repair needs it. Put it in the same recipe file. It obeys every rule
  here: a strong check or two independent recipes, it derives on both builds, its lint section has no open
  finding, and its notes say what it is and which entry needs it. It counts as part of this entry's repair. On
  the old build it adds 1 to N in `derived N/N`. Remove it again if the entry is handed off.
- **Notes.** Rewrite the entry's `notes` so that they are true for both builds: what the entry is, each recipe
  (what it anchors on, what is wildcarded or kept), the checks, and "What changed in `<NEW_ID>`" with the
  class. Add the two pieces of evidence. Number recipes from 0. Do not put an RVA in the notes.

## Step 6: Verify each entry

Read the clock and write it into the ledger before each attempt. If 20 minutes have passed since the entry's
start, do not start the attempt; go to Step 7. An attempt that fails one point below is a failed attempt
(Step 7 counts them).

1. **Old build, derive twice** (see "Derive twice"):
   `mise run derive -- --exe <OLD> --recipes recipes --out <R>/verify/old-1 --report <R>/verify/old-1 --compare <OLDDATA> > <R>/verify/old-1.txt 2>&1`,
   then the same into `old-2`. Each report `<R>/verify/old-1/<OLD_ID>.md` must say `derived N/N` (N grows by
   1 for each helper entry you added) and `deterministic: yes`, and must have **no** `## Moved` section (no
   value moved against the old data). Its `## Unverified` section must list no entry that the baseline report
   did not list.
2. **New build, derive twice:** the same commands with `--exe <NEW>`, into `<R>/verify/new-1` and `new-2`.
   On the new build, `## Moved` is expected (values move between builds), and other entries may still fail.
   Check these points:
   - Each report says `deterministic: yes`, and the two data files are identical.
   - The entry resolves to the accepted candidate: open `<R>/verify/new-1/<NEW_ID>.hcl`, find the block
     `resolved "<id>" {`, and read its `value`. It must equal the candidate (for a `vslot`, the slot in hex).
     `mise run inspect -- explain --exe <NEW> <id> --recipes recipes` must show every recipe with that result.
   - No other value changed: run `git diff --no-index <R>/accepted.hcl <R>/verify/new-1/<NEW_ID>.hcl` and read
     the full output. Allowed: a new `resolved` block for this entry, its helper entries and its dependents, and
     changed `recipes =` lines of those entries. Not allowed: any changed `value` line of another entry, or a
     removed block.
   - The `## Failures` table of the new report lists no entry that `<R>/accepted.hcl` holds (entries that
     are not in `accepted.hcl` can still fail; this point covers accepted entries only), and the
     `## Unverified` section lists no entry that the cold report did not list.
3. **Lint on both exes.**
   `mise run lint -- --exe <OLD> --recipes recipes --report <R>/verify/lint-old.md > <R>/verify/lint-old.txt 2>&1`,
   and the same for `<NEW>` into `lint-new.md`. In each report, the section `## <id> (<kind>)` of the entry
   (and of each helper entry) must have no open finding: no fixed byte, no `did not run`, no
   `literal vtable slot`, no `not decodable`, no narrow range. Other entries can still have findings while
   their jobs are open.
4. When all of this passes, copy `<R>/verify/new-1/<NEW_ID>.hcl` to `<R>/accepted.hcl`. Read the clock. Write
   into the ledger: the end time, the minutes, the cause, the method that found the candidate, the two pieces
   of evidence, the number of attempts and the class: "would have survived with a better recipe" or "real
   code change".

## Step 7: Stop rule

**Per entry.** Stop work on the entry when one of these is true:

- 20 minutes have passed since the entry's start time (check the clock before each method and each attempt);
- 3 recipe attempts failed Step 6;
- 2 or more candidates remain and the evidence cannot tell them apart.

Then restore the entry so that it fails safely and gives no guess:

1. In the recipe file, replace only this entry's block with the block from `<W>/before.hcl`. Do not copy the
   whole file back: other entries in the file can hold accepted repairs. Remove any helper entry that you
   added for this entry.
2. Derive the new build once into `<R>/verify/handoff` and compare it with `<R>/accepted.hcl` as in Step 6.2:
   no other value changed.
3. Mark the entry "handed off" in the ledger with the end time, the candidates, the evidence for and against
   each, and what you tried. Then go to the next job.

**Whole run.** Stop when every job is repaired or handed off. Never weaken a check or a deriver rule, never
write an address, and never change mod code to get an entry through.

## Step 8: End

1. `mise run check > <R>/check.txt 2>&1` and `mise run test > <R>/test.txt 2>&1`. Both must exit 0. If one
   fails, do not change code: write the failing lines into the ledger and tell the user.
2. Final derive twice of each build (the Step 6.1 and 6.2 commands) into `<R>/final/old-1`, `old-2`, `new-1`,
   `new-2`. The old build must pass Step 6.1. Write the derived counts and "identical" or "differ" into the
   ledger.
3. `mise run signature -- --exe <NEW> --compare <R>/signature-old.hcl > <R>/signature-end.txt 2>&1`. Write
   each `changed` line into "Mod-code work". An `unknown` line now means a hooked entry was handed off.
4. Lint on both exes into `<R>/lint-end-old.md` and `<R>/lint-end-new.md`. No entry that this run changed may
   have an open finding. List the other entries with findings in the ledger section "Lint findings not in
   scope", and say whether each was already in `lint-start-*.md`.
5. Read the clock. Write the ledger summary: the end time, repaired count, handed-off count, the total
   minutes, one row per entry with its minutes, method and class, and the signature changes for the mods.
6. Tell the user: the counts, the handed-off entries with their candidates, the mod-code work, and the ledger
   path.

## What broke before, and what survives

These causes broke recipes on an earlier build change. Look for them first.

| Cause | What to do |
| --- | --- |
| A `nop` after `call [guard]` | End patterns at the call |
| Frame size changed, or its encoding (`imm8` or `imm32`) | Wildcard the immediate; end prologue patterns at the pushes |
| Member offsets moved | Wildcard displacements |
| A fixed distance from an anchor to the start | `function_start_of`, or `ref` + in-function prologue `pattern` |
| Register allocation changed | Wildcard REX and ModRM bytes |
| Vtable slots moved (an interface lost or gained a virtual) | Call-site `displacement_to_slot` or `slot_of`; an `in_vtable` window |
| Bases added or removed (store offsets in constructors moved) | Wildcard the store displacements and registers |
| Asserts removed, push list changed, a body became a leaf | Real code change: anchor after the change, or `pattern_any` with one alternative per shape. A function that became a leaf without `.pdata` and is called directly: `leaf_start` instead of `function_start` (this is the stronger check for a leaf, not a weaker one) |
| A twin function (the same body twice) | An anchor outside the body, an `in_vtable` window, or one kept literal with `literal_ok` |
| A function argument added | Real code change; `signature --compare` shows it; it is mod-code work |
| A member changed meaning (for example a pointer became a value held inline) | Real code change (layout): never give the old name the new meaning. New entry names per layout, an `entry_is` guard on a helper field that says which layout the build has; see "One name, one layout" in `docs/recipes.md`. The loader logs the other layout's entries as "not for this build" (INFO), not missing |

A cold derive refuses rather than guesses: a fixed slot or a fixed offset that moved gives a disagreement or
a refusal, not a wrong value. Keep it that way.

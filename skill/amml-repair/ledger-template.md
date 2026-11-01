# Repair ledger: <NEW_ID>

- new exe: <NEW>
- old exe: <OLD> (<OLD_ID>)
- old data: <OLDDATA> (given, or derived into <R>/baseline)
- start: <YYYY-MM-DD HH:MM>
- end: <YYYY-MM-DD HH:MM>

## Cold derive

- cold derive exit code: <0 | 3>; job index BuildId = <NEW_ID>: <yes | no>

- old build: derived <n>/<n>, deterministic, no value moved
- new build: derived <n>/<total>; failed <n> (<n> roots, <n> dependents); degraded <n>

| id | kind | failure (from the cold derive) |
| --- | --- | --- |

## Mod-code work

From `signature --compare`. This run does not change mod code.

| entry | start | end | line |
| --- | --- | --- | --- |

## Entries

One block per job, in the order of the job index. Copy this block for each entry.

### <id>

- start: <HH:MM>
- end: <HH:MM>
- minutes: <n>
- result: repaired | handed off | handed off: blocked by <root> | fixed by its root
- failure: <the reason from the job file>
- cause: <what changed in the code, and which recipe step it broke>
- method that found it: <1 explain | 2 draft + scan | 3 strings / callers | 4 similar | 5 vtable | 6 decompile>
- evidence 1: <...>
- evidence 2: <...>
- recipe attempts: <n>
  - attempt 1 at <HH:MM>: <passed, or the Step 6 point it failed on>
- class: would have survived with a better recipe | real code change
- verify: old derive twice, identical, no value moved, no new unverified entry; new derive twice, identical,
  value = candidate, no other value changed against accepted.hcl; lint: no open finding for this entry on
  both exes

For a handed-off entry, also:

- candidates: <each candidate, with the evidence for and against it>
- tried: <the methods and recipes that were tried>
- restored: the entry block is the block from `work/<id>/before.hcl`; helper entries removed

## Degraded rows

Entries that resolve on the new build with a failed recipe. Not repaired in this run.

| id | recipe | error |
| --- | --- | --- |

## Lint findings not in scope

Open findings at the end in entries that this run did not change.

| id | finding | in lint-start too |
| --- | --- | --- |

## Summary

- repaired: <n>
- handed off: <n>
- fixed by their root: <n>
- total minutes: <n> (start to end)
- final derive: old <n>/<n> twice, identical; new <n>/<total> twice, identical
- `mise run check`: <pass | fail>; `mise run test`: <passed / skipped / failed>
- signature changes for the mods: <each changed line, or none>

| id | result | minutes | method | class |
| --- | --- | --- | --- | --- |

use crate::structs::{
    EntryComparison, Reg, Signature, SignatureComparison, SignatureFile, SignatureReport, SignatureStatus,
};

pub struct SignatureCompare;

impl SignatureCompare {
    /// Compares `new` against `old`, entry by entry in the order of `new`, then the entries only `old` has.
    pub fn compare(old: &SignatureFile, new: &SignatureFile) -> SignatureReport {
        let mut entries = Vec::new();
        for entry in &new.entries {
            let (comparison, returns) = match old.entries.iter().find(|other| other.id == entry.id) {
                Some(before) => (
                    Self::compare_one(&before.signature, &entry.signature),
                    Self::returns_note(&before.signature, &entry.signature),
                ),
                None => (SignatureComparison::Unknown("not in the other file".to_string()), None),
            };
            entries.push(EntryComparison::new(entry.id.clone(), comparison, returns));
        }
        for entry in &old.entries {
            if !new.entries.iter().any(|other| other.id == entry.id) {
                let comparison = SignatureComparison::Unknown("only in the other file".to_string());
                entries.push(EntryComparison::new(entry.id.clone(), comparison, None));
            }
        }
        SignatureReport::new(old.build, new.build, entries)
    }

    /// "unknown" whenever either side could not be read, even if both sides look alike. The return kind takes no
    /// part, see `returns_note`.
    pub fn compare_one(old: &Signature, new: &Signature) -> SignatureComparison {
        match (&old.status, &new.status) {
            (SignatureStatus::Unknown(reason), SignatureStatus::Unknown(_)) => {
                return SignatureComparison::Unknown(format!("both builds: {reason}"));
            }
            (SignatureStatus::Unknown(reason), _) => {
                return SignatureComparison::Unknown(format!("the other build: {reason}"));
            }
            (_, SignatureStatus::Unknown(reason)) => {
                return SignatureComparison::Unknown(format!("this build: {reason}"));
            }
            _ => {}
        }
        let mut changes = Self::register_changes(&old.args_int, &new.args_int);
        changes.extend(Self::register_changes(&old.args_xmm, &new.args_xmm));
        for slot in new.stack_args.iter().filter(|slot| !old.stack_args.contains(slot)) {
            changes.push(format!("+ stack {slot:#x}"));
        }
        for slot in old.stack_args.iter().filter(|slot| !new.stack_args.contains(slot)) {
            changes.push(format!("- stack {slot:#x}"));
        }
        if changes.is_empty() { SignatureComparison::Same } else { SignatureComparison::Changed(changes) }
    }

    /// The advisory return kind of two known signatures: `int`, or `int -> none` when they differ. A void function
    /// that uses eax as scratch on every path looks like int, and one that returns what a call returned looks like
    /// none, so it is a hint only.
    pub fn returns_note(old: &Signature, new: &Signature) -> Option<String> {
        if old.status != SignatureStatus::Known || new.status != SignatureStatus::Known {
            return None;
        }
        Some(if old.returns == new.returns {
            old.returns.name().to_string()
        } else {
            format!("{} -> {}", old.returns.name(), new.returns.name())
        })
    }

    fn register_changes(old: &[Reg], new: &[Reg]) -> Vec<String> {
        let mut changes = Vec::new();
        for reg in Reg::ARGUMENTS {
            let before = old.iter().find(|item| item.register == reg);
            let after = new.iter().find(|item| item.register == reg);
            match (before, after) {
                (None, Some(after)) => changes.push(format!("+ {}", Self::described(after))),
                (Some(before), None) => changes.push(format!("- {}", Self::described(before))),
                (Some(before), Some(after)) if before.bits != after.bits => {
                    changes.push(format!("{} {}-bit -> {}-bit", after.name(), before.bits, after.bits));
                }
                _ => {}
            }
        }
        changes
    }

    fn described(reg: &Reg) -> String {
        format!("{} ({}-bit)", reg.name(), reg.bits)
    }
}

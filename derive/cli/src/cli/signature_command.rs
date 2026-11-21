use std::path::Path;

use amml_data::{DataError, EntryId, EntryKind, RecipeLoader, RecipeSet};

use super::SignatureArgs;
use crate::errors::CliError;
use crate::services::{Deriver, ImageLoader, SignatureCompare, SignatureProbe, SignatureStore};
use crate::structs::{EntrySignature, Image, Signature, SignatureFile, SignatureReport};

pub struct SignatureCommand;

impl SignatureCommand {
    /// Derives the build, probes every hooked entry, writes the signatures to `--out` (or standard output when
    /// neither `--out` nor `--compare` is given) and, with `--compare`, compares them with that file.
    pub fn run(args: &SignatureArgs) -> Result<(SignatureFile, Option<SignatureReport>), CliError> {
        let set = RecipeLoader::load_dir(&args.recipes)?;
        let image = ImageLoader::from_path(&args.exe)?;
        let file = Self::probe(&image, &set)?;
        match &args.out {
            Some(path) => {
                SignatureStore::write(path, &file).map_err(|source| CliError::Write { path: path.clone(), source })?
            }
            None if args.compare.is_none() => print!("{}", SignatureStore::render(&file)),
            None => {}
        }
        let report = args.compare.as_deref().map(|other| Self::compare(&file, other)).transpose()?;
        Ok((file, report))
    }

    /// The signatures of the hooked entries in recipe order; an entry that did not derive is unknown. A hooked
    /// `vslot` is probed at the function in its slot of each vtable its `slot_of` steps name: the first row keeps
    /// the entry id, a later vtable gives a row `<id> (<vtable>)`.
    pub fn probe(image: &Image, set: &RecipeSet) -> Result<SignatureFile, DataError> {
        let outcome = Deriver::new(image, set).derive()?;
        let probe = SignatureProbe::new(image);
        let value = |id: &EntryId| outcome.data.entries.get(id).map(|resolved| resolved.value);
        let mut entries = Vec::new();
        for entry in set.entries().iter().filter(|entry| entry.hooked) {
            if entry.kind != EntryKind::VSlot {
                let rva = value(&entry.id).and_then(|value| u32::try_from(value).ok());
                entries.push(EntrySignature::new(entry.id.clone(), rva, Self::probe_at(&probe, rva)));
                continue;
            }
            let vtables = entry.slot_vtables();
            if vtables.is_empty() {
                let signature = Signature::unknown("a hooked vslot names no vtable in a slot_of step".to_string());
                entries.push(EntrySignature::new(entry.id.clone(), None, signature));
            }
            for (index, vtable) in vtables.into_iter().enumerate() {
                let id = if index == 0 { entry.id.clone() } else { EntryId(format!("{} ({})", entry.id.0, vtable.0)) };
                let rva = Self::slot_target(image, value(vtable), value(&entry.id));
                entries.push(EntrySignature::new(id, rva, Self::probe_at(&probe, rva)));
            }
        }
        Ok(SignatureFile::new(image.build(), entries))
    }

    // The function in `slot` of the vtable at `vtable` (both derived values); `None` when either did not derive or
    // the slot does not hold code.
    fn slot_target(image: &Image, vtable: Option<u64>, slot: Option<u64>) -> Option<u32> {
        let address = vtable?.checked_add(slot?.checked_mul(8)?)?;
        image.code_pointer(u32::try_from(address).ok()?)
    }

    fn probe_at(probe: &SignatureProbe, rva: Option<u32>) -> Signature {
        match rva {
            Some(rva) => probe.probe(rva),
            None => Signature::unknown("the entry did not derive".to_string()),
        }
    }

    /// Compares `current` against the signature file at `other`.
    pub fn compare(current: &SignatureFile, other: &Path) -> Result<SignatureReport, CliError> {
        let old = SignatureStore::read(other)?;
        Ok(SignatureCompare::compare(&old, current))
    }
}

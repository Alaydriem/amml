use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

use crate::error::GenError;
use crate::model::{Owner, OwnershipFile, TokenFile};
use crate::naming::TokenName;
use crate::resolve::{ResolvedToken, TokenResolver};
use crate::write::{FamilyWriter, OutputTree, SelectorWriter, TokenWriter};

use super::CliArgs;

/// Builds the amml-ui and amml-studio-ui token trees, and writes or checks them.
pub struct Generator;

impl Generator {
    /// The amml-ui tree and the amml-studio-ui tree, built in memory.
    pub fn build(file: &TokenFile, owners: &OwnershipFile) -> Result<(OutputTree, OutputTree), GenError> {
        let resolved = TokenResolver::resolve(file, owners)?;
        let by_name: BTreeMap<&str, &ResolvedToken> =
            resolved.iter().map(|token| (token.entry.name.as_str(), token)).collect();

        // Members of each folder, by crate and folder name: (module, type).
        let mut folders: BTreeMap<(Owner, String), Vec<(String, String)>> = BTreeMap::new();
        let mut trees: BTreeMap<Owner, OutputTree> =
            [(Owner::AmmlUi, OutputTree::default()), (Owner::StudioUi, OutputTree::default())].into();

        for token in &resolved {
            let family = TokenName::family(token.entry.kind).to_owned();
            let module = TokenWriter::module_name(&token.entry.name)?;
            let content = TokenWriter::render(token, &by_name)?;
            Self::tree(&mut trees, token.owner).insert(Path::new(&family).join(format!("{module}.rs")), content);
            folders
                .entry((token.owner, family))
                .or_default()
                .push((module, TokenWriter::struct_name(&token.entry.name)?));
        }

        for spec in &owners.selectors {
            let module = SelectorWriter::module_name(&spec.name);
            let content = SelectorWriter::render(spec, &by_name)?;
            Self::tree(&mut trees, spec.crate_owner)
                .insert(Path::new("selector").join(format!("{module}.rs")), content);
            folders.entry((spec.crate_owner, "selector".to_owned())).or_default().push((module, spec.name.clone()));
        }

        let mut roots: BTreeMap<Owner, Vec<String>> = BTreeMap::new();
        for ((owner, folder), members) in &folders {
            Self::tree(&mut trees, *owner).insert(Path::new(folder).join("mod.rs"), FamilyWriter::render(members));
            roots.entry(*owner).or_default().push(folder.clone());
        }
        for (owner, tree) in &mut trees {
            let families = roots.get(owner).cloned().unwrap_or_default();
            tree.insert(PathBuf::from("mod.rs"), FamilyWriter::render_root(&families));
        }

        let kit = trees.remove(&Owner::AmmlUi).unwrap_or_default();
        let studio = trees.remove(&Owner::StudioUi).unwrap_or_default();
        Ok((kit, studio))
    }

    /// Reads the inputs, builds both trees and, unless `args.check`, writes them. Returns the paths
    /// that changed (or would change), under their output folders. Nothing is written on an error.
    pub fn generate(args: &CliArgs) -> Result<Vec<PathBuf>, GenError> {
        let file: TokenFile = Self::read_json(&args.tokens)?;
        let owners: OwnershipFile = Self::read_json(&args.ownership)?;
        let (kit, studio) = Self::build(&file, &owners)?;
        let outputs = [(&kit, args.kit_out.as_path()), (&studio, args.studio_out.as_path())];

        let mut changed = BTreeSet::new();
        for (tree, root) in outputs {
            changed.extend(tree.diff(root)?.into_iter().map(|path| root.join(path)));
        }
        if !args.check {
            for (tree, root) in outputs {
                tree.write_to(root)?;
            }
        }
        Ok(changed.into_iter().collect())
    }

    fn tree(trees: &mut BTreeMap<Owner, OutputTree>, owner: Owner) -> &mut OutputTree {
        trees.entry(owner).or_default()
    }

    fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, GenError> {
        let text = fs::read_to_string(path).map_err(|source| GenError::Io { path: path.to_path_buf(), source })?;
        serde_json::from_str(&text).map_err(|source| GenError::Json { path: path.to_path_buf(), source })
    }
}

use std::fs;

use clap::Parser;

use crate::errors::RegistryError;
use crate::services::{
    AllowListReader, Clock, JsonText, ListingCheck, ManifestReader, RegistryUpdater, ReleaseSource, ReleaseVerifier,
    TagNamer, TreeBuilder, TreeStore,
};
use crate::structs::{
    AddAmmlArgs, AddModArgs, AllowList, AmmlSection, BuildTreeArgs, CliArgs, CliOutput, Command, LintManifestArgs,
    NextTagArgs, UpdateResult,
};

/// The `amml-registry` command line. `main` only parses the process arguments and calls `run`.
pub struct RegistryCli<S: ReleaseSource> {
    verifier: ReleaseVerifier<S>,
}

impl<S: ReleaseSource> RegistryCli<S> {
    const DEFAULT_NAME: &'static str = "Official";

    pub fn new(source: S) -> Self {
        Self { verifier: ReleaseVerifier::new(source) }
    }

    /// `args[0]` is the program name.
    pub fn run(&self, args: &[String]) -> CliOutput {
        let parsed = match CliArgs::try_parse_from(args) {
            Ok(parsed) => parsed,
            Err(error) if error.use_stderr() => return CliOutput::failure(error.to_string().trim_end()),
            Err(error) => return CliOutput::with_code(CliOutput::CHANGED, error.to_string()),
        };
        match self.dispatch(parsed.command) {
            Ok(output) => output,
            Err(error) => CliOutput::failure(&error.to_string()),
        }
    }

    fn dispatch(&self, command: Command) -> Result<CliOutput, RegistryError> {
        match command {
            Command::AddMod(args) => self.add_mod(args),
            Command::AddAmml(args) => self.add_amml(args),
            Command::NextTag(args) => Self::next_tag(args),
            Command::LintManifest(args) => Self::lint_manifest(args),
            Command::BuildTree(args) => Self::build_tree(args),
        }
    }

    fn add_mod(&self, args: AddModArgs) -> Result<CliOutput, RegistryError> {
        let now = args.now.unwrap_or_else(Clock::now);
        let allow = Self::read_allow(&args.allow)?;
        // the allow list is checked before any download, and its repo spelling is the one that is stored
        let repo = RegistryUpdater::authorize_mod(&allow, &args.repo, &args.namespace, &args.name)?.repo.clone();
        let store = TreeStore::new(args.tree);
        let mut document = store.load_document(&args.namespace, &args.name)?;
        let verified = self.verifier.check_mod(&repo, &args.namespace, &args.name, &args.version, &now)?;
        // a mod that needs a newer AMML than any listed one would be unusable for every user
        RegistryUpdater::check_min_amml(&verified.release, store.load_root()?.as_ref().map(|root| &root.amml))?;
        let (sha256, asset) = (verified.release.sha256.clone(), verified.release.asset.clone());
        let result = RegistryUpdater::add_mod(
            &mut document,
            &allow,
            &repo,
            &args.namespace,
            &args.name,
            verified.release,
            &verified.listing,
        )?;
        let id = format!("@{}/{}", args.namespace, args.name);
        if result == UpdateResult::Unchanged {
            return Ok(CliOutput::with_code(
                CliOutput::UNCHANGED,
                format!("{id} {} is already listed\n", args.version),
            ));
        }
        if let Some(document) = document {
            store.write_document(&document, &JsonText::pretty(&document)?)?;
            // only the newest release owns the listing, so an older backfill keeps the current icon
            if result == UpdateResult::Added {
                match &verified.icon {
                    Some(icon) => store.write_icon(&args.namespace, &args.name, icon)?,
                    None => store.remove_icon(&args.namespace, &args.name)?,
                }
            }
        }
        Self::rebuild(&store, &allow, Self::DEFAULT_NAME, Some(&now))?;
        Ok(CliOutput::with_code(
            CliOutput::CHANGED,
            format!("added {id} {}\nsha256 {sha256}\nasset {asset}\n", args.version),
        ))
    }

    fn add_amml(&self, args: AddAmmlArgs) -> Result<CliOutput, RegistryError> {
        let now = args.now.unwrap_or_else(Clock::now);
        let allow = Self::read_allow(&args.allow)?;
        let store = TreeStore::new(args.tree);
        let mut section = match store.load_root()? {
            Some(root) => root.amml,
            None => AmmlSection { repo: allow.amml_repo.clone(), versions: Vec::new() },
        };
        let repo = RegistryUpdater::authorize_amml(&allow, &args.repo)?.to_string();
        let release = self.verifier.check_amml(&repo, &args.version, args.api_version, &args.game_versions)?;
        let (sha256, asset) = (release.sha256.clone(), release.asset.clone());
        let result = RegistryUpdater::add_amml(&mut section, &allow, &repo, release)?;
        if result == UpdateResult::Unchanged {
            return Ok(CliOutput::with_code(
                CliOutput::UNCHANGED,
                format!("amml {} is already listed\n", args.version),
            ));
        }
        let name = store.load_root()?.map(|root| root.name).unwrap_or_else(|| Self::DEFAULT_NAME.to_string());
        let documents = store.load_documents()?;
        let tree = TreeBuilder::build(&name, &now, &section, &documents)?;
        store.write_tree(&tree)?;
        Ok(CliOutput::with_code(
            CliOutput::CHANGED,
            format!("added amml {}\nsha256 {sha256}\nasset {asset}\n", args.version),
        ))
    }

    fn next_tag(args: NextTagArgs) -> Result<CliOutput, RegistryError> {
        let tag = TagNamer::next(&args.date, &args.existing)?;
        Ok(CliOutput::with_code(CliOutput::CHANGED, format!("{tag}\n")))
    }

    fn lint_manifest(args: LintManifestArgs) -> Result<CliOutput, RegistryError> {
        let text = fs::read_to_string(&args.manifest)?;
        let manifest = ManifestReader::parse(&text)?;
        ManifestReader::check_floor(&manifest)?;
        let listing = manifest.listing.ok_or_else(|| RegistryError::Listing {
            field: "listing",
            reason: "the amml.hcl has no listing block".to_string(),
        })?;
        ListingCheck::check(&listing)?;
        for (field, expected, got) in
            [("namespace", &args.namespace, &manifest.namespace), ("name", &args.name, &manifest.name)]
        {
            if let Some(expected) = expected
                && expected != got
            {
                return Err(RegistryError::ManifestMismatch { field, expected: expected.clone(), got: got.clone() });
            }
        }
        if let Some(version) = &args.version
            && &manifest.version != version
        {
            return Err(RegistryError::ManifestMismatch {
                field: "version",
                expected: version.clone(),
                got: manifest.version,
            });
        }
        if let Some(icon) = &listing.icon {
            let folder = args.manifest.parent().map(|parent| parent.to_path_buf()).unwrap_or_default();
            let bytes = fs::read(folder.join(icon)).map_err(|_| RegistryError::Listing {
                field: "icon",
                reason: format!("`{icon}` is not in the mod folder"),
            })?;
            ListingCheck::check_icon(&bytes)?;
        }
        Ok(CliOutput::with_code(CliOutput::CHANGED, format!("ok: @{}/{}\n", manifest.namespace, manifest.name)))
    }

    fn build_tree(args: BuildTreeArgs) -> Result<CliOutput, RegistryError> {
        let store = TreeStore::new(args.tree);
        let allow = match &args.allow {
            Some(path) => Some(Self::read_allow(path)?),
            None => None,
        };
        let allow = allow.unwrap_or(AllowList { amml_repo: String::new(), mods: Vec::new() });
        Self::rebuild(&store, &allow, &args.name, args.published.as_deref())?;
        Ok(CliOutput::with_code(CliOutput::CHANGED, "tree rebuilt\n".to_string()))
    }

    // regenerates the derived files; `published` replaces the old time only when given
    fn rebuild(store: &TreeStore, allow: &AllowList, name: &str, published: Option<&str>) -> Result<(), RegistryError> {
        let root = store.load_root()?;
        let (name, old_published, amml) = match root {
            Some(root) => (root.name, root.published, root.amml),
            None => (
                name.to_string(),
                published.map(str::to_string).unwrap_or_else(Clock::now),
                AmmlSection { repo: allow.amml_repo.clone(), versions: Vec::new() },
            ),
        };
        let mut documents = store.load_documents()?;
        for document in &mut documents {
            RegistryUpdater::sync_policy(document, allow);
            store.write_document(document, &JsonText::pretty(document)?)?;
        }
        let tree = TreeBuilder::build(&name, published.unwrap_or(&old_published), &amml, &documents)?;
        store.write_tree(&tree)
    }

    fn read_allow(path: &std::path::Path) -> Result<AllowList, RegistryError> {
        AllowListReader::parse(&fs::read_to_string(path)?)
    }
}

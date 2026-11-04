use hcl::{Block, Body, Expression};

use crate::errors::RegistryError;
use crate::structs::{AllowList, AllowedMod};

/// Reads `mods.hcl`.
pub struct AllowListReader;

impl AllowListReader {
    pub fn parse(text: &str) -> Result<AllowList, RegistryError> {
        let body = hcl::parse(text).map_err(|error| Self::bad(format!("not valid HCL: {error}")))?;
        if let Some(attribute) = body.attributes().next() {
            return Err(Self::bad(format!("unexpected attribute `{}`", attribute.key())));
        }
        let mut amml_repo = None;
        let mut mods: Vec<AllowedMod> = Vec::new();
        for block in body.blocks() {
            match block.identifier() {
                "amml" => {
                    if amml_repo.is_some() {
                        return Err(Self::bad("`amml` appears twice".to_string()));
                    }
                    amml_repo = Some(Self::amml_repo(block)?);
                }
                "mod" => {
                    let entry = Self::allowed_mod(block)?;
                    if mods.iter().any(|known| known.namespace == entry.namespace && known.name == entry.name) {
                        return Err(Self::bad(format!("{} appears twice", entry.id())));
                    }
                    mods.push(entry);
                }
                other => return Err(Self::bad(format!("unknown block `{other}`"))),
            }
        }
        let amml_repo = amml_repo.ok_or_else(|| Self::bad("the `amml` block is missing".to_string()))?;
        Ok(AllowList { amml_repo, mods })
    }

    fn amml_repo(block: &Block) -> Result<String, RegistryError> {
        if !block.labels().is_empty() {
            return Err(Self::bad("`amml` takes no labels".to_string()));
        }
        let mut repo = None;
        for attribute in block.body().attributes() {
            match attribute.key() {
                "repo" => repo = Some(Self::text(attribute.key(), attribute.expr())?),
                other => return Err(Self::bad(format!("unknown key `{other}` in `amml`"))),
            }
        }
        repo.ok_or_else(|| Self::bad("`amml` needs `repo`".to_string()))
    }

    fn allowed_mod(block: &Block) -> Result<AllowedMod, RegistryError> {
        let labels = block.labels();
        if labels.len() != 2 {
            return Err(Self::bad("`mod` takes two labels: namespace and name".to_string()));
        }
        let namespace = labels[0].as_str().to_string();
        let name = labels[1].as_str().to_string();
        let mut repo = None;
        let mut core = false;
        let mut default = false;
        for attribute in Self::attributes(block.body()) {
            match attribute.key() {
                "repo" => repo = Some(Self::text("repo", attribute.expr())?),
                "core" => core = Self::flag("core", attribute.expr())?,
                "default" => default = Self::flag("default", attribute.expr())?,
                other => return Err(Self::bad(format!("unknown key `{other}` in mod @{namespace}/{name}"))),
            }
        }
        let repo = repo.ok_or_else(|| Self::bad(format!("mod @{namespace}/{name} needs `repo`")))?;
        Ok(AllowedMod { namespace, name, repo, core, default: default || core })
    }

    fn attributes(body: &Body) -> impl Iterator<Item = &hcl::Attribute> {
        body.attributes()
    }

    fn text(key: &str, expression: &Expression) -> Result<String, RegistryError> {
        match expression {
            Expression::String(text) => Ok(text.clone()),
            _ => Err(Self::bad(format!("`{key}` must be a string"))),
        }
    }

    fn flag(key: &str, expression: &Expression) -> Result<bool, RegistryError> {
        match expression {
            Expression::Bool(value) => Ok(*value),
            _ => Err(Self::bad(format!("`{key}` must be true or false"))),
        }
    }

    fn bad(message: String) -> RegistryError {
        RegistryError::BadAllowList(message)
    }
}

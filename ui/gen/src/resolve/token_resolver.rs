use std::collections::{BTreeMap, BTreeSet};

use crate::error::GenError;
use crate::model::{Owner, OwnershipFile, TokenFile};
use crate::naming::TokenName;
use crate::value::{TokenValue, ValueParser};

use super::ResolvedToken;

/// Checks every token and gives each its owner and parsed value.
pub struct TokenResolver;

impl TokenResolver {
    /// The tokens in file order. Fails on the first bad token, after listing every unowned one.
    pub fn resolve(file: &TokenFile, owners: &OwnershipFile) -> Result<Vec<ResolvedToken>, GenError> {
        let mut unowned: Vec<String> = file
            .tokens
            .iter()
            .filter(|entry| !owners.owners.contains_key(&entry.name))
            .map(|entry| entry.name.clone())
            .collect();
        if !unowned.is_empty() {
            unowned.sort();
            unowned.dedup();
            return Err(GenError::Unowned { tokens: unowned });
        }

        let mut resolved = Vec::with_capacity(file.tokens.len());
        let mut seen = BTreeSet::new();
        for entry in &file.tokens {
            let fits = TokenName::split(&entry.name)
                .is_some_and(|(kind, rest)| kind == entry.kind && TokenName::is_identifier(&rest));
            if !fits || !seen.insert(entry.name.as_str()) {
                return Err(GenError::BadName { token: entry.name.clone() });
            }
            let owner = owners
                .owners
                .get(&entry.name)
                .copied()
                .ok_or_else(|| GenError::Unowned { tokens: vec![entry.name.clone()] })?;
            resolved.push(ResolvedToken { entry: entry.clone(), owner, value: ValueParser::parse_entry(entry)? });
        }

        let by_name: BTreeMap<&str, &ResolvedToken> =
            resolved.iter().map(|token| (token.entry.name.as_str(), token)).collect();
        for token in &resolved {
            Self::check_alias(token, &by_name)?;
        }
        Ok(resolved)
    }

    fn check_alias(token: &ResolvedToken, by_name: &BTreeMap<&str, &ResolvedToken>) -> Result<(), GenError> {
        let TokenValue::Alias(target_name) = &token.value else {
            return Ok(());
        };
        let name = &token.entry.name;
        let target = by_name
            .get(target_name.as_str())
            .filter(|target| target.entry.kind == token.entry.kind)
            .ok_or_else(|| GenError::MissingAlias { token: name.clone(), target: target_name.clone() })?;
        if token.owner == Owner::AmmlUi && target.owner == Owner::StudioUi {
            return Err(GenError::WrongDirectionAlias { token: name.clone(), target: target_name.clone() });
        }
        // Follow the chain: a loop would generate consts that refer to each other.
        let mut steps = 0;
        let mut at = *target;
        while let TokenValue::Alias(next) = &at.value {
            steps += 1;
            if steps > by_name.len() {
                return Err(GenError::AliasCycle { token: name.clone() });
            }
            match by_name.get(next.as_str()) {
                Some(found) => at = found,
                None => break,
            }
        }
        Ok(())
    }
}

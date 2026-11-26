//! Logical content dependencies, ordered for a client or dedicated-server installation.
use crate::{Manifest, Side, manifest::valid_id};
use semver::{Version, VersionReq};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Dependency {
    Version(VersionReq),
    Options(Options),
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Options {
    pub version: VersionReq,
    #[serde(default)]
    pub optional: bool,
    #[serde(default = "both")]
    pub side: Side,
    #[serde(default)]
    pub order: Order,
    pub reason: Option<String>,
    pub url: Option<String>,
}

fn both() -> Side {
    Side::Both
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Order {
    #[default]
    After,
    Before,
    None,
}

impl Dependency {
    pub fn applies_to(&self, installation: Side) -> bool {
        self.fields().2.includes(installation)
    }

    pub(crate) fn check(&self, id: &str) -> Result<(), String> {
        if !valid_id(id) {
            return Err(format!(
                "horizon-mod.toml: invalid dependency mod id {id:?}"
            ));
        }
        if let Self::Options(options) = self {
            if options
                .reason
                .as_ref()
                .is_some_and(|reason| reason.len() > 512)
            {
                return Err(format!(
                    "horizon-mod.toml: dependency {id} reason is over 512 bytes"
                ));
            }
            if options
                .url
                .as_ref()
                .is_some_and(|url| !url.starts_with("https://"))
            {
                return Err(format!(
                    "horizon-mod.toml: dependency {id} URL must start with https://"
                ));
            }
        }
        Ok(())
    }

    fn fields(&self) -> (&VersionReq, bool, Side, Order) {
        match self {
            Self::Version(version) => (version, false, Side::Both, Order::After),
            Self::Options(row) => (&row.version, row.optional, row.side, row.order),
        }
    }
}

/// Order the enabled, readable packages for this installation. Returned indices
/// refer to the input; packages and dependency rows exclusive to the other side
/// are omitted before testing requirements. Ties retain input order.
pub fn resolve(mods: &[&Manifest], side: Side) -> Result<Vec<usize>, String> {
    resolve_for(mods, side, |manifest| manifest.side.includes(side))
}

/// Order content as Offline loads it, including legacy content on dedicated servers.
pub fn resolve_content(mods: &[&Manifest], side: Side) -> Result<Vec<usize>, String> {
    resolve_for(mods, side, |manifest| manifest.content_for(side))
}

fn resolve_for(
    mods: &[&Manifest],
    side: Side,
    includes: impl Fn(&Manifest) -> bool,
) -> Result<Vec<usize>, String> {
    let mut ids = BTreeMap::new();
    let mut active = Vec::new();
    for (index, item) in mods.iter().enumerate().filter(|(_, m)| includes(m)) {
        if ids.insert(item.id.as_str(), index).is_some() {
            return Err(format!("duplicate mod id {}", item.id));
        }
        active.push(index);
    }

    let mut predecessors = vec![Vec::new(); mods.len()];
    let mut successors = vec![Vec::new(); mods.len()];
    for &index in &active {
        let item = mods[index];
        for (id, dependency) in &item.dependencies {
            let (requirement, optional, _, order) = dependency.fields();
            if !dependency.applies_to(side) {
                continue;
            }
            let Some(&target) = ids.get(id.as_str()) else {
                if optional {
                    continue;
                }
                return Err(format!(
                    "{} requires missing mod {id} ({requirement})",
                    item.id
                ));
            };
            let version = &mods[target].version;
            let found = Version::parse(version)
                .map_err(|e| format!("{id} has invalid mod version {version:?}: {e}"))?;
            if !requirement.matches(&found) {
                return Err(format!(
                    "{} requires {id} {requirement}, installed version is {version}",
                    item.id
                ));
            }
            let (from, to) = match order {
                Order::After => (target, index),
                Order::Before => (index, target),
                Order::None => continue,
            };
            predecessors[to].push(from);
            successors[from].push(to);
        }
    }

    let mut pending: Vec<_> = predecessors.iter().map(Vec::len).collect();
    let mut ready: BTreeSet<_> = active
        .iter()
        .copied()
        .filter(|&idx| pending[idx] == 0)
        .collect();
    let mut result = Vec::with_capacity(active.len());
    while let Some(index) = ready.pop_first() {
        result.push(index);
        for &next in &successors[index] {
            pending[next] -= 1;
            if pending[next] == 0 {
                ready.insert(next);
            }
        }
    }
    if let Some(mut current) = active.into_iter().find(|&idx| pending[idx] != 0) {
        let mut seen = BTreeMap::new();
        let mut path = Vec::new();
        while !seen.contains_key(&current) {
            seen.insert(current, path.len());
            path.push(mods[current].id.as_str());
            current = *predecessors[current]
                .iter()
                .find(|&&idx| pending[idx] != 0)
                .expect("pending predecessor");
        }
        let mut cycle = path[seen[&current]..].to_vec();
        cycle.reverse();
        cycle.push(cycle[0]);
        return Err(format!("mod dependency cycle: {}", cycle.join(" -> ")));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(id: &str, version: &str, side: &str, dependencies: &str) -> Manifest {
        Manifest::parse(&format!(
            "id = '{id}'\nname = '{id}'\nversion = '{version}'\napi = '0.2'\nside = '{side}'\ncapabilities = ['registries']\n[dependencies]\n{dependencies}"
        ))
        .unwrap()
    }

    #[test]
    fn orders_the_required_mod_first_and_allows_an_absent_optional_mod() {
        let extension = manifest(
            "extension",
            "1.0.0",
            "both",
            "base = '^1.2'\nextra = { version = '^2', optional = true }",
        );
        let base = manifest("base", "1.2.3", "both", "");
        let unrelated = manifest("neighbor", "1.0.0", "both", "");
        assert_eq!(
            resolve(&[&extension, &base, &unrelated], Side::Client),
            Ok(vec![1, 0, 2])
        );
        assert!(
            resolve(&[&extension, &unrelated], Side::Client)
                .unwrap_err()
                .contains("requires missing mod base")
        );
    }

    #[test]
    fn checks_installed_optional_dependencies_for_version() {
        let base = manifest(
            "base",
            "1.2.3",
            "both",
            "extra = { version = '^2', optional = true }",
        );
        let extra = manifest("extra", "3.0.0", "both", "");
        assert!(
            resolve(&[&base, &extra], Side::Client)
                .unwrap_err()
                .contains("installed version is 3.0.0")
        );
    }

    #[test]
    fn filters_packages_and_requirements_before_resolving_the_server() {
        let base = manifest(
            "base",
            "1.0.0",
            "both",
            "extra = { version = '^2', side = 'client' }",
        );
        let extra = manifest("extra", "2.0.0", "client", "");
        assert_eq!(resolve(&[&base, &extra], Side::Server), Ok(vec![0]));
        assert_eq!(resolve(&[&base, &extra], Side::Client), Ok(vec![1, 0]));
    }

    #[test]
    fn keeps_legacy_content_on_a_dedicated_server_when_a_mod_declares_dependencies() {
        let extension = manifest("extension", "1.0.0", "both", "base = '^2'");
        let legacy = manifest("base", "2.0.0", "client", "");
        assert_eq!(
            resolve_content(&[&extension, &legacy], Side::Server),
            Ok(vec![1, 0])
        );
    }

    #[test]
    fn checks_explicit_order_and_names_cycles() {
        let base = manifest(
            "base",
            "1.0.0",
            "both",
            "extension = { version = '^1', order = 'before' }",
        );
        let extension = manifest(
            "extension",
            "1.0.0",
            "both",
            "base = { version = '^1', order = 'none' }",
        );
        assert_eq!(resolve(&[&extension, &base], Side::Client), Ok(vec![1, 0]));
        let reversed = manifest(
            "extension",
            "1.0.0",
            "both",
            "base = { version = '^1', order = 'before' }",
        );
        let error = resolve(&[&base, &reversed], Side::Client).unwrap_err();
        assert!(error.contains("mod dependency cycle:"));
        assert!(error.contains("base"));
        assert!(error.contains("extension"));
    }
}

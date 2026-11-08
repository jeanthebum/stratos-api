//! Public shader API versions are independent of the mod and of any shader pack.
use semver::{Version, VersionReq};

/// Implementation release. Updating it does not change the shader contract.
pub const IMPLEMENTATION_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Contracts actually implemented by this renderer, not all versions below a maximum.
pub const SUPPORTED_APIS: &[&str] = &["0.1.0"];
/// Frame0_2's ABI for StratosAPI 0.1; additions require a distinct contract.
pub const FRAME_BYTES: usize = 816;

/// Frozen stage signatures and uniform layout for the public 0.1.0 contract.
pub const ABI_SNAPSHOT: &str = include_str!("../data/contracts/0.1.0.json");

/// Negotiate the highest explicitly implemented version accepted by the pack.
pub fn negotiate(requirement: &str, supported: &[&str]) -> Result<String, String> {
    if requirement.trim().is_empty() {
        return Err("stratos.api must declare a supported version range".into());
    }
    let wanted = VersionReq::parse(requirement)
        .map_err(|e| format!("invalid stratos.api requirement {requirement:?}: {e}"))?;
    let mut versions = Vec::new();
    for text in supported {
        let version = Version::parse(text)
            .map_err(|e| format!("invalid renderer API version {text:?}: {e}"))?;
        if wanted.matches(&version) {
            versions.push(version);
        }
    }
    versions.into_iter().max().map(|v| v.to_string()).ok_or_else(|| {
        format!(
            "this shader requires StratosAPI {requirement}; renderer {} implements [{}]. Install a compatible StratosAPI mod or choose a compatible shader pack",
            IMPLEMENTATION_VERSION, supported.join(", ")
        )
    })
}

pub fn resolve(requirement: &str) -> Result<String, String> {
    negotiate(requirement, SUPPORTED_APIS)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_caret_and_bounded_ranges() {
        for r in ["=0.1.0", "^0.1.0", ">=0.1.0, <0.2.0"] {
            assert_eq!(resolve(r).unwrap(), "0.1.0");
        }
    }
    #[test]
    fn rejects_unknown_future_old_and_invalid_contracts() {
        for r in ["", "potato", ">=0.2.0", "=0.0.9", "=0.1.1"] {
            assert!(resolve(r).is_err(), "{r}");
        }
        let e = resolve("^0.2.0").unwrap_err();
        assert!(e.contains("requires StratosAPI ^0.2.0") && e.contains("[0.1.0]"));
    }
    #[test]
    fn an_engine_upgrade_can_serve_several_old_contracts() {
        assert_eq!(negotiate(">=0.1.0, <0.2.0", &["0.2.0", "0.1.0", "0.1.1"]).unwrap(), "0.1.1");
        assert_eq!(negotiate("=0.1.0", &["0.2.0", "0.1.0"]).unwrap(), "0.1.0");
        assert!(negotiate("=0.1.0", &["0.2.0"]).is_err());
    }
    #[test]
    fn prereleases_require_explicit_consent() {
        assert!(negotiate("^0.1.0", &["0.1.1-beta.1"]).is_err());
        assert_eq!(negotiate("=0.1.1-beta.1", &["0.1.1-beta.1"]).unwrap(), "0.1.1-beta.1");
    }
}

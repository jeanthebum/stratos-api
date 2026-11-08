# Public API versioning

Three versions have different jobs:

| Version | Location | Meaning |
| --- | --- | --- |
| Shader pack release | Top-level `version` | The shader author's own release number |
| Required public API | `[stratos] api` | SemVer range of contracts the pack supports |
| Mod implementation | `horizon-native.json` / core `Cargo.toml` | The installed renderer implementation |

Horizon `api = "0.2"` remains the underlying stage/colour format. It is not the StratosAPI release number. Atmos versions have no role in API negotiation.

The implementation advertises contracts it actually serves in `SUPPORTED_APIS`. Version 0.1.0 serves exactly contract 0.1.0. The highest advertised version satisfying the pack's range is selected. A renderer does not assume it supports every version less than its own. `=0.1.1` therefore fails on this implementation, even though it is a nearby version. `^0.1.0` and `>=0.1.0, <0.2.0` accept contract 0.1.0. Empty/invalid ranges and unsupported requirements produce explicit manifest errors. Prereleases require explicit opt-in under Cargo SemVer semantics; OR (`||`) expressions are not supported.

During 0.x development, a breaking public contract change increments the minor version. Compatible additions may increment the patch contract version. Implementation fixes can ship under a new mod release while retaining contract 0.1.0. Stable 1.x will use major versions for breaking changes.

Public contract includes field meanings, units and offsets, stage signatures and bindings, colour/depth conventions, option semantics, profile behaviour, and weather geometry/accessors. Fixing an implementation to match the documented contract is not a new ABI.

The 0.1.0 snapshot records all Frame offsets/types, its 816-byte size, and hashes of every composed stage prelude and entry point. Regression tests compare it with the implementation and retain Horizon 0.1's existing ABI checks. Do not update the snapshot to make a breaking change pass: add a new contract and an adapter/implementation. Merely adding a number to `SUPPORTED_APIS` does not implement compatibility.

Future renderers should retain old contracts when practical. If a contract must be removed, it is removed explicitly from the advertised list, documented as an implementation breaking change, and old packs receive the compatibility error. Never silently execute a pack against a different ABI.

Legacy Atmos packs without `[stratos]` retain a narrow compatibility mapping for their old particle-control IDs. This is not part of the third-party API; new packs must declare the contract and their own weather controls.

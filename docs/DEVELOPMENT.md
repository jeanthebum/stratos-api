# Build and review

The repository is an unpacked Horizon native mod at its root. `mixins/stratos_api/core` is its standalone Rust library and guarded build integration. `core/data/engine/` contains the changed Horizon source files, paired with original/result SHA-256 hashes in `engine-patches.json`. All text hashes normalize CRLF to LF.

Use Rust 1.99.1 (the pinned Horizon toolchain), Python 3.11+, the platform C/C++ linker/development libraries Horizon requires, and the exact Horizon/mixins revisions in `horizon-native.json`. Do not distribute prepared Minecraft resources.

## Validate the core

```sh
cargo test --manifest-path mixins/stratos_api/core/Cargo.toml
python tools/package.py
```

The standalone core build does not patch a checkout. Its tests cover SemVer negotiation, independent implementation/contract versions, stage guards, full conflict preflight, corrupted data, traversal/duplicate paths, line endings and repeated application. Cargo may create a local lockfile; it is ignored/excluded from the mod because native builds use Horizon's pinned lock.

## Validate native integration

Export the pinned Horizon commit into a fresh directory and write its full SHA to `.horizon-revision`. Build `horizon-mixins` at the pinned commit, then:

```sh
horizon-mixins apply --workspace PATH_TO_PRISTINE_HORIZON --mods PATH_TO_STRATOS/mixins --out PATH_TO_STAGE
cd PATH_TO_STAGE
cargo build --locked --profile fast -p horizon-client --bin horizon-client -p horizon-client-lib --bin horizon-ctl -p horizon-shader-api --bin horizon-shader -p cargo-horizon-mod --bin cargo-horizon-mod
cargo test --locked --profile fast -p stratos-api-core -p horizon-shader-api --lib --tests
target/fast/horizon-shader check PATH_TO_STRATOS/examples/summer --json
target/fast/cargo-horizon-mod check PATH_TO_STRATOS/dist/StratosAPI-0.1.0.hnmod --json
```

Build once before naming newly introduced test targets: Cargo discovers test filenames before dependency build scripts run. The native overlay is applied during that first build. The test suite then includes the generated stage/Frame ABI checks and incompatible-manifest cases.

For repeated manual `horizon-mixins apply` runs on Windows, choose a fresh output directory. The pinned mixin CLI cannot atomically replace an existing changed stage outside Linux (`E_ATOMIC_STAGE`). The launcher uses a fresh transformation directory/snapshot and then synchronizes its build stage. The overlay watches its target files so restoring pristine sources causes Cargo to reapply it.

For a GPU check, launch this newly built client in a scratch game directory with isolated Horizon/launcher/data/config homes. Load the example without `HORIZON_HDR_FRAME`, verify shader selection and precipitation, then exit. Never test against a player's only world copy. Test an incompatible contract as well; manifest rejection must happen before GPU compilation.

The launcher provides the normal player installation path and manages the source/toolchain/cache. Manual commands above test the underlying native staging and build mechanism.

## Updating the renderer

Keep a pristine upstream pin and make renderer edits in a development checkout. Review the diff, update the payload and hashes, then test application against the pristine export. A new upstream revision requires a new implementation release with updated pins/hashes and another native/GPU validation. Do not loosen hash checks to accommodate another mod: reconcile changes explicitly in a new supported integration.

The build script prevalidates every file, then replaces only allowlisted source/reference paths under a directory with both native-stage markers. It does not download code, install binaries, modify launcher settings, or touch worlds. An interrupted filesystem write can leave an incomplete disposable stage; restaging from pristine source is the recovery path.

Successful hash validation proves source compatibility, not semantic compatibility with every other mod. Two mods editing the same file are deliberately refused. Supporting arbitrary structural composition needs upstream renderer hook work; this release does not claim it.

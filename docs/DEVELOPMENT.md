# Build and review

The repository is an unpacked Horizon native mod at its root. `mixins/stratos_api/core` is its standalone Rust library and guarded build integration. `core/data/engine/` contains the changed Horizon source files, paired with original/result SHA-256 hashes in `engine-patches.json`. All text hashes normalize CRLF to LF.

Use Rust 1.99.1 (the pinned Horizon toolchain), Python 3.11+, the platform C/C++ linker/development libraries Horizon requires, and the exact Horizon/mixins revisions in `horizon-native.json`. Do not distribute prepared Minecraft resources.

## Validate the core

```sh
cargo test --workspace
python -m unittest discover -s tools -p "test_*.py"
python tools/package.py
```

The standalone core build does not patch a checkout. Its tests cover SemVer negotiation, independent implementation/contract versions, stage guards, full conflict preflight, corrupted data, traversal/duplicate paths, line endings and repeated application. The root lockfile covers standalone SDK tests. A lockfile inside the bundled core is excluded from the mod because native builds use Horizon's pinned lock.

## Official native check

Clone [horizon-mods](https://github.com/horizon-rust/horizon-mods), then run its documented check directly on Linux/WSL (Rust, linker dependencies and rsync required):

```sh
python /path/to/horizon-mods/native/tools/native.py check /absolute/path/to/stratos-api
```

On native Windows, use our small filesystem adapter:

```sh
python tools/check.py --registry /path/to/horizon-mods
```

The adapter imports the unmodified official driver and replaces only its `rsync` stage-copy operation with a checksummed Python copy inside the expected disposable target directories. Its source download, horizon-mixins transformation and `cargo check --locked -D warnings` run normally. It preserves additional platform linker flags without allowing them to bypass warnings-as-errors. The check was exercised with horizon-mods revision `32abdd651c045ca2a6aab40c4560152507c07d67`.

Do not pass `--horizon latest`: the overlay supports only the exact compatibility pin. `compatibility.json`'s hooks minimum describes lifecycle hooks, not the structural renderer's compatibility. Do not use upstream `native.py package` for this extension: that packager omits the overlay data and advertises generic hook compatibility. Use `tools/package.py`, which includes the verified payload and preserves the exact pins.

## Validate native integration

Export the pinned Horizon commit into a fresh directory and write its full SHA to `.horizon-revision`. Build `horizon-mixins` at the pinned commit, then:

```sh
horizon-mixins apply --workspace PATH_TO_PRISTINE_HORIZON --mods PATH_TO_STRATOS/mixins --out PATH_TO_STAGE
cd PATH_TO_STAGE
cargo build --locked --profile fast -p horizon-client --bin horizon-client -p horizon-client-lib --bin horizon-ctl -p horizon-shader-api --bin horizon-shader -p cargo-horizon-mod --bin cargo-horizon-mod
cargo test --locked --profile fast -p stratos-api-core -p horizon-shader-api --lib --tests
target/fast/horizon-shader check PATH_TO_STRATOS/examples/summer --json
target/fast/cargo-horizon-mod check PATH_TO_STRATOS/dist/StratosAPI-0.1.1.hnmod --json
```

Build once before naming newly introduced test targets: Cargo discovers test filenames before dependency build scripts run. The native overlay is applied during that first build. The test suite then includes the generated stage/Frame ABI checks and incompatible-manifest cases.

For repeated manual `horizon-mixins apply` runs on Windows, choose a fresh output directory. The pinned mixin CLI cannot atomically replace an existing changed stage outside Linux (`E_ATOMIC_STAGE`). The launcher uses a fresh transformation directory/snapshot and then synchronizes its build stage. The overlay watches its target files so restoring pristine sources causes Cargo to reapply it.

For a GPU check, launch this newly built client in a scratch game directory with isolated Horizon/launcher/data/config homes. Load the example without `HORIZON_HDR_FRAME`, verify shader selection and precipitation, then exit. Never test against a player's only world copy. Test an incompatible contract as well; manifest rejection must happen before GPU compilation.

The launcher provides the normal player installation path and manages the source/toolchain/cache. Manual commands above test the underlying native staging and build mechanism.

## Updating the renderer

Keep a pristine upstream pin and make renderer edits in a development checkout. Review the diff, update the payload and hashes, then test application against the pristine export. A new upstream revision requires a new implementation release with updated pins/hashes and another native/GPU validation. Do not loosen hash checks to accommodate another mod: reconcile changes explicitly in a new supported integration.

The build script prevalidates every file, then replaces only allowlisted source/reference paths under a directory with both native-stage markers. It does not download code, install binaries, modify launcher settings, or touch worlds. An interrupted filesystem write can leave an incomplete disposable stage; restaging from pristine source is the recovery path.

Successful hash validation proves source compatibility, not semantic compatibility with every other mod. Two mods editing the same file are deliberately refused. Supporting arbitrary structural composition needs upstream renderer hook work; this release does not claim it.

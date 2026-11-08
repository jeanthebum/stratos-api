# StratosAPI

An independently versioned shader API and native renderer mod for [Horizon](https://playhorizon.dev/), the Rust Minecraft client. Write WGSL shader packs with linear lighting, reflections, biome data, procedural weather and performance presets.

**Implementation:** 0.1.0 · **Public shader contract:** 0.1.0 · **Status:** experimental, pinned native integration.

StratosAPI is a separate project from [Atmos Shader](https://github.com/jeanthebum/atmos-shader). Atmos is one consumer; no Atmos shader sources, textures, game assets, worlds or executables are included here.

## Install

In a Horizon launcher that supports native mods, open your instance's **Mods → Add from GitHub** and enter:

```text
https://github.com/jeanthebum/stratos-api
```

Review the source/trust prompt, install `stratos_api`, and let the launcher build the native game. Restart that instance before selecting a Stratos-compatible `.hshader`. The first build downloads a toolchain/dependencies and can take substantial time and disk space. This is source compiled into an isolated game build, not a runtime DLL or an Iris/OptiFine mod.

CLI equivalent, after reviewing the native code:

```sh
horizon-launcher mods add-github https://github.com/jeanthebum/stratos-api --mod stratos_api --instance YOUR_INSTANCE --trust-native-code
horizon-launcher native-build --instance YOUR_INSTANCE --trust-native-code
```

To build a local `.hnmod`, install Python 3.11+ and run `python tools/package.py`. It writes the mod, an SDK source archive, and SHA-256 checksums under `dist/`. An unpacked checkout can also be passed to `horizon-launcher native-build PATH --instance YOUR_INSTANCE --trust-native-code`.

**Compatibility:** this release pins Horizon `fd06d1c2650b52d8d4bb3d808ca73e55971ba717` and horizon-mixins `5a64af938ec86b00d53d16a408aabc0038fb3e57` (Minecraft 26.3). It is not compatible with every Nightly. The launcher builds the pinned source. Do not override the pins to make a newer source revision appear supported. This project has not been reviewed or accepted into Horizon's mod registry.

Disable/remove StratosAPI in the instance's native mods and rebuild to uninstall. Shader packs requiring it must also be deselected. Player worlds are not part of this mod.

## Make a shader

Start with [`examples/summer`](examples/summer). Its pack ID and version are independent of Atmos and of StratosAPI. In `horizon-shaders.toml`:

```toml
schema = 1
id = "my_shader"
name = "My Shader"
version = "4.7.0"
api = "0.2" # Horizon's underlying shader format

[stratos]
api = ">=0.1.0, <0.2.0" # Your supported StratosAPI contracts
```

Add at least one stage, then validate/package with the **Stratos-built** `horizon-shader` tool:

```sh
horizon-shader check examples/summer --json
horizon-shader pack examples/summer --out Summer.hshader
```

An unsupported or malformed requirement fails during manifest loading, before shader compilation. Compatible declarations select the float-frame renderer automatically; no `HORIZON_HDR_FRAME` environment variable is needed. Stock Horizon's validator does not know `[stratos]` and cannot fully validate these packs.

## What is available

- Six WGSL stages: sky, terrain vertex, material, weather, post-processing and tonemapping.
- Linear-light shader calculations, floating-point scene storage and an SDR display transform. Framebuffer blending currently uses Horizon's gamma mode.
- Opaque scene colour/depth, planar water reflection colour/depth, shadow maps, post-pass history and named intermediate targets.
- Blended camera-biome weights for jungle, swamp, taiga, dark forest, arid terrain, snow/ice, cherry, plains, forest, coast and savanna.
- Shader-defined rain and water beads, leaf attachment, falling motion and impact geometry. Packs choose their own optics and control names.
- Validated option bundles for quality presets, with a Custom state that preserves manual adjustments.

See [the public contract](docs/API.md), [versioning policy](docs/VERSIONING.md), [stage reference](docs/REFERENCE-0.1.md), [build and review guide](docs/DEVELOPMENT.md) and [validation record](docs/VALIDATION.md).

## Native integration limits

This first implementation needs structural renderer changes that Horizon's named hooks cannot express. A bundled crate's `build.rs` applies a checksummed source overlay **only inside horizon-mixins' disposable stage**, before the shader API and its dependent render/client crates compile. Original checkouts are not edited. Direct builds of the core library do not apply an overlay.

All files are checked before any writes. An unexpected source hash fails the build instead of overwriting another mod's edits. Mods changing the same renderer files therefore do not compose with this release. The manifest deliberately does not claim generic `hooks` compatibility. See [`engine-patches.json`](mixins/stratos_api/core/data/engine-patches.json) for the exact file list and hashes; the adjacent `engine/` directory contains the reviewable source.

The API provides renderer inputs and passes, not a finished visual style or a performance guarantee. Planar reflections have visibility limits; biome values describe camera surroundings, not a biome ID for each pixel. Runtime validation is currently Windows x64; other platforms need their own compile and GPU validation.

MIT licensed; Horizon-derived code retains its upstream copyright. See [LICENSE](LICENSE) and [NOTICE](NOTICE).

Keywords: horizon, minecraft, shader-api, native-mod, rust, wgsl, wgpu, rendering.

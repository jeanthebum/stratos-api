# StratosAPI 0.1.0 contract

See [REFERENCE-0.1.md](REFERENCE-0.1.md) for inherited WGSL structures/resources. This document defines the Stratos extensions and takes precedence over upstream experimental-status notes. Use the Stratos-built validator, not a stock Horizon executable.

## Declaration and rendering

Every new Stratos pack declares top-level `api = "0.2"` and `[stratos] api = ">=0.1.0, <0.2.0"`. The latter is validated before GPU compilation and enables the float scene renderer. Pack names, IDs and release numbers do not select this API.

Stage functions are ordinary WGSL functions; do not declare entry points or bindings, or use the reserved `horizon_` prefix. Stage colours are linear BT.709/sRGB primaries, with HDR scene values allowed above 1. Tone mapping converts to SDR display output. Framebuffer blending currently uses gamma mode: omit `[frame]` or set `blend = "gamma"`. Horizon parses `"linear"` but the runtime does not serve it; this release does not remove that limitation. Stage inputs and outputs are converted around that storage/blending boundary. Depth is reversed: zero is sky/far, larger is nearer. Positions are camera-relative world axes unless a field explicitly says otherwise.

## Biome inputs

The uniform buffer is 816 bytes. The [frozen layout](../mixins/stratos_api/core/data/contracts/0.1.0.json) records every offset and type.

| WGSL field | Meaning |
| --- | --- |
| `frame.biome_jungle` | Jungle weight |
| `frame.biome_swamp` | Swamp/mangrove weight |
| `frame.biome_taiga` | Temperate taiga weight |
| `frame.biome_other.xyzw` | Dark forest, arid, snow/ice, cherry |
| `frame.biome_landscape.xyzw` | Plains/meadow, temperate forest, ocean/coast, savanna |

Weights are bounded 0–1 and spatially blended around the camera. They are not per-pixel biome membership and must not be assumed to form a complete normalized partition. Shader authors decide how strongly to tint light, water and fog, and should expose a toggle if biome tint is optional.

## Procedural precipitation

Declare a `[weather]` stage, plus optional native geometry controls:

```toml
[stratos.weather]
liquid_droplets = true
leaf_drips_option = "my_drips"      # Existing float/int option, runtime clamped 0–2
humidity_option = "my_humidity"     # Existing float/int option, runtime clamped 0–2
enabled_option = "my_wet_effects"   # Existing bool option; gates leaf drips
```

All control fields are optional. Without `leaf_drips_option`, leaf drip strength is zero. Humidity defaults to 1, the enable flag to true. `liquid_droplets` defaults to false; a leaf-drip option requires it true. Option names/types are checked. Runtime consumes the resolved option values, including defaults and saved settings. Non-finite runtime values are ignored. Omitting the entire table opts out of custom native particles.

Like all Horizon options, these controls must also have matching WGSL `override` declarations in a stage, even when only the native geometry consumes the value. Float options use `f32`, integer options use `i32`, and boolean options use `bool`.

`liquid_droplets` enables shader weather geometry and conversion of applicable native water-particle events. `enabled_option` and zero drip strength stop new canopy emissions; they do not disable rain, remove existing droplets immediately, or switch back the renderer's water-particle conversion. Toggle the rain shading inside your weather stage if desired.

| Helper | Result |
| --- | --- |
| `weather_kind()` | 0 rain, 1 snow, 2 hanging leaf bead, 3 falling bead, 4 impact ring |
| `weather_uv()` | Geometry-local 2D coordinates |
| `weather_screen()` | Pixel x/y plus reversed depth z |
| `weather_progress()` | Normalized particle lifetime progress |
| `weather_opacity()` | Geometry opacity/lifetime envelope |

`weather_scene_color` is the scene colour texture; `weather_scene_depth` is its depth. Use `textureDimensions` and clamp integer sample positions. Refraction offsets should reject foreground depth crossings. The renderer supplies leaf anchors, gravity, collision and bounded populations; the shader supplies shape, transparency, refraction and highlights. Snow remains separately identifiable. The example intentionally uses simple optics, without Atmos assets.

## Reflections and post-processing

Material stages can sample `opaque_color`, `opaque_depth`, `reflection` and `reflection_depth`. Check `frame.has_reflection`, use the declared reflection transforms/dimensions, and handle missing/out-of-view samples. Reflections are planar water mirrors, not arbitrary ray-traced reflections. Opaque buffers do not contain hidden surfaces or later translucent layers.

Post stages support scaled named outputs, previous-pass colour, history, depth and shadow inputs. Read the stage reference for binding-specific availability; not every resource is present in every stage. Keep named-output dependencies acyclic and use the validator to catch unsupported combinations.

## Quality presets

`[profiles]` binds an existing choice option to named presets. `option` is its ID and `custom` is the manual-settings choice. Each `[[profiles.presets]]` has `value` and a `values` table mapping option IDs to valid typed values. All presets must specify the same set of controlled options. Do not include the profile selector itself. Manual edits of a controlled value select Custom; edits to unrelated artistic controls preserve the current profile. Presets must not include the Custom choice. The validator rejects duplicate/incomplete presets, missing options and invalid values. See the included renderer profile tests for complete fixtures.

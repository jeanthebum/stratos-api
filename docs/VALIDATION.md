# Validation: implementation 0.1.1

Validated November 8, 2026 on Windows x64 and an NVIDIA RTX 5060 Laptop GPU (Vulkan). Public shader contract **0.1.0** is unchanged. These are functional checks, not a performance benchmark or cross-platform certification.

## Compiler and packages

- Horizon's official `native/tools/native.py check` passed against the pinned source with Rust warnings denied. The native Windows adapter substituted only its rsync filesystem operation. The registry tools checkout was `32abdd651c045ca2a6aab40c4560152507c07d67`.
- **74 API/core tests passed:** 66 shader API tests, four SemVer tests and four guarded-overlay tests. Two Windows stage-copy tests also passed. The public ABI snapshot and Horizon 0.1 ABI checks remained unchanged.
- The standalone Summer example passed four stage checks; the packaged Atmos 0.16.0 passed all 13. Horizon's native bundle validator accepted the 0.1.1 `.hnmod` and its exact pins.

## Launcher installation

The actual installed Horizon Launcher built the unpacked 0.1.1 mod in a separate instance and isolated launcher/home/config/data folders. It downloaded the pinned Horizon source, built the pinned horizon-mixins, checked dependencies against Horizon's lockfile, compiled with its managed Rust 1.99.1 GNU/WinLibs toolchain, installed the native game and selected it for that instance. Existing downloaded toolchain/dependency/base-resource caches were copied into the isolated home; this was not a clean-machine download test.

The first performance-profile build completed in 11 min 28 s. A final lifecycle adjustment rebuilt incrementally in 3 min 25 s. The tested native build was `native-fd06d1c-589b426f2959`, with no mods omitted. These are observations from this machine, not promised build times. The optional Windows ZIP contains this actual client; its shader CLI was built from the same source/API with Rust 1.99.1 and LLVM-MinGW.

Separately, the repository owner reported and supplied a launcher screenshot showing **0.1.0** installed from GitHub commit `42a230f`, successfully built with the release profile in 14 min 55 s, and running Atmos. That user-reported GitHub result is distinct from the isolated 0.1.1 local-bundle installation above.

## Two bounded live checks

1. **Lifecycle with the independent Summer example:** the real Options > Mods row toggled StratosAPI off while the menu was open. All canopy/liquid counters cleared and remained zero while playing and after a full client restart. Switching it on again rebuilt renderer resources and loaded the shader, with its selection and saved options unchanged. The initial enabled run emitted 66 canopy beads and converted 12 water-particle events; re-enabling emitted 42. On/off captures confirmed the custom sky gave way to vanilla clouds. An initial harness attempt had to be corrected to leave the Options screen before waiting for game ticks; the successful rerun tested the same binaries.
2. **Atmos smoke test:** Atmos 0.16.0 loaded all its real GPU pipelines without `HORIZON_HDR_FRAME`. The run emitted 47 canopy beads, registered 42 landings and converted 12 native water-particle events. The temporary fixture was removed and the test client exited.

Both checks used a capped, small offscreen ShaderLab window. No player world or normal launcher installation was changed. Counters are evidence of the integration running, not a guaranteed particle rate.

## Limits

The structural renderer overlay remains experimental and deliberately refuses unexpected source hashes or conflicting renderer edits. Standard hooks manage discovery and lifecycle; the full GPU extension is not claimed to be hook-only or forward-compatible with arbitrary Horizon versions. Linux/macOS, other GPUs, arbitrary renderer-mod combinations and clean-machine provisioning remain untested.

`[frame] blend = "linear"` remains unsupported by this runtime. Shader calculations use linear light and floating-point scene storage; framebuffer blending uses Horizon's supported gamma mode. The public example and documentation reflect that limit.

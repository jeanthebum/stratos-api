# Validation: implementation 0.1.0

Validated on November 8, 2026, on Windows x64 with Rust 1.99.1, LLVM-MinGW, and an NVIDIA RTX 5060 Laptop GPU using Vulkan. This is functional validation, not a cross-platform certification or performance benchmark.

- Exported pristine Horizon `fd06d1c2650b52d8d4bb3d808ca73e55971ba717` and applied the native bundle with official horizon-mixins `5a64af938ec86b00d53d16a408aabc0038fb3e57`.
- Compiled a small isolated fixture first. Its test proves the native dependency build script modifies staged source before the dependent package compiles. Core/overlay tests also passed there.
- Built the complete staged `horizon-client`, `horizon-ctl`, `horizon-shader` and `cargo-horizon-mod` with `--locked --profile fast`.
- Verified all 32 original source/reference files remain at their pristine hashes (including absence of newly added files), and all staged files match the shipped overlay hashes.
- **74 API/core tests passed:** 66 shader API tests, four SemVer tests, and four overlay tests. These include shader schemas, WGSL validation, packaging, quality presets, old Horizon 0.1 ABI stability, frozen Stratos 0.1.0 ABI, independent pack IDs, invalid/future contracts, weather control types, checksum/path/conflict guards and repeated application.
- Horizon's native-mod validator accepted the `.hnmod` as `stratos_api` 0.1.0, with the intended game and exact source pins.
- The standalone Summer example passed all four stage checks. Atmos 0.16.0, declaring the same Stratos contract, passed all 13 stage checks.
- Both packs loaded actual GPU pipelines in isolated ShaderLab runs with `HORIZON_HDR_FRAME` absent. The example emitted 66 canopy beads, registered 55 landings and converted 12 native water-particle events; Atmos emitted 46, registered 41 landings and converted 12 events. Counters vary with timing; these prove the generic integration path, not a required particle rate.
- Captures and native shader-menu state were inspected. Both test clients exited, and the temporary canopy fixture and test settings were restored. No player-world installation was changed.

The initial GPU test exposed the inherited runtime limitation on `[frame] blend = "linear"`. The example now uses supported gamma blending; documentation explicitly preserves that limitation. Shader calculations still use the underlying 0.2 linear-light contract and floating-point scene storage.

Not yet validated: Linux/macOS runtime, other GPUs, arbitrary renderer-mod combinations, or every future Horizon revision. The overlay deliberately refuses unexpected source hashes. Full launcher-managed download/build installation on a clean end-user machine remains a separate installation test; the official native bundle validation, staging, locked native build and live GPU path have been exercised here.

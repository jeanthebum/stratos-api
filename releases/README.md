# StratosAPI 0.1.1 downloads

- **[StratosAPI-0.1.1.hnmod](StratosAPI-0.1.1.hnmod)**: the installable native-mod source bundle. Horizon compiles this into the game for your instance.
- **[SHA256SUMS](SHA256SUMS)**: checksums for the mod, source SDK and optional Windows build.
- Additional source/Windows archives can be attached to [GitHub Releases](https://github.com/jeanthebum/stratos-api/releases). They are excluded from the source SDK and the native-mod bundle.

For a release upload, use tag **v0.1.1** and title **StratosAPI 0.1.1**. The public shader contract stays **0.1.0**, independently of this implementation version and Atmos 0.16.0.

## Changes

StratosAPI now appears in Options → Mods. Turning it off suspends dependent shaders and shader-made water droplets immediately. The off state persists across restarts; turning it on reloads the selected pack while preserving its options. When necessary, enabling it reloads renderer resources to restore floating-point scene storage.

This release adds compatibility metadata, a root test workspace, the official native-check workflow with a Windows filesystem adapter, and installable packages. The full renderer remains an **experimental extension pinned to Horizon fd06d1c2650b52d8d4bb3d808ca73e55971ba717**. Structural GPU changes still use a guarded source overlay inside the disposable native build stage. This is not generic hook-only compatibility or upstream endorsement.

## Optional Windows build

`StratosAPI-0.1.1-windows-x64.zip` contains the actual launcher-built **performance-profile** client, the Stratos shader validator/packager, build information and dependency licenses. It is an unsigned optional renderer build, not a DLL or a standalone Minecraft distribution. Extract it and use Horizon Launcher's Import Build feature on `horizon-client.exe` with your own prepared Minecraft 26.3 resources. An imported executable is updated by importing another build; installing the `.hnmod` remains the recommended managed-mod route.

The `.hshader` format contains WGSL programs; the renderer compiles them for the local GPU at load time. Worlds, prepared game assets and personal settings are excluded from every downloadable artifact.

See [validation](../docs/VALIDATION.md) for the checks performed and their limits.

# Packaging PoC

The packaging workflows deliberately separate cross-compilation from target-platform packaging and trust operations.

## Windows x64

Run from PowerShell on Windows:

```powershell
.\packaging\windows\Publish-SingleFile.ps1
```

The script publishes into an internal staging directory and copies only `Mnelys.exe` into the delivery directory. Native PDB files in staging are not user artifacts.

## macOS arm64

Run on an Apple Silicon Mac with the .NET 10 SDK and Xcode command-line tools:

```bash
./packaging/macos/build-dmg.sh
```

Without `MNELYS_CODESIGN_IDENTITY`, the script uses ad-hoc signing for the PoC. A public artifact requires a Developer ID Application identity, Hardened Runtime verification, notarization, ticket stapling, and Gatekeeper testing. Intel/x64 is intentionally out of scope.

## Linux x64

Run on the selected old-glibc Linux build image with `appimagetool` available:

```bash
./packaging/linux/build-appimage.sh
```

The script creates an AppDir, applies executable permissions, and invokes `appimagetool`. Validate the result on the distribution, desktop environment, display-server, GPU, FUSE, and Secret Service matrix before promotion.

See [the M0 packaging report](../docs/M0_PACKAGING_POC.md) for current evidence and remaining gates.

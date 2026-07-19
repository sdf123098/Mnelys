# M0 packaging PoC report

Status: In progress — cross-platform payloads built; target-platform packaging gates remain
Last updated: 2026-07-19

## Baseline

| Component | Version |
|---|---|
| Host | Windows 11 x64 (`10.0.26200`) |
| .NET SDK | `10.0.302` |
| .NET runtime | `10.0.10` |
| C# | 14.0 |
| Avalonia | `12.1.0` |
| MVVM | CommunityToolkit.Mvvm `8.4.2` |

The Shell uses compiled bindings, an opaque root background, system window decorations, a 1280×800 initial size, and a 900×600 minimum size. It does not request Mica, Acrylic, desktop sampling, or undocumented DWM effects.

## Results

| Target | Result | Evidence |
|---|---|---|
| Debug build | Pass | `0` warnings, `0` errors |
| Windows development launch | Pass | Process remained stable and exposed the main window title `忆涟 · Mnelys` |
| Windows x64 single-file | Pass on development host | Delivery contains one `Mnelys.exe`, 50,602,905 bytes, SHA-256 `E041B72706F38D9E5A1F4463CD568EC0C7D57125B4B124021983836F24060292` |
| Windows startup sample | Pass on development host | Approximately 1,755 ms from process start to a main-window handle; this is not yet a controlled benchmark |
| Windows bundle extraction | Observed | Three native files, 18,839,080 bytes total: ANGLE GLES, HarfBuzz, and SkiaSharp |
| macOS arm64 payload | Cross-publish pass | 221 files, 118,433,829 bytes |
| macOS `.app` layout | Structural pass on Windows | 223 files, 120,113,504 bytes; executable, `Info.plist`, and ICNS paths present |
| Linux x64 payload | Cross-publish pass | 221 files, 108,629,845 bytes |
| Linux AppDir layout | Structural pass on Windows | 225 files, 108,915,997 bytes; AppRun, desktop entry, icon, and executable paths present |

The Windows publish staging directory includes `libSkiaSharp.pdb` and `libHarfBuzzSharp.pdb`. They are intentionally excluded from the one-file delivery directory while remaining available as internal diagnostics artifacts.

## Native AOT trial

The project declares `IsAotCompatible=true`, uses compiled bindings, and does not use the template's reflection-based ViewLocator.

The first AOT attempt exposed and fixed an invalid explicit `PublishTrimmed=false` setting. The next attempt reached the Windows native toolchain check and stopped because the host does not have the Visual Studio Desktop Development for C++ workload/platform linker installed.

No Native AOT artifact was produced. This is an environment/toolchain blocker, not evidence that the current Avalonia Shell is incompatible. Self-contained single-file JIT remains the release baseline as required by ADR 0002.

## Host-specific SDK observation

On this machine, `dotnet --info` reports a workload installer `TypeInitializationException` even though SDK compilation and publishing work. PoC commands disable the MSBuild workload resolver because this desktop project needs no .NET workload. This workaround must not hide workload failures once mobile or other workload-dependent targets are introduced.

## Remaining target-platform gates

- Windows: repeat on a clean VM without a preinstalled .NET runtime, with Chinese/space/long paths and restricted temporary directories.
- macOS arm64: run `build-dmg.sh` on Apple Silicon, review the ICNS at native sizes, sign with Developer ID, notarize, staple, and pass Gatekeeper.
- Linux x64: run `build-appimage.sh` on the selected old-glibc builder, then test FUSE/no-FUSE, X11/XWayland, representative desktops, and GPU drivers.
- Native AOT: install the Windows C++ linker prerequisites before the next milestone trial, then repeat on each supported target OS.

## Reproduction

- Windows: `packaging/windows/Publish-SingleFile.ps1`
- macOS arm64: `packaging/macos/build-dmg.sh`
- Linux x64: `packaging/linux/build-appimage.sh`

# Mnelys / 忆涟 branding assets

## Canonical source

| Field                               | Value                                                              |
| ----------------------------------- | ------------------------------------------------------------------ |
| File                                | `mnelys-icon-source.jpg`                                           |
| Purpose                             | Canonical source artwork for the Mnelys / 忆涟 application icon    |
| Supplied by                         | Project owner, 2026-07-19                                          |
| Dimensions                          | 690×690 pixels                                                     |
| Encoding                            | JPEG, 24-bit RGB, opaque white background                          |
| File size                           | 92,967 bytes                                                       |
| SHA-256                             | `9B77E06FDB89767EF9AA4524D857F969BBB0FD95D4A0251F7FFDEA9B05E4B2E1` |
| Authorship and redistribution terms | Not yet recorded; required before public release                   |

The source file is preserved without recompression or editing. Do not overwrite it when generating platform assets.

## Planned derivatives

- Windows: multi-resolution `Mnelys.ico`, including 16, 24, 32, 48, 64, 128, and 256 px representations.
- macOS: `Mnelys.icns` generated from an approved padded square master and visually checked against the current macOS icon mask.
- Linux: lossless PNG sizes under the hicolor icon layout, including a 512 px master.
- Documentation: optimized preview files may be generated separately; they are never treated as masters.

All generated assets must be reviewed at native size. Do not rely only on a large preview because fine linework and the eye mark may become unclear at launcher and taskbar sizes.

## Current derivatives

Generated from the canonical JPEG during the M0/Tauri baseline work. Paths reflect the repository as it stands after the .NET/Avalonia removal (ADR 0005).

| File                              | Usage                             |          Size | SHA-256                                                            |
| --------------------------------- | --------------------------------- | ------------: | ------------------------------------------------------------------ |
| `src-tauri/icons/mnelys-icon.png` | 512 px square master              | 285,593 bytes | `5CF72B2F765E087BA55CC28804C867FB1F48B4EE1F418F979097BB983656439D` |
| `src-tauri/icons/icon.png`        | 512 px bundle icon                | 295,365 bytes | `E46ABF6EE0CB31C5054E3561B6B10EFC0ED0E863D15412DF885CFA1D66F4B283` |
| `src-tauri/icons/128x128.png`     | 128 px platform icon              |  27,520 bytes | `8959DB58A410F8FD1C463B7484D7FC8FDABB466641FBA460634A5A4C28258716` |
| `public/app-icon.png`             | Web favicon and in-app brand mark |  27,520 bytes | `8959DB58A410F8FD1C463B7484D7FC8FDABB466641FBA460634A5A4C28258716` |

`public/app-icon.png` is a byte-for-byte copy of `src-tauri/icons/128x128.png`; it is the only brand raster the frontend renders, and it replaces no Lucide glyph.

These are mechanical derivatives from the canonical JPEG. They require native-size visual review before a public release and do not replace the master artwork.

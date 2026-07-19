#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
project="$repo_root/src/Mnelys.Desktop/Mnelys.Desktop.csproj"
artifact_root="$repo_root/artifacts/poc/osx-arm64"
publish_dir="$artifact_root/publish"
app_dir="$artifact_root/Mnelys.app"
dmg_root="$artifact_root/dmg-root"
dmg_path="$artifact_root/Mnelys-0.1.0-macos-arm64.dmg"

case "$artifact_root" in
  "$repo_root"/artifacts/poc/osx-arm64) ;;
  *) echo "Refusing to package outside the expected artifact directory." >&2; exit 1 ;;
esac

dotnet publish "$project" \
  -c Release \
  -r osx-arm64 \
  --self-contained true \
  -p:PublishSingleFile=false \
  -p:DebugType=None \
  -p:DebugSymbols=false \
  -p:NuGetAudit=false \
  -o "$publish_dir"

rm -rf "$app_dir" "$dmg_root"
mkdir -p "$app_dir/Contents/MacOS" "$app_dir/Contents/Resources" "$dmg_root"
cp -R "$publish_dir"/. "$app_dir/Contents/MacOS/"
cp "$repo_root/packaging/macos/Info.plist" "$app_dir/Contents/Info.plist"
chmod +x "$app_dir/Contents/MacOS/Mnelys"

iconset_dir="$(mktemp -d)/Mnelys.iconset"
mkdir -p "$iconset_dir"
trap 'rm -rf "${iconset_dir%/Mnelys.iconset}"' EXIT
source_icon="$repo_root/src/Mnelys.Desktop/Assets/mnelys-icon.png"

while read -r filename pixels; do
  sips -z "$pixels" "$pixels" "$source_icon" --out "$iconset_dir/$filename" >/dev/null
done <<'EOF'
icon_16x16.png 16
icon_16x16@2x.png 32
icon_32x32.png 32
icon_32x32@2x.png 64
icon_128x128.png 128
icon_128x128@2x.png 256
icon_256x256.png 256
icon_256x256@2x.png 512
icon_512x512.png 512
icon_512x512@2x.png 1024
EOF

iconutil -c icns "$iconset_dir" -o "$app_dir/Contents/Resources/Mnelys.icns"

codesign_identity="${MNELYS_CODESIGN_IDENTITY:--}"
codesign --force --deep --options runtime --sign "$codesign_identity" "$app_dir"
codesign --verify --deep --strict --verbose=2 "$app_dir"

cp -R "$app_dir" "$dmg_root/Mnelys.app"
ln -s /Applications "$dmg_root/Applications"
rm -f "$dmg_path"
hdiutil create -volname "Mnelys" -srcfolder "$dmg_root" -ov -format UDZO "$dmg_path"

echo "Created $dmg_path"
if [[ "$codesign_identity" == "-" ]]; then
  echo "PoC uses ad-hoc signing. Set MNELYS_CODESIGN_IDENTITY for a release candidate."
fi

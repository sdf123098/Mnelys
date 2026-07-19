#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
project="$repo_root/src/Mnelys.Desktop/Mnelys.Desktop.csproj"
artifact_root="$repo_root/artifacts/poc/linux-x64"
publish_dir="$artifact_root/publish"
app_dir="$artifact_root/AppDir"
output_path="$artifact_root/Mnelys-0.1.0-x86_64.AppImage"

case "$artifact_root" in
  "$repo_root"/artifacts/poc/linux-x64) ;;
  *) echo "Refusing to package outside the expected artifact directory." >&2; exit 1 ;;
esac

if ! command -v appimagetool >/dev/null 2>&1; then
  echo "appimagetool is required to create the AppImage." >&2
  exit 2
fi

dotnet publish "$project" \
  -c Release \
  -r linux-x64 \
  --self-contained true \
  -p:PublishSingleFile=false \
  -p:DebugType=None \
  -p:DebugSymbols=false \
  -p:NuGetAudit=false \
  -o "$publish_dir"

rm -rf "$app_dir"
mkdir -p "$app_dir/usr/bin" "$app_dir/usr/share/applications" \
  "$app_dir/usr/share/icons/hicolor/512x512/apps"
cp -R "$publish_dir"/. "$app_dir/usr/bin/"
cp "$repo_root/packaging/linux/AppRun" "$app_dir/AppRun"
cp "$repo_root/packaging/linux/io.mnelys.launcher.desktop" \
  "$app_dir/io.mnelys.launcher.desktop"
cp "$repo_root/packaging/linux/io.mnelys.launcher.desktop" \
  "$app_dir/usr/share/applications/io.mnelys.launcher.desktop"
cp "$repo_root/src/Mnelys.Desktop/Assets/mnelys-icon.png" \
  "$app_dir/usr/share/icons/hicolor/512x512/apps/io.mnelys.launcher.png"
ln -s "usr/share/icons/hicolor/512x512/apps/io.mnelys.launcher.png" \
  "$app_dir/io.mnelys.launcher.png"
chmod +x "$app_dir/AppRun" "$app_dir/usr/bin/Mnelys"

ARCH=x86_64 appimagetool "$app_dir" "$output_path"
sha256sum "$output_path"

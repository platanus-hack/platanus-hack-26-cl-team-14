#!/usr/bin/env bash
# Build the AppImage and publish it in a GitHub release: tools/release-appimage.sh [tag]
# No tag: replace the AppImage in the latest release.
# New tag: create that release. It becomes "latest", so add the macOS ZIP too,
# because the landing page links to releases/latest/download/santiaguito-macos-arm64.zip.
set -eu
cd "$(dirname "$0")/.."
export GH_PAGER=cat
tag=${1:-$(gh release view --json tagName -q .tagName)}
dir=$(mktemp -d)
trap 'rm -rf "$dir"' EXIT
file="$dir/santiaguito-x86_64.AppImage"

tools/build-appimage.sh "$file"

sums="$dir/SHA256SUMS"
if gh release view "$tag" >/dev/null 2>&1; then
    # Keep the checksums of the other assets. Replace only the AppImage line.
    gh release download "$tag" -p SHA256SUMS -O - | grep -v ' santiaguito-x86_64.AppImage$' > "$sums"
    (cd "$dir" && sha256sum santiaguito-x86_64.AppImage) >> "$sums"
    gh release upload "$tag" "$file" "$sums" --clobber
else
    (cd "$dir" && sha256sum santiaguito-x86_64.AppImage) > "$sums"
    gh release create "$tag" "$file" "$sums" --target "$(git rev-parse HEAD)" \
        --title "santiaguito.cl $tag" --notes "Linux x86_64 AppImage. Run: chmod +x santiaguito-x86_64.AppImage && ./santiaguito-x86_64.AppImage"
fi
gh release view "$tag" --json url -q .url

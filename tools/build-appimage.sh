#!/usr/bin/env bash
# Pack the Linux city app into one AppImage: tools/build-appimage.sh [output]
# The AppImage copies its files to ~/.local/share/santiaguito/app on first run,
# because the chat tools save replays next to the data.
set -eu
cd "$(dirname "$0")/.."
out=${1:-$PWD/Santiaguito-x86_64.AppImage}
tool=${APPIMAGETOOL:-appimagetool}
if ! command -v "$tool" >/dev/null; then
    tool="${XDG_CACHE_HOME:-$HOME/.cache}/santiaguito/appimagetool"
    if [ ! -x "$tool" ]; then
        mkdir -p "$(dirname "$tool")"
        curl -fL -o "$tool" https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage
        chmod +x "$tool"
    fi
fi
dir=$(mktemp -d)
trap 'rm -rf "$dir"' EXIT
app="$dir/AppDir/app"

cargo build --locked --release --manifest-path engine/Cargo.toml --bin santiago-scenario-core
cargo build --locked --release --manifest-path viz/Cargo.toml

mkdir -p "$app/prebuilt/release" "$app/data" "$app/web" "$app/tools"
cp engine/target/release/santiago-scenario-core viz/target/release/santiago-miniature "$app/prebuilt/release/"
cp -r data/mvp data/replays "$app/data/"
cp -r web/data "$app/web/"
cp -r tools/santiago-mcp tools/terminal-font tools/city-chat.sh tools/city-terminal-linux.sh "$app/tools/"
cp SIMULATION_LIMITATIONS.md "$app/"
# The packed launcher uses the two binaries above in place of Cargo.
sed -e '/^target=/c\target=prebuilt' \
    -e '/^cargo build/,/--target-dir/d' \
    -e 's|^cargo run .*&$|prebuilt/release/santiago-miniature \&|' run.sh > "$app/run.sh"
chmod +x "$app/run.sh"
! grep -q cargo\ run "$app/run.sh"
git rev-parse HEAD > "$app/VERSION"
date +%s >> "$app/VERSION"

cat > "$dir/AppDir/AppRun" <<'RUN'
#!/usr/bin/env bash
set -eu
here=$(dirname "$(readlink -f "$0")")
home="${XDG_DATA_HOME:-$HOME/.local/share}/santiaguito/app"
if ! cmp -s "$here/app/VERSION" "$home/VERSION" 2>/dev/null; then
    echo "Instalando Santiaguito en $home ..."
    rm -rf "$home"
    mkdir -p "$(dirname "$home")"
    cp -r "$here/app" "$home"
fi
exec "$home/run.sh" "$@"
RUN
chmod +x "$dir/AppDir/AppRun"
cp project-logo.png "$dir/AppDir/santiaguito.png"
printf '[Desktop Entry]\nType=Application\nName=Santiaguito\nExec=AppRun\nIcon=santiaguito\nCategories=Education;\nTerminal=true\n' > "$dir/AppDir/santiaguito.desktop"
ARCH=x86_64 "$tool" --appimage-extract-and-run "$dir/AppDir" "$out"
echo "Wrote $out"

# Source from run.sh. Bevy uses physical pixels. Terminal uses desktop points.
city_mac_font_active() {
    # A fresh AppKit process sees changes made by the macOS font service.
    local active
    active=$(/usr/bin/osascript -l JavaScript <<'JS'
ObjC.import('AppKit')
$.NSFontManager.sharedFontManager.availableFonts.containsObject('JetBrainsMonoNFM-Regular') ? 'yes' : 'no'
JS
    ) && [[ $active == yes ]]
}

city_mac_prepare_font() {
    city_mac_font_active && return 0
    local font_file="JetBrainsMonoNerdFontMono-Regular.ttf"
    local font_dest="$HOME/Library/Fonts/$font_file" attempt
    if [[ -e $font_dest || -L $font_dest ]]; then
        printf 'Puedes activar %s en Catálogo Tipográfico para la próxima sesión.\n' "$font_dest" >&2
        return 1
    fi
    printf 'Se instalará la fuente del chat en %s.\n' "$font_dest" >&2
    mkdir -p "$HOME/Library/Fonts" && cp -n "$repo/tools/terminal-font/$font_file" "$font_dest" || return 1
    for ((attempt=0; attempt<20; attempt++)); do
        city_mac_font_active && return 0
        sleep 0.25
    done
    printf 'Puedes activar %s en Catálogo Tipográfico para la próxima sesión.\n' "$font_dest" >&2
    return 1
}

set_city_layout_mac() {
    local layout
    layout=$(/usr/bin/osascript -l JavaScript <<'JS'
ObjC.import('AppKit')
const screen = $.NSScreen.screens.firstObject
const work = screen.visibleFrame
const x = work.origin.x, width = work.size.width, height = work.size.height
const y = screen.frame.size.height - work.origin.y - height
const strip = Math.round(width * 0.35)
const title = $.NSWindow.frameRectForContentRectStyleMask(
    $.NSMakeRect(0, 0, 100, 100), $.NSWindowStyleMaskTitled).size.height - 100
// Winit creates a macOS window using a content rectangle below its title bar.
const pixels = [x, y + title, width - strip, height - title]
    .map(n => Math.round(n * screen.backingScaleFactor))
pixels.concat([x + width - strip, y, x + width, y + height].map(Math.round)).join(' ')
JS
    ) || return
    read -r city_x city_y city_width city_height terminal_left terminal_top terminal_right terminal_bottom <<<"$layout"
}

open_city_terminal_mac() {
    local owner font_name
    if city_mac_prepare_font; then
        font_name="JetBrainsMonoNFM-Regular"
    else
        font_name=""
        printf 'El chat usará la fuente de Terminal a 17 puntos.\n' >&2
    fi
    owner=$(/usr/bin/osascript - <<'APPLESCRIPT'
on run
    tell application "Terminal"
        launch
        set ownedTab to do script ""
        set ownedTTY to tty of ownedTab
        repeat with win in windows
            if tty of selected tab of win is ownedTTY then
                return (id of win as text) & " " & ownedTTY
            end if
        end repeat
    end tell
end run
APPLESCRIPT
    ) || return
    read -r city_terminal_window_id city_terminal_tty <<<"$owner"
    # Keep ownership for cleanup even if Terminal rejects a style change.
    /usr/bin/osascript - "$city_terminal_window_id" "$city_terminal_tty" \
        "$terminal_left" "$terminal_top" "$terminal_right" "$terminal_bottom" \
        "$repo" "$SANTIAGO_ENGINE" "${SANTIAGO_CHAT_CLI:-}" "$PATH" "$font_name" <<'APPLESCRIPT'
on run argv
    set {ownedID, ownedTTY, x1, y1, x2, y2, repoDir, engineURL, providerName, commandPath, chatFont} to argv
    set providerArg to "-u SANTIAGO_CHAT_CLI"
    if providerName is not "" then set providerArg to quoted form of ("SANTIAGO_CHAT_CLI=" & providerName)
    set cmd to "cd " & quoted form of repoDir & " && exec /usr/bin/env " & ¬
        providerArg & " " & quoted form of ("PATH=" & commandPath) & " " & ¬
        quoted form of ("SANTIAGO_ENGINE=" & engineURL) & " /bin/bash ./tools/city-chat.sh"
    tell application "Terminal"
        set win to window id (ownedID as integer)
        set ownedTab to first tab of win whose tty is ownedTTY
        -- Style the tab directly. The saved profile can also belong to other windows.
        tell ownedTab
            if chatFont is not "" then
                try
                    set font name to chatFont
                on error
                    log "El chat usará la fuente de Terminal a 17 puntos."
                end try
            end if
            set font size to 17
            set background color to {6168, 8481, 10280}
            set normal text color to {59110, 61166, 59881}
            set bold text color to {59110, 61166, 59881}
            set cursor color to {40349, 54741, 50372}
            set title displays device name to false
            set title displays file name to false
            set title displays shell path to false
            set title displays window size to false
            set custom title to "Santiago"
            set title displays custom title to true
        end tell
        set bounds of win to {x1 as integer, y1 as integer, x2 as integer, y2 as integer}
        do script cmd in ownedTab
        activate
    end tell
end run
APPLESCRIPT
}

cleanup_city_terminal_mac() {
    [[ -n ${city_terminal_window_id:-} ]] || return 0
    /usr/bin/osascript - "$city_terminal_window_id" "$city_terminal_tty" <<'APPLESCRIPT'
on run argv
    set {ownedID, ownedTTY} to argv
    if application "Terminal" is not running then return
    tell application "Terminal"
        if not (exists window id (ownedID as integer)) then return
        set win to window id (ownedID as integer)
        repeat with ownedTab in tabs of win
            if tty of ownedTab is ownedTTY then
                if busy of ownedTab then
                    my stopTTY(ownedTTY)
                end if
                try
                    if (count tabs of win) is 1 then close win
                end try
                return
            end if
        end repeat
    end tell
end run

on stopTTY(ownedTTY)
    do shell script "/usr/bin/pkill -HUP -t " & quoted form of (text 6 thru -1 of ownedTTY) & " || true"
end stopTTY
APPLESCRIPT
}

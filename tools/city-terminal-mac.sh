# Source from run.sh. Bevy uses physical pixels. Terminal uses desktop points.
open_city_terminal_mac() {
    local layout terminal_left terminal_top terminal_right terminal_bottom owner font_available
    local font_file="JetBrainsMonoNerdFontMono-Regular.ttf"
    local font_dest="$HOME/Library/Fonts/$font_file"
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
const hasFont = $.NSFontManager.sharedFontManager.availableFonts.containsObject('JetBrainsMonoNFM-Regular')
pixels.concat([x + width - strip, y, x + width, y + height].map(Math.round), hasFont ? "yes" : "no").join(' ')
JS
    ) || return
    read -r city_x city_y city_width city_height terminal_left terminal_top terminal_right terminal_bottom font_available <<<"$layout"
    if [[ $font_available != yes ]]; then
        if [[ -e $font_dest || -L $font_dest ]]; then
            printf 'Ya existe %s. Activa JetBrains Mono Nerd Font Mono en Catálogo Tipográfico.\n' "$font_dest" >&2
            return 1
        fi
        printf 'Se instalará la fuente del chat en %s.\n' "$font_dest" >&2
        mkdir -p "$HOME/Library/Fonts" && cp -n "$repo/tools/terminal-font/$font_file" "$font_dest" || return 1
    fi
    owner=$(/usr/bin/osascript - "$repo" "$SANTIAGO_ENGINE" "${SANTIAGO_CHAT_CLI:-}" "$PATH" <<'APPLESCRIPT'
on run argv
    set {repoDir, engineURL, providerName, commandPath} to argv
    set providerArg to "-u SANTIAGO_CHAT_CLI"
    if providerName is not "" then set providerArg to quoted form of ("SANTIAGO_CHAT_CLI=" & providerName)
    set cmd to "cd " & quoted form of repoDir & " && exec /usr/bin/env " & ¬
        providerArg & " " & quoted form of ("PATH=" & commandPath) & " " & ¬
        quoted form of ("SANTIAGO_ENGINE=" & engineURL) & " /bin/bash ./tools/city-chat.sh"
    tell application "Terminal"
        launch
        set ownedTab to do script cmd
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
        "$terminal_left" "$terminal_top" "$terminal_right" "$terminal_bottom" <<'APPLESCRIPT'
on run argv
    set {ownedID, ownedTTY, x1, y1, x2, y2} to argv
    tell application "Terminal"
        set win to window id (ownedID as integer)
        set ownedTab to first tab of win whose tty is ownedTTY
        -- Style the tab directly. The saved profile can also belong to other windows.
        tell ownedTab
            set font name to "JetBrainsMonoNFM-Regular"
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

# Sourced by run.sh. The native viewer uses the physical coordinates set here.
open_city_terminal_linux() {
    local x=0 y=0 width height i3_rect= geometry term_width
    local template="santiago-chat-$$" city_class="SantiagoCity$$"
    if [[ -z ${DISPLAY:-} ]] || ! command -v xterm >/dev/null; then
        echo 'The Linux launcher needs an X11 display and xterm.' >&2
        return 1
    fi

    # i3 reports the focused monitor's usable rectangle, excluding the bar.
    if command -v i3-msg >/dev/null && i3_rect=$(i3-msg -t get_workspaces 2>/dev/null); then
        read -r x y width height < <(node -e '
          const r = JSON.parse(require("fs").readFileSync(0)).find(w => w.focused).rect;
          console.log(r.x, r.y, r.width, r.height)' <<< "$i3_rect")
    else
        geometry=$(xrandr --current | awk '/ connected/ {
            for (i=3; i<=NF; i++) if ($i ~ /^[0-9]+x[0-9]+[+][0-9]+[+][0-9]+$/) {
                if (!first || $3 == "primary") first=$i; if ($3 == "primary") exit
            }
        } END { print first }')
        if [[ ! $geometry =~ ^([0-9]+)x([0-9]+)\+([0-9]+)\+([0-9]+)$ ]]; then
            echo 'Cannot read the X11 monitor geometry.' >&2
            return 1
        fi
        width=${BASH_REMATCH[1]} height=${BASH_REMATCH[2]}
        x=${BASH_REMATCH[3]} y=$((BASH_REMATCH[4] + 32))
        height=$((height - 72))
    fi

    city_x=$x city_y=$y city_width=$((width * 65 / 100)) city_height=$height
    term_width=$((width - city_width))
    export SANTIAGO_CITY_WINDOW_CLASS="$city_class"
    # Xterm takes rows and columns in -geometry. Request the strip in pixels
    # through its pty before the CLI starts, so font DPI does not change the split.
    FONTCONFIG_FILE="$repo/tools/terminal-font/fonts.conf" \
        xterm -name "$template" -T Santiago -u8 -fa 'JetBrainsMono Nerd Font Mono' -fs 17 \
            -bg '#182128' -fg '#e6eee9' -cr '#9dd5c4' +sb -xrm '*toolBar: false' -b 10 \
            -geometry "+$((x + city_width))+$y" -e bash -c '
                printf "\033[4;%d;%dt" "$1" "$2"
                exec bash "$3"
            ' santiago "$height" "$term_width" "$repo/tools/city-chat.sh" &
    terminal_pid=$!

    if [[ -n $i3_rect ]]; then
        # i3 ignores requested window geometry when it first tiles a window.
        # Match only this launch. Cargo can build for a while before Bevy opens.
        (
            for spec in "instance $template $((x + city_width)) $term_width" \
                        "class $city_class $x $city_width"; do
                read -r field tag left span <<< "$spec"
                while kill -0 "$terminal_pid" 2>/dev/null; do
                    tree=$(i3-msg -t get_tree) || break
                    if [[ $tree == *\"$tag\"* ]]; then
                        i3-msg -q "[$field=\"^$tag\$\"] floating enable, border none, resize set $span px $height px, move position $left px $y px"
                        break
                    fi
                    sleep 0.5
                done
            done
        ) &
        terminal_layout_pid=$!
    fi
}

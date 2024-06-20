#!/usr/bin/bash
# This is very specific for my setup. The program will run but MIDI
# probably won't work :(

shopt -s expand_aliases

main() {
    pw_match() {
        pw-cli ls | grep -B 10 "$3" | perl -0 -pe "s/.+$1(\S+)$2.*/\$1/sg"
    }

    pw_id() {
        pw_match "id " "," "$1"
    }
    pw_port() {
        pw_match "port\\.id = \"" "\"" "$1"
    }

    MIDI_BRIDGE="$(pw_id 'media\.class = "Midi/Bridge"')"
    MIDI_KB="$(pw_port 'at usb.*playback_0')"
    DAGRID="$(pw_id 'node\.name = "dagrid"')"
    DAGRID_IN="$(pw_port 'dagrid:midi_input')"

    pw-cli cl -m "$MIDI_BRIDGE" "$MIDI_KB" "$DAGRID" "$DAGRID_IN"
}

cargo build --release
target/release/dagrid -p 128 & main && kill $!

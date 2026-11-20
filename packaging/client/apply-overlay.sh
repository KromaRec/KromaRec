#!/bin/sh
set -eu
if [ "$#" -ne 1 ]; then
    echo "Usage: $0 /path/to/recflare-patch" >&2
    exit 2
fi
here=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
checkout=$1
test "$(git -C "$checkout" rev-parse HEAD)" = 68f42eb6614e4fa788603bc0dd8124527c382e30
test ! -e "$checkout/Patches/KromaRecPhotonTicket.cs"
git -C "$checkout" apply --check "$here/recflare-20230414.patch"
git -C "$checkout" apply "$here/recflare-20230414.patch"
cp "$here/KromaRecPhotonTicket.cs" "$checkout/Patches/"

#!/bin/sh
# Build the admission extension from the exact Luxon 1.5 source.
set -eu
here=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
if [ "$#" -ne 1 ]; then
    echo "Usage: $0 /path/to/fresh-luxon-build-directory" >&2
    exit 2
fi
mkdir -p "$1"
work=$(CDPATH='' cd -- "$1" && pwd)
git clone --branch v1.5 --depth 1 https://github.com/niansa/LuxonServer.git "$work/source"
test "$(git -C "$work/source" rev-parse HEAD)" = e4bab98d8d6b3aee9ecf4035309a78a59b584f93
git -C "$work/source" submodule update --init --depth 1 Luxon tracy
git -C "$work/source" apply "$here/v1.5-admission.patch"
git -C "$work/source" apply "$here/v1.5-live-admission.patch"
cp "$here/kromarec_gate.hpp" "$work/source/src/"
cmake -S "$work/source" -B "$work/build" -DCMAKE_BUILD_TYPE=Release \
    -DLUXON_SERVER_ENABLE_WEBSERVER=OFF -DLUXON_SERVER_USE_SPDLOG=OFF \
    -DLUXON_SERVER_USE_SANMAKE=OFF -DLUXON_SERVER_ENABLE_MULTIPROCESSING=OFF \
    -DLUXON_SERVER_ENABLE_SETTINGS_DATABASE=OFF
cmake --build "$work/build" --target luxon_server --parallel "${LUXON_BUILD_JOBS:-2}"

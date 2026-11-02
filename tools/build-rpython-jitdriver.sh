#!/usr/bin/env bash
set -euo pipefail

if (( $# != 2 )); then
    echo "usage: $0 RUNTIME_ROOT BUILD_DIR" >&2
    exit 64
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUNTIME_ROOT="$(realpath -m "$1")"
BUILD_DIR="$(realpath -m "$2")"

if [[ "$(uname -s)" != Linux || "$(uname -m)" != x86_64 ]]; then
    echo "the pinned RPython toolchain currently supports Linux x86_64 only" >&2
    exit 69
fi

PYPY2_ARCHIVE=pypy2.7-v8.0.1-linux64.tar.gz
PYPY2_SHA256=78674cdf95eb4cb1dd5a9b68a70200387111cb29467f8a472ab7cf07a251653c
PYPY2_URL="https://downloads.python.org/pypy/$PYPY2_ARCHIVE"
PYPY2_DIR=pypy2.7-v8.0.1-linux64

RPYTHON_ARCHIVE=pypy3.12-v8.0.1-src.tar.gz
RPYTHON_SHA256=9d5c82d39d590dc04677f4721738b21d670570cdc9824c003836bafd38556cda
RPYTHON_URL="https://downloads.python.org/pypy/$RPYTHON_ARCHIVE"
RPYTHON_DIR=pypy3.12-v8.0.1-src

mkdir -p "$RUNTIME_ROOT" "$BUILD_DIR"

acquire() {
    local url=$1 archive=$2 sha256=$3 directory=$4
    local path="$RUNTIME_ROOT/$archive"
    if [[ ! -f "$path" ]]; then
        curl --fail --location --retry 3 --output "$path.part" "$url"
        mv "$path.part" "$path"
    fi
    printf '%s  %s\n' "$sha256" "$path" | sha256sum --check --status || {
        echo "checksum mismatch: $path" >&2
        rm -f "$path"
        exit 65
    }
    if [[ ! -d "$RUNTIME_ROOT/$directory" ]]; then
        tar -C "$RUNTIME_ROOT" -xzf "$path"
    fi
}

acquire "$PYPY2_URL" "$PYPY2_ARCHIVE" "$PYPY2_SHA256" "$PYPY2_DIR"
acquire "$RPYTHON_URL" "$RPYTHON_ARCHIVE" "$RPYTHON_SHA256" "$RPYTHON_DIR"

SCCACHE_BIN="$(command -v sccache)"
CC_BIN="${MUSEUM_RPYTHON_CC:-$(command -v gcc)}"
UV_BIN="$(command -v uv)"
RG_BIN="$(command -v rg)"

cat > "$BUILD_DIR/cc-sccache" <<EOF_CC
#!/usr/bin/env bash
set -euo pipefail
export SCCACHE_START_SERVER=0
export SCCACHE_NO_AUTO_START_SERVER=1
exec "$SCCACHE_BIN" "$CC_BIN" "\$@"
EOF_CC
chmod +x "$BUILD_DIR/cc-sccache"

TARGET="$BUILD_DIR/target_hashwx.py"
OUTPUT="$BUILD_DIR/hashwx-rpython-jit"
LOG="$BUILD_DIR/translate.log"

"$UV_BIN" run --no-project --python 3 python \
    "$ROOT/workloads/hashwx-fixed-v1/implementations/rpython-jitdriver/generate_target.py" \
    --output "$TARGET"

mkdir -p "$BUILD_DIR/udir"
export PYPY_USESSION_DIR="$BUILD_DIR/udir"
export PYPY_USESSION_BASENAME=museum-jit-rpython
export PYPY_USESSION_KEEP=1
export SCCACHE_START_SERVER=0
export SCCACHE_NO_AUTO_START_SERVER=1

"$SCCACHE_BIN" --show-stats > "$BUILD_DIR/sccache-before.txt"
set +e
"$RUNTIME_ROOT/$PYPY2_DIR/bin/pypy" \
    "$RUNTIME_ROOT/$RPYTHON_DIR/rpython/bin/rpython" \
    -Ojit --make-jobs=1 --output="$OUTPUT" --cc="$BUILD_DIR/cc-sccache" \
    "$TARGET" 2>&1 | tee "$LOG"
translate_status=${PIPESTATUS[0]}
set -e
"$SCCACHE_BIN" --show-stats > "$BUILD_DIR/sccache-after.txt"

if "$RG_BIN" -qi 'compiling locally instead|failed to connect to sccache server|server looks like it shut down unexpectedly' "$LOG"; then
    rm -f "$OUTPUT"
    echo "translation observed an sccache local-fallback condition; rejecting artifact" >&2
    exit 70
fi
if (( translate_status != 0 )); then
    rm -f "$OUTPUT"
    exit "$translate_status"
fi

"$OUTPUT" 0 1 1 | tee "$BUILD_DIR/smoke.txt"
printf 'translated executable: %s\n' "$OUTPUT"

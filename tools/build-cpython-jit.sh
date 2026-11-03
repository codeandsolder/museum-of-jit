#!/usr/bin/env bash
set -euo pipefail

if (( $# != 3 )); then
    echo "usage: $0 RUNTIME_ROOT BUILD_DIR SYSROOT" >&2
    exit 64
fi

RUNTIME_ROOT="$(realpath -m "$1")"
BUILD_DIR="$(realpath -m "$2")"
SYSROOT="$(realpath "$3")"
PREFIX="$BUILD_DIR/install"
OBJDIR="$BUILD_DIR/obj"

if [[ "$(uname -s)" != Linux || "$(uname -m)" != x86_64 ]]; then
    echo "the pinned CPython JIT toolchain currently supports Linux x86_64 only" >&2
    exit 69
fi
if [[ ! -d "$SYSROOT/usr/include" || ! -d "$SYSROOT/usr/lib" ]]; then
    echo "SYSROOT does not look like a complete Linux sysroot: $SYSROOT" >&2
    exit 66
fi

PYTHON_ARCHIVE=Python-3.15.2.tar.xz
PYTHON_SHA256=8b5536d280142ecf6cdfe15d12a21ba1c65e300bbf7b28a8d0cdb849575797de
PYTHON_URL="https://www.python.org/ftp/python/3.15.2/$PYTHON_ARCHIVE"
PYTHON_DIR=Python-3.15.2

LLVM_ARCHIVE=LLVM-21.1.8-Linux-X64.tar.xz
LLVM_SHA256=b3b7f2801d15d50736acea3c73982994d025b01c2f035b91ae3b49d1b575732b
LLVM_URL="https://github.com/llvm/llvm-project/releases/download/llvmorg-21.1.8/$LLVM_ARCHIVE"
LLVM_DIR=LLVM-21.1.8-Linux-X64

for tool in curl tar sha256sum uv rg file make pkg-config sccache cmp; do
    command -v "$tool" >/dev/null || {
        echo "required tool not found: $tool" >&2
        exit 69
    }
done

mkdir -p "$RUNTIME_ROOT" "$BUILD_DIR"

acquire_xz() {
    local url=$1 archive=$2 sha256=$3 directory=$4 sentinel=$5
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
    if [[ ! -e "$RUNTIME_ROOT/$directory/$sentinel" ]]; then
        local extract_tmp
        extract_tmp="$(mktemp -d "$RUNTIME_ROOT/.extract-${directory}.XXXXXX")"
        if ! tar -C "$extract_tmp" -xJf "$path"; then
            rm -rf "$extract_tmp"
            exit 65
        fi
        if [[ ! -e "$extract_tmp/$directory/$sentinel" ]]; then
            echo "archive did not contain expected $directory/$sentinel" >&2
            rm -rf "$extract_tmp"
            exit 65
        fi
        rm -rf "${RUNTIME_ROOT:?}/${directory:?}"
        mv "$extract_tmp/$directory" "$RUNTIME_ROOT/$directory"
        rm -rf "$extract_tmp"
    fi
}

acquire_xz "$PYTHON_URL" "$PYTHON_ARCHIVE" "$PYTHON_SHA256" "$PYTHON_DIR" configure
acquire_xz "$LLVM_URL" "$LLVM_ARCHIVE" "$LLVM_SHA256" "$LLVM_DIR" bin/clang
for llvm_tool in clang llvm-readobj llvm-objdump llvm-dwarfdump; do
    [[ -x "$RUNTIME_ROOT/$LLVM_DIR/bin/$llvm_tool" ]] || {
        echo "LLVM archive is incomplete: missing bin/$llvm_tool" >&2
        exit 65
    }
done

SCCACHE_BIN="$(command -v sccache)"
UV_BIN="$(command -v uv)"
RG_BIN="$(command -v rg)"
FILE_BIN="$(command -v file)"

pick_sysroot_tool() {
    local name=$1
    if [[ -x "$SYSROOT/usr/bin/$name" ]]; then
        printf '%s\n' "$SYSROOT/usr/bin/$name"
    else
        command -v "$name"
    fi
}

CC_REAL="${MUSEUM_CPYTHON_CC:-$(pick_sysroot_tool gcc)}"
CXX_REAL="${MUSEUM_CPYTHON_CXX:-$(pick_sysroot_tool g++)}"
AR_REAL="${MUSEUM_CPYTHON_AR:-$(pick_sysroot_tool gcc-ar)}"
RANLIB_REAL="${MUSEUM_CPYTHON_RANLIB:-$(pick_sysroot_tool gcc-ranlib)}"
NM_REAL="${MUSEUM_CPYTHON_NM:-$(pick_sysroot_tool gcc-nm)}"
STRIP_REAL="${MUSEUM_CPYTHON_STRIP:-$(command -v strip)}"
OBJDUMP_REAL="${MUSEUM_CPYTHON_OBJDUMP:-$(command -v objdump)}"
MAKE_JOBS="${MUSEUM_CPYTHON_MAKE_JOBS:-4}"
PROBE_PORT="${MUSEUM_SCCACHE_PROBE_PORT:-}"
JIT_PORT="${MUSEUM_SCCACHE_JIT_PORT:-$PROBE_PORT}"

for tool in "$CC_REAL" "$CXX_REAL" "$AR_REAL" "$RANLIB_REAL" "$NM_REAL" "$STRIP_REAL" "$OBJDUMP_REAL"; do
    [[ -x "$tool" ]] || {
        echo "required tool is not executable: $tool" >&2
        exit 69
    }
    "$tool" --version >/dev/null 2>&1 || {
        echo "required tool cannot run on the build host: $tool" >&2
        exit 69
    }
done

if [[ "$BUILD_DIR" == / ]]; then
    echo "refusing to use / as BUILD_DIR" >&2
    exit 64
fi

SYSROOT_LIBC="$(find "$SYSROOT/lib" "$SYSROOT/usr/lib" -type f -name 'libc.so.6' -print -quit 2>/dev/null || true)"
SYSROOT_FEATURES="$SYSROOT/usr/include/features.h"
BUILD_KEY="$BUILD_DIR/build-key.txt"
BUILD_KEY_NEXT="$BUILD_DIR/build-key.next"
{
    printf 'python_sha256=%s\n' "$PYTHON_SHA256"
    printf 'llvm_sha256=%s\n' "$LLVM_SHA256"
    printf 'sysroot=%s\n' "$SYSROOT"
    if [[ -n "$SYSROOT_LIBC" ]]; then sha256sum "$SYSROOT_LIBC"; fi
    if [[ -f "$SYSROOT_FEATURES" ]]; then sha256sum "$SYSROOT_FEATURES"; fi
    printf 'cc=%s\n' "$CC_REAL"
    "$CC_REAL" --version | head -n 1
    printf 'cxx=%s\n' "$CXX_REAL"
    "$CXX_REAL" --version | head -n 1
    printf 'configure=--enable-optimizations --with-lto=full --enable-experimental-jit=yes --with-ensurepip=no\n'
    printf 'cflags_jit=--sysroot=%s\n' "$SYSROOT"
} > "$BUILD_KEY_NEXT"

if [[ -f "$BUILD_KEY" ]] && cmp -s "$BUILD_KEY" "$BUILD_KEY_NEXT"; then
    reuse_build=1
    rm -f "$BUILD_KEY_NEXT"
else
    reuse_build=0
    rm -rf "${OBJDIR:?}" "${PREFIX:?}"
    mv "$BUILD_KEY_NEXT" "$BUILD_KEY"
fi

write_compiler_wrapper() {
    local path=$1 compiler=$2
    cat > "$path" <<EOF_WRAPPER
#!/usr/bin/env bash
set -euo pipefail
export SCCACHE_START_SERVER=0 SCCACHE_NO_AUTO_START_SERVER=1
probe_port='$PROBE_PORT'
if [[ -n "\$probe_port" ]]; then
    for arg in "\$@"; do
        case "\$arg" in
            *conftest*) export SCCACHE_SERVER_PORT="\$probe_port"; unset SCCACHE_SERVER_UDS; break ;;
        esac
    done
fi
exec "$SCCACHE_BIN" "$compiler" --sysroot="$SYSROOT" "\$@"
EOF_WRAPPER
    chmod +x "$path"
}

write_compiler_wrapper "$BUILD_DIR/cc-sccache" "$CC_REAL"
write_compiler_wrapper "$BUILD_DIR/cxx-sccache" "$CXX_REAL"

LLVM_FACADE="$BUILD_DIR/llvm-tools"
mkdir -p "$LLVM_FACADE/bin"
cat > "$LLVM_FACADE/bin/clang" <<EOF_CLANG
#!/usr/bin/env bash
set -euo pipefail
export SCCACHE_START_SERVER=0 SCCACHE_NO_AUTO_START_SERVER=1
jit_port='$JIT_PORT'
if [[ -n "\$jit_port" ]]; then
    export SCCACHE_SERVER_PORT="\$jit_port"
    unset SCCACHE_SERVER_UDS
fi
exec "$SCCACHE_BIN" "$RUNTIME_ROOT/$LLVM_DIR/bin/clang" "\$@"
EOF_CLANG
chmod +x "$LLVM_FACADE/bin/clang"
ln -sfn clang "$LLVM_FACADE/bin/clang-21"
for tool in llvm-readobj llvm-objdump llvm-dwarfdump; do
    ln -sfn "$RUNTIME_ROOT/$LLVM_DIR/bin/$tool" "$LLVM_FACADE/bin/$tool"
done

cat > "$BUILD_DIR/build-python" <<EOF_PYTHON
#!/usr/bin/env bash
set -euo pipefail
exec "$UV_BIN" run --no-project --python 3.14 python "\$@"
EOF_PYTHON
chmod +x "$BUILD_DIR/build-python"

mkdir -p "$OBJDIR"

export SCCACHE_START_SERVER=0 SCCACHE_NO_AUTO_START_SERVER=1
export LLVM_VERSION=21
export LLVM_TOOLS_INSTALL_DIR="$LLVM_FACADE"
export CFLAGS_JIT="--sysroot=$SYSROOT"
export PYTHON_FOR_REGEN="$BUILD_DIR/build-python"
export CC="$BUILD_DIR/cc-sccache"
export CXX="$BUILD_DIR/cxx-sccache"
export AR="$AR_REAL" RANLIB="$RANLIB_REAL" NM="$NM_REAL" STRIP="$STRIP_REAL"
export PKG_CONFIG_SYSROOT_DIR="$SYSROOT"
export PKG_CONFIG_LIBDIR="$SYSROOT/usr/lib/x86_64-linux-gnu/pkgconfig:$SYSROOT/usr/lib/pkgconfig:$SYSROOT/usr/share/pkgconfig"
unset PKG_CONFIG_PATH

"$SCCACHE_BIN" --show-stats > "$BUILD_DIR/sccache-before.txt"
if [[ -n "$PROBE_PORT" ]]; then
    SCCACHE_SERVER_PORT="$PROBE_PORT" SCCACHE_START_SERVER=0 SCCACHE_NO_AUTO_START_SERVER=1 \
        "$SCCACHE_BIN" --show-stats > "$BUILD_DIR/sccache-probe-before.txt"
fi

CONFIGURE_LOG="$BUILD_DIR/configure.log"
BUILD_LOG="$BUILD_DIR/build.log"
pushd "$OBJDIR" >/dev/null
if (( reuse_build == 0 )) || [[ ! -f Makefile || ! -f pyconfig.h ]]; then
    set +e
    "$RUNTIME_ROOT/$PYTHON_DIR/configure" \
        --prefix="$PREFIX" \
        --enable-optimizations \
        --with-lto=full \
        --enable-experimental-jit=yes \
        --with-ensurepip=no \
        2>&1 | tee "$CONFIGURE_LOG"
    configure_status=${PIPESTATUS[0]}
    set -e
    if "$RG_BIN" -qi 'sccache: error:|compiling locally instead|failed to connect to sccache server|server looks like it shut down unexpectedly' "$CONFIGURE_LOG"; then
        popd >/dev/null
        echo "configure observed an sccache infrastructure failure; rejecting feature detection" >&2
        exit 70
    fi
    if (( configure_status != 0 )); then
        popd >/dev/null
        exit "$configure_status"
    fi
else
    echo "reusing matching configure output in $OBJDIR"
fi

set +e
{
    make -j"$MAKE_JOBS"
    make -j"$MAKE_JOBS" install
} 2>&1 | tee "$BUILD_LOG"
build_status=${PIPESTATUS[0]}
set -e
popd >/dev/null

"$SCCACHE_BIN" --show-stats > "$BUILD_DIR/sccache-after.txt"
if [[ -n "$PROBE_PORT" ]]; then
    SCCACHE_SERVER_PORT="$PROBE_PORT" SCCACHE_START_SERVER=0 SCCACHE_NO_AUTO_START_SERVER=1 \
        "$SCCACHE_BIN" --show-stats > "$BUILD_DIR/sccache-probe-after.txt"
fi

if "$RG_BIN" -qi 'sccache: error:|compiling locally instead|failed to connect to sccache server|server looks like it shut down unexpectedly' "$BUILD_LOG"; then
    echo "build observed an sccache infrastructure failure; rejecting artifact" >&2
    exit 70
fi
if (( build_status != 0 )); then
    exit "$build_status"
fi

PYTHON_BIN="$PREFIX/bin/python3.15"
[[ -x "$PYTHON_BIN" ]] || {
    echo "installed CPython executable missing: $PYTHON_BIN" >&2
    exit 70
}

"$PYTHON_BIN" -VV | tee "$BUILD_DIR/python-version.txt"
PYTHON_JIT=1 "$PYTHON_BIN" - <<'PY' | tee "$BUILD_DIR/jit-smoke.txt"
import _opcode
import sys

assert sys._jit.is_available(), "runtime does not expose tier-2/JIT support"
assert sys._jit.is_enabled(), "PYTHON_JIT=1 did not enable the JIT"

def hot(n):
    value = 0
    while n:
        value = (value * 33 + n) & 0xFFFFFFFFFFFFFFFF
        n -= 1
    return value

for _ in range(12_000):
    hot(64)

executors = []
code = hot.__code__
for offset in range(0, len(code.co_code), 2):
    try:
        executors.append(_opcode.get_executor(code, offset))
    except ValueError:
        pass
print("jit_available", sys._jit.is_available())
print("jit_enabled", sys._jit.is_enabled())
print("jit_executors", len({id(executor) for executor in executors}))
assert executors, "hot-loop smoke test installed no tier-2 executor"
PY

GLIBC_VERSIONS="$BUILD_DIR/glibc-versions.txt"
: > "$GLIBC_VERSIONS"
while IFS= read -r -d '' artifact; do
    if "$FILE_BIN" -b "$artifact" | "$RG_BIN" -q '^ELF '; then
        "$OBJDUMP_REAL" -T "$artifact" 2>/dev/null | "$RG_BIN" -o 'GLIBC_[0-9]+(\.[0-9]+)+' || true
    fi
done < <(find "$PREFIX" -type f -print0) | sort -Vu > "$GLIBC_VERSIONS"

max_glibc="$(sed -n 's/^GLIBC_//p' "$GLIBC_VERSIONS" | sort -V | tail -n 1)"
printf 'max_glibc=%s\n' "${max_glibc:-none}" | tee "$BUILD_DIR/abi-summary.txt"
if [[ -n "${MUSEUM_CPYTHON_MAX_GLIBC:-}" && -n "$max_glibc" ]]; then
    highest="$(printf '%s\n%s\n' "$MUSEUM_CPYTHON_MAX_GLIBC" "$max_glibc" | sort -V | tail -n 1)"
    if [[ "$highest" != "$MUSEUM_CPYTHON_MAX_GLIBC" ]]; then
        echo "artifact requires GLIBC_$max_glibc, above requested GLIBC_$MUSEUM_CPYTHON_MAX_GLIBC" >&2
        exit 70
    fi
fi

{
    printf 'python_version=3.15.2\n'
    printf 'python_sha256=%s\n' "$PYTHON_SHA256"
    printf 'llvm_version=21.1.8\n'
    printf 'llvm_sha256=%s\n' "$LLVM_SHA256"
    printf 'sysroot=%s\n' "$SYSROOT"
    printf 'cc=%s\n' "$CC_REAL"
    "$CC_REAL" --version | head -n 1 | sed 's/^/cc_version=/'
    "$RUNTIME_ROOT/$LLVM_DIR/bin/clang" --version | head -n 1 | sed 's/^/clang_version=/'
    printf 'make_jobs=%s\n' "$MAKE_JOBS"
    printf 'max_glibc=%s\n' "${max_glibc:-none}"
} > "$BUILD_DIR/build-manifest.txt"

printf 'installed CPython JIT: %s\n' "$PREFIX"

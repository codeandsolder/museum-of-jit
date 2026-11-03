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
OBJDIR="$(realpath -m "${MUSEUM_CPYTHON_OBJDIR:-$BUILD_DIR/obj}")"

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

for tool in curl tar sha256sum uv rg file make pkg-config sccache cmp sleep mktemp cp patch; do
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
for llvm_tool in clang clang++ llvm-profdata llvm-readobj llvm-objdump llvm-dwarfdump; do
    [[ -x "$RUNTIME_ROOT/$LLVM_DIR/bin/$llvm_tool" ]] || {
        echo "LLVM archive is incomplete: missing bin/$llvm_tool" >&2
        exit 65
    }
done

# Clang 21 native InstrProf bypasses the failing GCC 14 PGO path.
# Always use the unmodified, checksum-pinned CPython release sources.
ABSTRACT_SOURCE="$RUNTIME_ROOT/$PYTHON_DIR/Objects/abstract.c"
ABSTRACT_UPSTREAM_SHA256=000c079b34d6d9a6074cead81c04ab3e46e3682942acdce6786096bc43fc6b41
printf '%s  %s\n' "$ABSTRACT_UPSTREAM_SHA256" "$ABSTRACT_SOURCE" |
    sha256sum --check --status || { echo "CPython abstract.c differs from upstream" >&2; exit 65; }

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

CC_REAL="$RUNTIME_ROOT/$LLVM_DIR/bin/clang"
CXX_REAL="$RUNTIME_ROOT/$LLVM_DIR/bin/clang++"
LLD_REAL=/usr/bin/ld.lld
[[ -x "$LLD_REAL" ]] || { echo "required LLD missing: $LLD_REAL" >&2; exit 69; }
LLD_SHA256="$(sha256sum "$(realpath "$LLD_REAL")" | cut -d' ' -f1)"
LLD_VERSION="$("$LLD_REAL" --version)"
CLANG_SHA256="$(sha256sum "$(realpath "$CC_REAL")" | cut -d' ' -f1)"
MAKE_JOBS="${MUSEUM_CPYTHON_MAKE_JOBS:-4}"
PROBE_PORT="${MUSEUM_SCCACHE_PROBE_PORT:-}"
JIT_PORT="${MUSEUM_SCCACHE_JIT_PORT:-$PROBE_PORT}"

for tool in "$CC_REAL" "$CXX_REAL"; do
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
case "$OBJDIR" in
    /)
        echo "refusing unsafe OBJDIR: $OBJDIR" >&2
        exit 64
        ;;
esac
for protected_path in "$BUILD_DIR" "$RUNTIME_ROOT" "$SYSROOT"; do
    case "$protected_path" in
        "$OBJDIR"|"$OBJDIR"/*)
            echo "refusing OBJDIR that contains protected path $protected_path: $OBJDIR" >&2
            exit 64
            ;;
    esac
done

SYSROOT_LIBC="$(find "$SYSROOT/lib" "$SYSROOT/usr/lib" -type f -name 'libc.so.6' -print -quit 2>/dev/null || true)"
SYSROOT_FEATURES="$SYSROOT/usr/include/features.h"
SYSROOT_DPKG_STATUS="$SYSROOT/var/lib/dpkg/status"
SYSROOT_LOADER="$SYSROOT/usr/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2"
SYSROOT_BINUTIL_LIBPATH="$SYSROOT/usr/lib/x86_64-linux-gnu:$SYSROOT/lib/x86_64-linux-gnu:$SYSROOT/usr/lib:$SYSROOT/lib"
SYSROOT_BFD_LIB="$(find "$SYSROOT/usr/lib/x86_64-linux-gnu" -maxdepth 1 -type f -name 'libbfd-*-system.so' -print -quit 2>/dev/null || true)"

sysroot_binutil_target() {
    local name=$1
    case "$name" in
        ld|ld.bfd) printf '%s/usr/bin/x86_64-linux-gnu-ld.bfd\n' "$SYSROOT" ;;
        *) printf '%s/usr/bin/x86_64-linux-gnu-%s\n' "$SYSROOT" "$name" ;;
    esac
}

[[ -x "$SYSROOT_LOADER" && -f "$SYSROOT_BFD_LIB" ]] || {
    echo "sysroot is missing its dynamic loader or private binutils library" >&2
    exit 69
}
for tool in as ld ar ranlib nm strip objdump readelf; do
    target="$(sysroot_binutil_target "$tool")"
    [[ -x "$target" ]] || {
        echo "sysroot binutil is missing or not executable: $target" >&2
        exit 69
    }
    "$SYSROOT_LOADER" --library-path "$SYSROOT_BINUTIL_LIBPATH" "$target" --version >/dev/null 2>&1 || {
        echo "sysroot binutil cannot run through the sysroot loader: $target" >&2
        exit 69
    }
done
BINUTILS_VERSION="$("$SYSROOT_LOADER" --library-path "$SYSROOT_BINUTIL_LIBPATH" \
    "$(sysroot_binutil_target as)" --version | head -n 1)"
read -r BINUTILS_FINGERPRINT _ < <(
    {
        if [[ -f "$SYSROOT_DPKG_STATUS" ]]; then sha256sum "$SYSROOT_DPKG_STATUS"; fi
        sha256sum "$SYSROOT_LOADER" "$SYSROOT_BFD_LIB"
        for tool in as ld ar ranlib nm strip objdump readelf; do
            sha256sum "$(sysroot_binutil_target "$tool")"
        done
    } | sha256sum
)
SCCACHE_C_CACHE_BUSTER="museum-cpython-clang-$CLANG_SHA256-lld-$LLD_SHA256-binutils-$BINUTILS_FINGERPRINT"

read -r BUILDER_SHA256 _ < <(sha256sum "$(realpath "$0")")
BUILD_KEY="$BUILD_DIR/build-key.txt"
BUILD_KEY_NEXT="$BUILD_DIR/build-key.next"
{
    printf 'builder_sha256=%s\n' "$BUILDER_SHA256"
    printf 'python_sha256=%s\n' "$PYTHON_SHA256"
    printf 'abstract_upstream_sha256=%s\n' "$ABSTRACT_UPSTREAM_SHA256"
    printf 'clang_sha256=%s\n' "$CLANG_SHA256"
    printf 'lld_sha256=%s\n' "$LLD_SHA256"
    printf 'lld_version=%s\n' "$LLD_VERSION"
    printf 'llvm_sha256=%s\n' "$LLVM_SHA256"
    printf 'sysroot=%s\n' "$SYSROOT"
    if [[ -n "$SYSROOT_LIBC" ]]; then sha256sum "$SYSROOT_LIBC"; fi
    if [[ -f "$SYSROOT_FEATURES" ]]; then sha256sum "$SYSROOT_FEATURES"; fi
    if [[ -f "$SYSROOT_DPKG_STATUS" ]]; then sha256sum "$SYSROOT_DPKG_STATUS"; fi
    sha256sum "$SYSROOT_LOADER" "$SYSROOT_BFD_LIB"
    for tool in as ld ar ranlib nm strip objdump readelf; do
        sha256sum "$(sysroot_binutil_target "$tool")"
    done
    printf 'binutils_version=%s\n' "$BINUTILS_VERSION"
    printf 'binutils_fingerprint=%s\n' "$BINUTILS_FINGERPRINT"
    printf 'sccache_c_custom_cache_buster=%s\n' "$SCCACHE_C_CACHE_BUSTER"
    printf 'cc=%s\n' "$CC_REAL"
    "$CC_REAL" --version | head -n 1
    printf 'cxx=%s\n' "$CXX_REAL"
    "$CXX_REAL" --version | head -n 1
    printf 'configure=--enable-optimizations --with-lto=full --enable-experimental-jit=yes --with-ensurepip=no --with-openssl=%s/usr\n' "$SYSROOT"
    printf 'cflags_jit=--sysroot=%s\n' "$SYSROOT"
    printf 'objdir=%s\n' "$OBJDIR"
} > "$BUILD_KEY_NEXT"

if [[ -f "$BUILD_KEY" ]] && cmp -s "$BUILD_KEY" "$BUILD_KEY_NEXT"; then
    reuse_build=1
    rm -f "$BUILD_KEY_NEXT"
else
    reuse_build=0
    rm -rf "${OBJDIR:?}" "${PREFIX:?}"
    mv "$BUILD_KEY_NEXT" "$BUILD_KEY"
fi

BINUTILS_FACADE="$BUILD_DIR/binutils-tools"
rm -rf "${BINUTILS_FACADE:?}"
mkdir -p "$BINUTILS_FACADE/bin"
cat > "$BINUTILS_FACADE/bin/sysroot-binutil" <<EOF_BINUTIL
#!/usr/bin/env bash
set -euo pipefail
name=\${0##*/}
name=\${name#x86_64-linux-gnu-}
case "\$name" in
    ld|ld.bfd) target="$SYSROOT/usr/bin/x86_64-linux-gnu-ld.bfd" ;;
    as|ar|ranlib|nm|strip|objdump|readelf) target="$SYSROOT/usr/bin/x86_64-linux-gnu-\$name" ;;
    *) echo "unsupported sysroot binutil wrapper name: \$name" >&2; exit 64 ;;
esac
exec "$SYSROOT_LOADER" --library-path "$SYSROOT_BINUTIL_LIBPATH" "\$target" "\$@"
EOF_BINUTIL
chmod +x "$BINUTILS_FACADE/bin/sysroot-binutil"
for tool in as ld ld.bfd ar ranlib nm strip objdump readelf; do
    ln -sfn sysroot-binutil "$BINUTILS_FACADE/bin/$tool"
    ln -sfn sysroot-binutil "$BINUTILS_FACADE/bin/x86_64-linux-gnu-$tool"
done
ln -sfn "$LLD_REAL" "$BINUTILS_FACADE/bin/ld.lld"
ln -sfn "$RUNTIME_ROOT/$LLVM_DIR/bin/llvm-profdata" "$BINUTILS_FACADE/bin/llvm-profdata"

AS_REAL="$BINUTILS_FACADE/bin/as"
LD_REAL="$BINUTILS_FACADE/bin/ld.lld"
AR_REAL="$BINUTILS_FACADE/bin/ar"
RANLIB_REAL="$BINUTILS_FACADE/bin/ranlib"
NM_REAL="$BINUTILS_FACADE/bin/nm"
STRIP_REAL="$BINUTILS_FACADE/bin/strip"
OBJDUMP_REAL="$BINUTILS_FACADE/bin/objdump"
READELF_REAL="$BINUTILS_FACADE/bin/readelf"

for tool in "$AS_REAL" "$LD_REAL" "$AR_REAL" "$RANLIB_REAL" "$NM_REAL" "$STRIP_REAL" "$OBJDUMP_REAL" "$READELF_REAL"; do
    "$tool" --version >/dev/null 2>&1 || {
        echo "pinned sysroot binutil facade cannot run: $tool" >&2
        exit 69
    }
done

BINUTILS_AUDIT="$BUILD_DIR/binutils-audit.txt"
{
    printf 'BINUTILS_VERSION=%s\n' "$BINUTILS_VERSION"
    printf 'BINUTILS_FINGERPRINT=%s\n' "$BINUTILS_FINGERPRINT"
    printf 'SCCACHE_C_CUSTOM_CACHE_BUSTER=%s\n' "$SCCACHE_C_CACHE_BUSTER"
    printf 'CLANG=%s\nLLD=%s\nLLD_VERSION=%s\n' "$CC_REAL" "$LLD_REAL" "$LLD_VERSION"
    printf 'CLANG_SHA256=%s\nLLD_SHA256=%s\n' "$CLANG_SHA256" "$LLD_SHA256"
    printf '%s\n' 'int main(void) { return 0; }' > "$BUILD_DIR/linker-audit.c"
    "$CC_REAL" --sysroot="$SYSROOT" --gcc-toolchain="$SYSROOT/usr" \
        -B"$BINUTILS_FACADE/bin" -fuse-ld=lld -### \
        "$BUILD_DIR/linker-audit.c" -o "$BUILD_DIR/linker-audit" 2> "$BUILD_DIR/linker-driver.txt"
    "$RG_BIN" -Fq "$BINUTILS_FACADE/bin/ld.lld" "$BUILD_DIR/linker-driver.txt" || {
        echo "Clang linker driver did not select the audited LLD" >&2
        exit 69
    }
    rm -f "$BUILD_DIR/linker-audit.c"
    printf 'AR=%s\n' "$AR_REAL"
    printf 'RANLIB=%s\n' "$RANLIB_REAL"
    printf 'NM=%s\n' "$NM_REAL"
    printf 'STRIP=%s\n' "$STRIP_REAL"
    printf 'OBJDUMP=%s\n' "$OBJDUMP_REAL"
    printf 'READELF=%s\n' "$READELF_REAL"
} > "$BINUTILS_AUDIT"

SCCACHE_RUN="$BUILD_DIR/sccache-run"
cat > "$SCCACHE_RUN" <<EOF_SCCACHE_RUN
#!/usr/bin/env bash
set -euo pipefail
if (( \$# < 2 )); then
    echo "usage: \$0 PORT COMPILER [ARGS...]" >&2
    exit 64
fi
port=\$1
compiler=\$2
shift 2
export SCCACHE_START_SERVER=0 SCCACHE_NO_AUTO_START_SERVER=1
if [[ -n "\$port" ]]; then
    export SCCACHE_SERVER_PORT="\$port"
    unset SCCACHE_SERVER_UDS
fi
errfile=\$(mktemp "\${TMPDIR:-/tmp}/museum-sccache.XXXXXX")
trap 'rm -f "\$errfile"' EXIT
attempt=1
while :; do
    : > "\$errfile"
    set +e
    "$SCCACHE_BIN" "\$compiler" "\$@" 2>"\$errfile"
    status=\$?
    set -e
    if (( status != 2 )); then
        cat "\$errfile" >&2
        if (( attempt > 1 )); then
            echo "MUSEUM_SCCACHE_RECOVERED: succeeded on attempt \$attempt" >&2
        fi
        exit "\$status"
    fi
    if (( attempt >= 5 )); then
        cat "\$errfile" >&2
        echo "MUSEUM_SCCACHE_UNRECOVERED: sccache client failed 5 times" >&2
        exit "\$status"
    fi
    sleep 0.25
    ((attempt += 1))
done
EOF_SCCACHE_RUN
chmod +x "$SCCACHE_RUN"

write_compiler_wrapper() {
    local path=$1 compiler=$2
    cat > "$path" <<EOF_WRAPPER
#!/usr/bin/env bash
set -euo pipefail
probe_port=''
configured_probe_port='$PROBE_PORT'
if [[ -n "\$configured_probe_port" ]]; then
    for arg in "\$@"; do
        case "\$arg" in
            *conftest*) probe_port="\$configured_probe_port"; break ;;
        esac
    done
fi
exec "$SCCACHE_RUN" "\$probe_port" "$compiler" --sysroot="$SYSROOT" --gcc-toolchain="$SYSROOT/usr" -B"$BINUTILS_FACADE/bin" -fuse-ld=lld "\$@"
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
jit_port='$JIT_PORT'
exec "$SCCACHE_RUN" "\$jit_port" "$RUNTIME_ROOT/$LLVM_DIR/bin/clang" "\$@"
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

# Keep configure, compiler, and binutils discovery inside the pinned toolchain/sysroot.
# These caller-controlled variables can otherwise inject build-host headers,
# libraries, config answers, or Python modules even when --sysroot is present.
unset CFLAGS CXXFLAGS CPPFLAGS LDFLAGS LIBS
unset CPATH C_INCLUDE_PATH CPLUS_INCLUDE_PATH OBJC_INCLUDE_PATH
unset LIBRARY_PATH COMPILER_PATH GCC_EXEC_PREFIX LD_LIBRARY_PATH CONFIG_SITE
unset PYTHONPATH PYTHONHOME
unset OPENSSL_CFLAGS OPENSSL_LIBS OPENSSL_LDFLAGS OPENSSL_INCLUDES
unset PKG_CONFIG_PATH PKG_CONFIG_SYSROOT_DIR PKG_CONFIG_LIBDIR
unset SCCACHE_SERVER_PORT SCCACHE_SERVER_UDS SCCACHE_DIRECT SCCACHE_C_CUSTOM_CACHE_BUSTER
export PATH="$BINUTILS_FACADE/bin:$PATH"
export SCCACHE_C_CUSTOM_CACHE_BUSTER="$SCCACHE_C_CACHE_BUSTER"

export SCCACHE_START_SERVER=0 SCCACHE_NO_AUTO_START_SERVER=1
export LLVM_VERSION=21
export LLVM_TOOLS_INSTALL_DIR="$LLVM_FACADE"
export CFLAGS_JIT="--sysroot=$SYSROOT"
export PYTHON_FOR_REGEN="$BUILD_DIR/build-python"
export CC="$BUILD_DIR/cc-sccache"
export CXX="$BUILD_DIR/cxx-sccache"
export AS="$AS_REAL" LD="$LD_REAL" AR="$AR_REAL" RANLIB="$RANLIB_REAL"
export NM="$NM_REAL" STRIP="$STRIP_REAL" OBJDUMP="$OBJDUMP_REAL" READELF="$READELF_REAL"
export PKG_CONFIG_SYSROOT_DIR="$SYSROOT"
export PKG_CONFIG_LIBDIR="$SYSROOT/usr/lib/x86_64-linux-gnu/pkgconfig:$SYSROOT/usr/lib/pkgconfig:$SYSROOT/usr/share/pkgconfig"

"$SCCACHE_BIN" --show-stats > "$BUILD_DIR/sccache-before.txt"
if [[ -n "$PROBE_PORT" ]]; then
    SCCACHE_SERVER_PORT="$PROBE_PORT" SCCACHE_START_SERVER=0 SCCACHE_NO_AUTO_START_SERVER=1 \
        "$SCCACHE_BIN" --show-stats > "$BUILD_DIR/sccache-probe-before.txt"
fi

CONFIGURE_LOG="$BUILD_DIR/configure.log"
BUILD_LOG="$BUILD_DIR/build.log"
MAKEFILE_AUDIT="$BUILD_DIR/makefile-audit.txt"
SCCACHE_FAILURE_PATTERN='MUSEUM_SCCACHE_UNRECOVERED:|sccache: error:|compiling locally instead|failed to connect to sccache server|server looks like it shut down unexpectedly'

sccache_log_failed() {
    local log
    for log in "$@"; do
        [[ -f "$log" ]] || continue
        if "$RG_BIN" -qi "$SCCACHE_FAILURE_PATTERN" "$log"; then
            return 0
        fi
    done
    return 1
}

trim_space() {
    local value=$1
    value="${value#"${value%%[![:space:]]*}"}"
    value="${value%"${value##*[![:space:]]}"}"
    printf '%s' "$value"
}

make_var() {
    local key=$1 line
    line="$("$RG_BIN" -N -m1 "^${key}=" Makefile)" || return 1
    trim_space "${line#*=}"
}

require_contains() {
    local label=$1 value=$2 needle=$3
    if [[ "$value" != *"$needle"* ]]; then
        printf 'generated Makefile invariant failed: %s lacks %q\n' "$label" "$needle" >&2
        printf '%s=%s\n' "$label" "$value" >&2
        return 1
    fi
}

audit_makefile() {
    [[ -f Makefile && -f pyconfig.h ]] || {
        echo "configure completed without Makefile/pyconfig.h" >&2
        return 1
    }

    local make_cc make_cxx make_ar pgo_gen pgo_use prof_merger profile_task cflags_nodist ldflags_nodist
    make_cc="$(make_var CC)" || return 1
    make_cxx="$(make_var CXX)" || return 1
    make_ar="$(make_var AR)" || return 1
    # CPython 3.15.2 leaves READELF=@READELF@ in the generated Makefile and
    # never references that variable from a build rule. The pinned readelf
    # facade is executed during binutils preflight, so do not require an
    # impossible Makefile substitution here.
    pgo_gen="$(make_var PGO_PROF_GEN_FLAG)" || return 1
    pgo_use="$(make_var PGO_PROF_USE_FLAG)" || return 1
    prof_merger="$(make_var LLVM_PROF_MERGER)" || return 1
    profile_task="$(make_var PROFILE_TASK)" || return 1
    cflags_nodist="$(make_var CONFIGURE_CFLAGS_NODIST)" || return 1
    ldflags_nodist="$(make_var CONFIGURE_LDFLAGS_NODIST)" || return 1

    [[ "$make_cc" == "$CC" ]] || {
        printf 'generated Makefile CC mismatch: expected %s, got %s\n' "$CC" "$make_cc" >&2
        return 1
    }
    [[ "$make_cxx" == "$CXX" ]] || {
        printf 'generated Makefile CXX mismatch: expected %s, got %s\n' "$CXX" "$make_cxx" >&2
        return 1
    }
    [[ "$make_ar" == "$AR" ]] || {
        printf 'generated Makefile AR mismatch: expected %s, got %s\n' "$AR" "$make_ar" >&2
        return 1
    }
    require_contains PGO_PROF_GEN_FLAG "$pgo_gen" '-fprofile-instr-generate' || return 1
    require_contains PGO_PROF_USE_FLAG "$pgo_use" '-fprofile-instr-use=' || return 1
    require_contains LLVM_PROF_MERGER "$prof_merger" 'llvm-profdata' || return 1
    require_contains LLVM_PROF_MERGER "$prof_merger" 'merge' || return 1
    [[ "$profile_task" == "-m test --pgo --timeout=\$(TESTTIMEOUT)" ]] || {
        printf 'unexpected PROFILE_TASK: %s\n' "$profile_task" >&2
        return 1
    }
    "$RG_BIN" -q '^all:[[:space:]]+profile-opt$' Makefile || {
        echo "generated Makefile default target is not profile-opt" >&2
        return 1
    }
    require_contains CONFIGURE_CFLAGS_NODIST "$cflags_nodist" '-flto' || return 1
    require_contains CONFIGURE_CFLAGS_NODIST "$cflags_nodist" '-D_Py_TIER2=1' || return 1
    require_contains CONFIGURE_CFLAGS_NODIST "$cflags_nodist" '-D_Py_JIT' || return 1
    require_contains CONFIGURE_LDFLAGS_NODIST "$ldflags_nodist" '-flto' || return 1

    "$RG_BIN" -Fq -- "'--with-openssl=$SYSROOT/usr'" Makefile || {
        echo "generated configure arguments lost the sysroot-scoped OpenSSL policy" >&2
        return 1
    }

    "$RG_BIN" -Fq -- "--cflags=\"--sysroot=$SYSROOT\"" Makefile || {
        echo "generated JIT command lost the Debian sysroot" >&2
        return 1
    }
    "$RG_BIN" -Fq -- '--llvm-version="21"' Makefile || {
        echo "generated JIT command is not pinned to LLVM 21" >&2
        return 1
    }
    "$RG_BIN" -Fq -- "--llvm-tools-install-dir=\"$LLVM_FACADE\"" Makefile || {
        echo "generated JIT command does not use the sccache-backed LLVM facade" >&2
        return 1
    }

    local host_root_flags
    host_root_flags="$(
        "$RG_BIN" -n \
            -e '(^|[[:space:]=])-[IL]/(usr|lib|lib64|opt)(/|[[:space:]])' \
            -e '(^|[[:space:]=])-isystem[[:space:]]+/(usr|lib|lib64|opt)(/|[[:space:]])' \
            -e '(^|[[:space:]=])-Wl,-rpath(=|,)/(usr|lib|lib64|opt)(/|[[:space:]])' \
            Makefile || true
    )"
    if [[ -n "$host_root_flags" ]]; then
        echo "generated Makefile contains host-root include/library/rpath flags outside the Debian sysroot" >&2
        printf '%s\n' "$host_root_flags" >&2
        return 1
    fi

    printf 'CC=%s\n' "$make_cc"
    printf 'CXX=%s\n' "$make_cxx"
    printf 'AR=%s\n' "$make_ar"
    printf 'PGO_PROF_GEN_FLAG=%s\n' "$pgo_gen"
    printf 'PGO_PROF_USE_FLAG=%s\n' "$pgo_use"
    printf 'LLVM_PROF_MERGER=%s\n' "$prof_merger"
    printf 'PROFILE_TASK=%s\n' "$profile_task"
    printf 'DEFAULT_TARGET=profile-opt\n'
    printf 'CONFIGURE_CFLAGS_NODIST=%s\n' "$cflags_nodist"
    printf 'CONFIGURE_LDFLAGS_NODIST=%s\n' "$ldflags_nodist"
    printf 'JIT_SYSROOT=%s\n' "$SYSROOT"
    printf 'LLVM_TOOLS_INSTALL_DIR=%s\n' "$LLVM_FACADE"
}

pushd "$OBJDIR" >/dev/null
if (( reuse_build == 0 )) || [[ ! -f Makefile || ! -f pyconfig.h ]]; then
    set +e
    "$RUNTIME_ROOT/$PYTHON_DIR/configure" \
        --prefix="$PREFIX" \
        --enable-optimizations \
        --with-lto=full \
        --enable-experimental-jit=yes \
        --with-ensurepip=no \
        --with-openssl="$SYSROOT/usr" \
        2>&1 | tee "$CONFIGURE_LOG"
    configure_status=${PIPESTATUS[0]}
    set -e
    if (( configure_status != 0 )); then
        popd >/dev/null
        exit "$configure_status"
    fi
else
    echo "reusing matching configure output in $OBJDIR"
fi

# A reused configure result is just as provenance-sensitive as a fresh one.
# Autoconf redirects compiler stderr to config.log, so the human transcript
# alone cannot prove that feature detection was infrastructure-clean.
if sccache_log_failed "$CONFIGURE_LOG" "$OBJDIR/config.log"; then
    popd >/dev/null
    echo "configure observed an sccache infrastructure failure; rejecting feature detection" >&2
    exit 70
fi

audit_makefile | tee "$MAKEFILE_AUDIT"

set +e
{
    make -j"$MAKE_JOBS" && make -j"$MAKE_JOBS" install
} 2>&1 | tee "$BUILD_LOG"
build_status=${PIPESTATUS[0]}
set -e
popd >/dev/null

"$SCCACHE_BIN" --show-stats > "$BUILD_DIR/sccache-after.txt"
if [[ -n "$PROBE_PORT" ]]; then
    SCCACHE_SERVER_PORT="$PROBE_PORT" SCCACHE_START_SERVER=0 SCCACHE_NO_AUTO_START_SERVER=1 \
        "$SCCACHE_BIN" --show-stats > "$BUILD_DIR/sccache-probe-after.txt"
fi

if sccache_log_failed "$BUILD_LOG"; then
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

# Prove the installed prefix can be relocated without duplicating its payload.
# A hard-link mirror gives the runtime a genuinely different pathname while
# PYTHONDONTWRITEBYTECODE prevents the smoke test from mutating linked files.
RELOC_ROOT="$(mktemp -d "$BUILD_DIR/.reloc.XXXXXX")"
cleanup_reloc() {
    rm -rf -- "$RELOC_ROOT"
}
trap cleanup_reloc EXIT
cp -al "$PREFIX/." "$RELOC_ROOT/"
PYTHONDONTWRITEBYTECODE=1 PYTHONHOME="$RELOC_ROOT" PYTHON_JIT=1 \
    "$RELOC_ROOT/bin/python3.15" - "$RELOC_ROOT" <<'PY' | tee "$BUILD_DIR/relocation-smoke.txt"
import json
import math
import pathlib
import sys

expected = pathlib.Path(sys.argv[1]).resolve()
assert pathlib.Path(sys.prefix).resolve() == expected, (sys.prefix, expected)
assert pathlib.Path(sys.exec_prefix).resolve() == expected, (sys.exec_prefix, expected)
assert pathlib.Path(json.__file__).resolve().is_relative_to(expected)
assert pathlib.Path(math.__file__).resolve().is_relative_to(expected)
assert sys._jit.is_available(), "relocated runtime lost JIT availability"
assert sys._jit.is_enabled(), "PYTHON_JIT=1 did not enable relocated runtime JIT"
print("relocated_prefix", expected)
print("stdlib", pathlib.Path(json.__file__).resolve())
print("extension", pathlib.Path(math.__file__).resolve())
print("jit_available", sys._jit.is_available())
print("jit_enabled", sys._jit.is_enabled())
PY
cleanup_reloc
trap - EXIT

GLIBC_VERSIONS="$BUILD_DIR/glibc-versions.txt"
: > "$GLIBC_VERSIONS"
while IFS= read -r -d '' artifact; do
    if "$FILE_BIN" -b "$artifact" | "$RG_BIN" -q '^ELF '; then
        "$OBJDUMP_REAL" -T "$artifact" 2>/dev/null | "$RG_BIN" -o 'GLIBC_[0-9]+(\.[0-9]+)+' || true
    fi
done < <(find "$PREFIX" -type f -print0) | sort -Vu > "$GLIBC_VERSIONS"

max_glibc="$(sed -n 's/^GLIBC_//p' "$GLIBC_VERSIONS" | sort -V | tail -n 1)"
if [[ -z "$max_glibc" ]]; then
    echo "ELF ABI scan found no GLIBC symbol versions in the installed runtime" >&2
    exit 70
fi
printf 'max_glibc=%s\n' "$max_glibc" | tee "$BUILD_DIR/abi-summary.txt"
if [[ -n "${MUSEUM_CPYTHON_MAX_GLIBC:-}" ]]; then
    highest="$(printf '%s\n%s\n' "$MUSEUM_CPYTHON_MAX_GLIBC" "$max_glibc" | sort -V | tail -n 1)"
    if [[ "$highest" != "$MUSEUM_CPYTHON_MAX_GLIBC" ]]; then
        echo "artifact requires GLIBC_$max_glibc, above requested GLIBC_$MUSEUM_CPYTHON_MAX_GLIBC" >&2
        exit 70
    fi
fi

{
    printf 'builder_sha256=%s\n' "$BUILDER_SHA256"
    printf 'python_version=3.15.2\n'
    printf 'python_sha256=%s\n' "$PYTHON_SHA256"
    printf 'abstract_upstream_sha256=%s\n' "$ABSTRACT_UPSTREAM_SHA256"
    printf 'clang_sha256=%s\n' "$CLANG_SHA256"
    printf 'lld_sha256=%s\n' "$LLD_SHA256"
    printf 'lld_version=%s\n' "$LLD_VERSION"
    printf 'llvm_version=21.1.8\n'
    printf 'llvm_sha256=%s\n' "$LLVM_SHA256"
    printf 'binutils_version=%s\n' "$BINUTILS_VERSION"
    printf 'binutils_facade=%s\n' "$BINUTILS_FACADE"
    printf 'binutils_fingerprint=%s\n' "$BINUTILS_FINGERPRINT"
    printf 'sccache_c_custom_cache_buster=%s\n' "$SCCACHE_C_CACHE_BUSTER"
    printf 'sysroot=%s\n' "$SYSROOT"
    printf 'cc=%s\n' "$CC_REAL"
    "$CC_REAL" --version | head -n 1 | sed 's/^/cc_version=/'
    "$RUNTIME_ROOT/$LLVM_DIR/bin/clang" --version | head -n 1 | sed 's/^/clang_version=/'
    printf 'make_jobs=%s\n' "$MAKE_JOBS"
    printf 'objdir=%s\n' "$OBJDIR"
    printf 'sccache_probe_port=%s\n' "${PROBE_PORT:-default}"
    printf 'sccache_jit_port=%s\n' "${JIT_PORT:-default}"
    printf 'max_glibc=%s\n' "$max_glibc"
} > "$BUILD_DIR/build-manifest.txt"

printf 'installed CPython JIT: %s\n' "$PREFIX"

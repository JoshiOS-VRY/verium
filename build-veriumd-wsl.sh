#!/usr/bin/env bash
# Build veriumd.exe for the Vericonomy wallet (MinGW cross-compile on WSL).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
JOBS="${JOBS:-$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)}"
BUILD_DIR="${VERIUMD_BUILD_DIR:-$HOME/vericonomy-build-verium}"
CROSS_HOST="x86_64-w64-mingw32"
EXE_SUFFIX=".exe"
LOG_FILE="${VERIUMD_BUILD_LOG:-$BUILD_DIR/build-veriumd.log}"

need() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "Missing required tool: $1" >&2
    exit 1
  }
}

log() {
  mkdir -p "$(dirname "$LOG_FILE")"
  echo "$@" | tee -a "$LOG_FILE"
}

need autoreconf
need make
need rsync
need "${CROSS_HOST}-g++"
need "${CROSS_HOST}-windres"

setup_mingw_path() {
  local wrap="${BUILD_DIR}/.mingw-wrap"
  mkdir -p "$wrap"
  cat > "${wrap}/windres" <<EOF
#!/bin/sh
exec ${CROSS_HOST}-windres \\
  -I/usr/x86_64-w64-mingw32/include \\
  -I/usr/share/mingw-w64/include \\
  "\$@"
EOF
  chmod +x "${wrap}/windres"
  export PATH="${wrap}:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
}

normalize_autotools() {
  local dir="$1"
  log "==> Normalizing CRLF under ${dir}"
  find "$dir" -type f \( \
    -name 'Makefile' -o -name '*.mk' -o -name '*.am' -o -name '*.ac' -o \
    -name '*.in' -o -name '*.m4' -o -name '*.sh' -o -name 'config.guess' -o \
    -name 'config.sub' -o -name '*.h' -o -name '*.cpp' -o -name '*.c' \
  \) -not -path '*/.git/*' -exec sed -i 's/\r$//' {} + 2>/dev/null || true
  chmod +x "$dir/depends/config.guess" "$dir/depends/config.sub" 2>/dev/null || true
  chmod +x "$dir/autogen.sh" "$dir/build-veriumd-wsl.sh" 2>/dev/null || true
}

sync_sources() {
  log "==> Syncing sources to ${BUILD_DIR}"
  mkdir -p "$(dirname "$BUILD_DIR")"
  rsync -a --delete \
    --exclude '.git' \
    --exclude 'depends/work' \
    --exclude 'depends/built' \
    --exclude 'autom4te.cache' \
    --exclude 'desktop' \
    "$ROOT/" "$BUILD_DIR/"
  normalize_autotools "$BUILD_DIR"
}

prepare_autotools() {
  cd "$BUILD_DIR"
  rm -rf autom4te.cache src/univalue/autom4te.cache 2>/dev/null || true
  rm -f configure Makefile.in src/univalue/configure src/univalue/Makefile.in
  chmod +x autogen.sh build-veriumd-wsl.sh 2>/dev/null || true
  ./autogen.sh
}

build_depends() {
  cd "$BUILD_DIR/depends"
  log "==> Building depends for ${CROSS_HOST}"
  setup_mingw_path
  make HOST="${CROSS_HOST}" NO_QT=1 NO_QR=1 -j"${JOBS}"
}

clean_tree() {
  cd "$BUILD_DIR"
  if [[ -f Makefile ]]; then
    log "==> make distclean"
    make distclean >/dev/null 2>&1 || make distclean || true
  fi
}

configure_daemon() {
  setup_mingw_path
  log "==> configure veriumd (${CROSS_HOST})"
  CONFIG_SITE="$BUILD_DIR/depends/${CROSS_HOST}/share/config.site" \
    ./configure --prefix=/ \
      --host="${CROSS_HOST}" \
      --without-gui --disable-tests --disable-bench \
      --with-incompatible-bdb
}

make_veriumd() {
  local target="veriumd${EXE_SUFFIX}"
  log "==> make -C src ${target} (-j${JOBS})"
  make -j"${JOBS}" -C src "${target}"
  if [[ ! -f "src/${target}" ]]; then
    echo "ERROR: expected src/${target} after build" >&2
    exit 1
  fi
  cp -f "src/${target}" "$ROOT/src/${target}"
  log "==> copied to $ROOT/src/${target}"
}

preflight() {
  mkdir -p "$BUILD_DIR"
  : >"$LOG_FILE"
  log "==> veriumd sidecar build"
  log "    source:  ${ROOT}"
  log "    build:   ${BUILD_DIR}"
  log "    host:    ${CROSS_HOST}"
  log "    jobs:    ${JOBS}"
  log "    log:     ${LOG_FILE}"
}

if [[ "$ROOT" == /mnt/* ]]; then
  preflight
  sync_sources
  if [[ ! -f "$BUILD_DIR/depends/${CROSS_HOST}/share/config.site" ]]; then
    prepare_autotools
    build_depends
  else
    cd "$BUILD_DIR"
    prepare_autotools
  fi
  cd "$BUILD_DIR"
  clean_tree
  configure_daemon
  make_veriumd
else
  preflight
  BUILD_DIR="$ROOT"
  EXE_SUFFIX=""
  cd "$BUILD_DIR"
  [[ -f ./configure ]] || ./autogen.sh
  clean_tree
  ./configure --without-gui --disable-tests --disable-bench --with-incompatible-bdb
  make_veriumd
fi

log ""
log "veriumd ready: $ROOT/src/veriumd${EXE_SUFFIX}"

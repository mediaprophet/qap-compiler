#!/usr/bin/env bash
# G-DEBIAN-GLIBC & G-IOT-HEADLESS-DEPS CI gate.
# PLAN-WEBIZEN-DEBIAN-NATIVE-PROFILES-2026-11-26 WP-3
#
# Inspects an ELF binary and fails closed when:
#   * any GLIBC_2.37 / GLIBC_2.38 / GLIBC_2.39 symbol is required
#   * libwebkit2gtk or libjavascriptcoregtk appears in NEEDED
#   * --profile webizen-iot-headless and libX11 / libwayland / libgtk / libvulkan appear in NEEDED
#
# Copyright (c) 2026 Timothy Charles Holborn
# License: Attribution-NonCommercial-NoDerivatives 4.0 International (CC BY-NC-ND 4.0)

set -euo pipefail

PROFILE=""
BIN=""

usage() {
  echo "Usage: $0 [--profile webizen-iot-headless] --bin <elf> | $0 [--profile ...] <elf>" >&2
  exit 1
}

while [ $# -gt 0 ]; do
  case "$1" in
    --profile) PROFILE="${2:-}"; shift 2 ;;
    --bin) BIN="${2:-}"; shift 2 ;;
    -h|--help) usage ;;
    *)
      if [ -z "$BIN" ]; then
        BIN="$1"
        shift
      else
        echo "Unknown argument: $1" >&2
        usage
      fi
      ;;
  esac
done

if [ -z "$BIN" ] || [ ! -f "$BIN" ]; then
  usage
fi

if ! command -v readelf >/dev/null 2>&1 && ! command -v objdump >/dev/null 2>&1; then
  echo "G-DEBIAN-GLIBC: readelf or objdump is required to audit $BIN" >&2
  exit 1
fi

dump_versions() {
  if command -v objdump >/dev/null 2>&1; then
    objdump -T "$BIN" 2>/dev/null || true
  fi
  if command -v readelf >/dev/null 2>&1; then
    readelf --version-info "$BIN" 2>/dev/null || true
    readelf -V "$BIN" 2>/dev/null || true
  fi
}

dump_needed() {
  if command -v readelf >/dev/null 2>&1; then
    readelf -d "$BIN" 2>/dev/null || true
  else
    objdump -p "$BIN" 2>/dev/null || true
  fi
}

VERSIONS="$(dump_versions)"
NEEDED="$(dump_needed)"

if echo "$VERSIONS" | grep -Eq 'GLIBC_2\.(37|38|39)'; then
  echo "G-DEBIAN-GLIBC: $BIN requires GLIBC_2.37+ and will not start on Debian 12 bookworm (glibc 2.36)" >&2
  echo "$VERSIONS" | grep -E 'GLIBC_2\.(37|38|39)' >&2 || true
  exit 1
fi

if echo "$NEEDED" | grep -Eqi 'libwebkit2gtk|libjavascriptcoregtk'; then
  echo "G-DEBIAN-GLIBC: $BIN links WebKit2GTK or JavaScriptCore; native Debian packages must not" >&2
  echo "$NEEDED" | grep -Ei 'libwebkit2gtk|libjavascriptcoregtk' >&2 || true
  exit 1
fi

if [ "$PROFILE" = "webizen-iot-headless" ]; then
  if echo "$NEEDED" | grep -Eqi 'libX11|libwayland|libgtk|libvulkan'; then
    echo "G-IOT-HEADLESS-DEPS: $BIN must not link libX11, libwayland, libgtk, or libvulkan" >&2
    echo "$NEEDED" | grep -Ei 'libX11|libwayland|libgtk|libvulkan' >&2 || true
    exit 1
  fi
fi

echo "G-DEBIAN-GLIBC: $BIN is compatible with Debian 12 bookworm (glibc <= 2.36)"
if [ -n "$PROFILE" ]; then
  echo "G-IOT-HEADLESS-DEPS: profile '$PROFILE' shared-library gate passed"
fi

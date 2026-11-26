#!/usr/bin/env bash
# Webizen / QualiaDB Debian native package builder (dpkg-deb).
# PLAN-WEBIZEN-DEBIAN-NATIVE-PROFILES-2026-11-26 WP-3
#
# Copyright (c) 2026 Timothy Charles Holborn
# License: Attribution-NonCommercial-NoDerivatives 4.0 International (CC BY-NC-ND 4.0)
#
# Emits profile-specific Depends lines for Debian 12 bookworm (glibc 2.36).
# Never includes libwebkit2gtk-4.1-0 or libjavascriptcoregtk-4.1-0.

set -euo pipefail

PROFILE="webizen-host"
ARCH="amd64"
PKG_NAME=""
VERSION="0.0.41"
BIN=""
OUT_DIR="./dist"

usage() {
  echo "Usage: $0 --profile <webizen-host|webizen-desktop|webizen-subsystem|webizen-mobile|webizen-iot-headless|webizen-iot-display> --arch <amd64|arm64|armhf> --pkg-name <name> --version <ver> --bin <path> --out-dir <dir>" >&2
  exit 1
}

while [ $# -gt 0 ]; do
  case "$1" in
    --profile) PROFILE="${2:-}"; shift 2 ;;
    --arch) ARCH="${2:-}"; shift 2 ;;
    --pkg-name) PKG_NAME="${2:-}"; shift 2 ;;
    --version) VERSION="${2:-}"; shift 2 ;;
    --bin) BIN="${2:-}"; shift 2 ;;
    --out-dir) OUT_DIR="${2:-}"; shift 2 ;;
    -h|--help) usage ;;
    *) echo "Unknown argument: $1" >&2; usage ;;
  esac
done

if [ -z "$PKG_NAME" ] || [ -z "$BIN" ]; then
  usage
fi

case "$ARCH" in
  amd64|arm64|armhf) ;;
  *) echo "Unsupported Debian arch: $ARCH" >&2; exit 1 ;;
esac

depends_for_profile() {
  case "$1" in
    webizen-host)
      echo "libc6 (>= 2.36), libssl3, ca-certificates, nftables"
      ;;
    webizen-desktop)
      echo "libc6 (>= 2.36), libvulkan1, libwayland-client0, libx11-6, libxkbcommon0, libfontconfig1, libasound2"
      ;;
    webizen-subsystem)
      echo "libc6 (>= 2.36), mesa-vulkan-drivers, libvulkan1, libwayland-client0, libxkbcommon0, libfontconfig1, libpulse0, libasound2-plugins"
      ;;
    webizen-mobile)
      echo "libc6 (>= 2.36), libvulkan1, libgles2, libwayland-client0, libxkbcommon0, libfontconfig1, libasound2"
      ;;
    webizen-iot-headless)
      echo "libc6 (>= 2.36), ca-certificates"
      ;;
    webizen-iot-display)
      echo "libc6 (>= 2.36), libvulkan1 | libgles2, libwayland-client0 | libdrm2, libxkbcommon0, libfontconfig1, libasound2"
      ;;
    *)
      echo "Unknown profile: $1" >&2
      exit 1
      ;;
  esac
}

DEPENDS="$(depends_for_profile "$PROFILE")"

if echo "$DEPENDS $PROFILE $PKG_NAME" | grep -Eqi 'webkit2gtk|javascriptcoregtk'; then
  echo "G-DEBIAN-GLIBC: native Debian packages must not depend on WebKit2GTK or JavaScriptCore" >&2
  exit 1
fi

if [ ! -f "$BIN" ]; then
  echo "Binary not found: $BIN" >&2
  exit 1
fi

mkdir -p "$OUT_DIR"
SAFE_VER="$(printf '%s' "$VERSION" | tr -c 'A-Za-z0-9.+~-' '-')"
PKG_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/${PKG_NAME}_${SAFE_VER}_${ARCH}.XXXXXX")"
cleanup() { rm -rf "$PKG_ROOT"; }
trap cleanup EXIT

install -d -m 0755 "$PKG_ROOT/DEBIAN" "$PKG_ROOT/usr/local/bin"
install -m 0755 "$BIN" "$PKG_ROOT/usr/local/bin/$(basename "$BIN")"

{
  echo "Package: $PKG_NAME"
  echo "Version: $VERSION"
  echo "Section: net"
  echo "Priority: optional"
  echo "Architecture: $ARCH"
  echo "Depends: $DEPENDS"
  if [ "$PROFILE" = "webizen-iot-headless" ]; then
    echo "Recommends: libudev1, libdbus-1-3"
  fi
  echo "Maintainer: Timothy Charles Holborn <timothy.holborn@gmail.com>"
  echo "Homepage: https://www.linkedin.com/in/ubiquitous/"
  echo "Description: Webizen Human-Centric ${PROFILE} Debian native package (bookworm glibc 2.36)"
  echo " Human rights, human dignity, and human agency runtime for Personal & Family Stewardship."
} > "$PKG_ROOT/DEBIAN/control"

DEB_OUT="${OUT_DIR}/${PKG_NAME}_${VERSION}_${ARCH}.deb"
dpkg-deb --build "$PKG_ROOT" "$DEB_OUT"
echo "Wrote $DEB_OUT"

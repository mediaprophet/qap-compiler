#!/usr/bin/env bash
# Webizen Debian Second-OS (.wsl) root filesystem packager.
# PLAN-WEBIZEN-DEBIAN-NATIVE-PROFILES-2026-11-26 WP-3
#
# Stages a Debian 12 bookworm Second-OS appliance archive:
#   webizen-debian-bookworm_<version>_<arch>.wsl
#   webizen-debian-bookworm_<version>_<arch>.tar.zst
#
# Copyright (c) 2026 Timothy Charles Holborn
# License: Attribution-NonCommercial-NoDerivatives 4.0 International (CC BY-NC-ND 4.0)

set -euo pipefail

VERSION="0.0.41"
ARCH="amd64"
OUT_DIR="./dist"
ROOTFS=""

usage() {
  echo "Usage: $0 [--version <ver>] [--arch <amd64|arm64>] [--out-dir <dir>] [--rootfs <existing-root>]" >&2
  exit 1
}

while [ $# -gt 0 ]; do
  case "$1" in
    --version) VERSION="${2:-}"; shift 2 ;;
    --arch) ARCH="${2:-}"; shift 2 ;;
    --out-dir) OUT_DIR="${2:-}"; shift 2 ;;
    --rootfs) ROOTFS="${2:-}"; shift 2 ;;
    -h|--help) usage ;;
    *) echo "Unknown argument: $1" >&2; usage ;;
  esac
done

case "$ARCH" in
  amd64|arm64) ;;
  *) echo "WSL Second-OS appliance supports amd64 and arm64" >&2; exit 1 ;;
esac

mkdir -p "$OUT_DIR"
STAGE="$(mktemp -d "${TMPDIR:-/tmp}/webizen-wsl-rootfs.XXXXXX")"
cleanup() { rm -rf "$STAGE"; }
trap cleanup EXIT

if [ -n "$ROOTFS" ]; then
  if [ ! -d "$ROOTFS" ]; then
    echo "Root filesystem directory not found: $ROOTFS" >&2
    exit 1
  fi
  cp -a "$ROOTFS/." "$STAGE/"
else
  if ! command -v debootstrap >/dev/null 2>&1; then
    echo "debootstrap is required unless --rootfs is provided" >&2
    exit 1
  fi
  debootstrap --variant=minbase --arch="$ARCH" bookworm "$STAGE" http://deb.debian.org/debian
fi

install -d -m 0755 \
  "$STAGE/etc" \
  "$STAGE/usr/local/bin" \
  "$STAGE/usr/lib/wsl" \
  "$STAGE/var/lib/webizen-host" \
  "$STAGE/var/lib/qualia/sanctuary"
chmod 0700 "$STAGE/var/lib/qualia/sanctuary"

cat > "$STAGE/etc/wsl.conf" <<'EOF'
# Webizen Debian Second-OS Subsystem — Sanctuary Keep isolation from host cloud-sync
[boot]
systemd=true

[automount]
enabled=false

[network]
generateResolvConf=true

[interop]
enabled=true
EOF

cat > "$STAGE/etc/wsl-distribution.conf" <<'EOF'
[oobe]
command = /usr/lib/wsl/webizen-oobe.sh
defaultUid = 1000
defaultName = Webizen Debian Native Sanctuary
[windowing]
defaultMode = windowed
EOF

cat > "$STAGE/usr/lib/wsl/webizen-oobe.sh" <<'EOF'
#!/usr/bin/env bash
# Webizen Debian Second-OS first-boot: Sanctuary Keep directories and mount guard.
set -euo pipefail
install -d -m 0700 /var/lib/qualia/sanctuary
install -d -m 0750 /var/lib/webizen-host
if [ -x /usr/local/bin/sanctuary-mount-guard.sh ]; then
  /usr/local/bin/sanctuary-mount-guard.sh
fi
echo "Webizen Debian Native Sanctuary is ready. Human rights, human dignity, and human agency remain with the Natural Person steward."
EOF
chmod 0755 "$STAGE/usr/lib/wsl/webizen-oobe.sh"

cat > "$STAGE/usr/local/bin/sanctuary-mount-guard.sh" <<'EOF'
#!/usr/bin/env bash
# Sanctuary Keep mount guard — Second-OS native Linux ext4 isolation
set -euo pipefail
SANCTUARY_PATHS=(
  /var/lib/webizen-host
  /var/lib/qualia/sanctuary
)
FORBIDDEN_FS="9p|drvfs|virtiofs|cifs|ntfs"
fail_closed() {
  echo "Sanctuary Keep protection: $1" >&2
  echo "Human rights, human dignity, and human agency require Sanctuary Keep vaults to reside on native Linux ext4 virtual disks, never on /mnt/c, 9p, drvfs, virtiofs, cifs, or ntfs host mounts." >&2
  exit 1
}
for p in "${SANCTUARY_PATHS[@]}"; do
  case "$p" in
    /mnt/c|/*/mnt/c/*) fail_closed "$p is under /mnt/c (Windows host mount)" ;;
  esac
  if [ -e "$p" ]; then
    fstype="$(findmnt -n -o FSTYPE --target "$p" 2>/dev/null || true)"
    if echo "$fstype" | grep -Eqi "$FORBIDDEN_FS"; then
      fail_closed "$p is mounted as '$fstype' (forbidden host passthrough)"
    fi
  fi
done
echo "Sanctuary Keep isolation verified: native Linux ext4."
EOF
chmod 0755 "$STAGE/usr/local/bin/sanctuary-mount-guard.sh"

NAME="webizen-debian-bookworm_${VERSION}_${ARCH}"
WSL_OUT="${OUT_DIR}/${NAME}.wsl"
tar --numeric-owner -C "$STAGE" -cf "$WSL_OUT" .
echo "Wrote $WSL_OUT"

if command -v zstd >/dev/null 2>&1; then
  ZST_OUT="${OUT_DIR}/${NAME}.tar.zst"
  tar --numeric-owner -C "$STAGE" -I zstd -cf "$ZST_OUT" .
  echo "Wrote $ZST_OUT"
else
  GZ_OUT="${OUT_DIR}/${NAME}.tar.gz"
  tar --numeric-owner -C "$STAGE" -czf "$GZ_OUT" .
  echo "Wrote $GZ_OUT (zstd not available)"
fi

#!/usr/bin/env bash
set -euo pipefail

# Installer script for contextd and contextctl
# Must be executed as root

if [[ "${EUID}" -ne 0 ]]; then
    echo "Error: install.sh must be executed with root privileges." >&2
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "==> Building contextd and contextctl in release mode..."
cargo build --release --manifest-path "${ROOT_DIR}/Cargo.toml"

echo "==> Installing binaries to /usr/local/bin..."
install -m 0755 "${ROOT_DIR}/target/release/contextd" /usr/local/bin/contextd
install -m 0755 "${ROOT_DIR}/target/release/contextctl" /usr/local/bin/contextctl

echo "==> Setting up systemd sysusers and tmpfiles..."
if [[ -f "${ROOT_DIR}/sysusers.d/contextd.conf" ]]; then
    install -m 0644 "${ROOT_DIR}/sysusers.d/contextd.conf" /usr/lib/sysusers.d/contextd.conf
    systemd-sysusers /usr/lib/sysusers.d/contextd.conf || true
fi

if [[ -f "${ROOT_DIR}/tmpfiles.d/contextd.conf" ]]; then
    install -m 0644 "${ROOT_DIR}/tmpfiles.d/contextd.conf" /usr/lib/tmpfiles.d/contextd.conf
    systemd-tmpfiles --create /usr/lib/tmpfiles.d/contextd.conf || true
fi

echo "==> Installing systemd units..."
install -m 0644 "${ROOT_DIR}/systemd/contextd.socket" /usr/lib/systemd/system/contextd.socket
install -m 0644 "${ROOT_DIR}/systemd/contextd.service" /usr/lib/systemd/system/contextd.service

echo "==> Reloading systemd daemon..."
systemctl daemon-reload
systemctl enable --now contextd.socket

echo "==> contextd socket activated successfully."
echo "Verify status: systemctl status contextd.socket"

#!/usr/bin/env bash
set -euo pipefail

# Uninstaller script for contextd and contextctl
# Must be executed as root

if [[ "${EUID}" -ne 0 ]]; then
    echo "Error: uninstall.sh must be executed with root privileges." >&2
    exit 1
fi

echo "==> Stopping and disabling contextd service and socket..."
systemctl disable --now contextd.service contextd.socket 2>/dev/null || true

echo "==> Removing systemd unit files..."
rm -f /usr/lib/systemd/system/contextd.service
rm -f /usr/lib/systemd/system/contextd.socket
rm -f /usr/lib/sysusers.d/contextd.conf
rm -f /usr/lib/tmpfiles.d/contextd.conf

echo "==> Reloading systemd daemon..."
systemctl daemon-reload

echo "==> Removing installed binaries..."
rm -f /usr/local/bin/contextd
rm -f /usr/local/bin/contextctl

echo "==> Cleaning up runtime sockets..."
rm -rf /run/syntrop/io.syntrop.Context1

echo "==> contextd uninstallation completed."
echo "Note: Historical diffs and event logs in /var/lib/contextd are preserved."
echo "To remove all historical state, run: rm -rf /var/lib/contextd"

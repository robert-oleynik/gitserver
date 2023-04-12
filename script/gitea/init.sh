#!/usr/bin/env bash

echo "running as user: $UID"
echo "===== Prepare /gitea ====="
set -xeo pipefail
mkdir -p "/gitea/custom/conf"
chown -R 1000:1000 "/gitea"
echo "DONE"

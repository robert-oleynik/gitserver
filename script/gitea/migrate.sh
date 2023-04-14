#!/usr/bin/env bash

echo "running as user: $UID"
echo "===== Run Migrations ====="
set -xeo pipefail
environment-to-ini
cat "$GITEA_APP_INI"
gitea migrate

# Configure admin user
if ! gitea admin user list --admin | grep "$ROOT_USER"; then
	# Install
	gitea admin user create --admin \
		--username "$ROOT_USER" \
		--password "$ROOT_PASSWD" \
		--email "$ROOT_EMAIL"
	SECRET_KEY=$(gitea generate secret SECRET_KEY)
	INTERNAL_TOKEN=$(gitea generate secret INTERNAL_TOKEN)
	cat "$GITEA_APP_INI" "-" > "/tmp/app.ini" << EOF
[security]
INSTALL_LOCK = true
SECRET_KEY = $SECRET_KEY
INTERNAL_TOKEN = $INTERNAL_TOKEN
EOF
	mv -f "/tmp/app.ini" "$GITEA_APP_INI"
else
	gitea admin user change-password \
		--username "$ROOT_USER" \
		--password "$ROOT_PASSWD"
fi

echo "DONE"

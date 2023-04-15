#!/usr/bin/env bash

echo "running as user: $UID"
echo "===== Install Jenkins Plugins ====="
set -xeo pipefail

ls -l /var

echo "$JENKINS_VERSION" > "$JENKINS_HOME/jenkins.install.UpgradeWizard.state"
echo "$JENKINS_VERSION" > "$JENKINS_HOME/jenkins.install.InstallUtil.lastExecVersion"

tee "$JENKINS_HOME/plugins.txt" << EOF
configuration-as-code
gitea
blueocean
EOF

jenkins-plugin-cli --verbose -f "$JENKINS_HOME/plugins.txt"
mkdir -p "$JENKINS_HOME/plugins"
cp /usr/share/jenkins/ref/plugins/* "$JENKINS_HOME/plugins"

echo "DONE"

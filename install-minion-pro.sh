#!/bin/bash

# find Cellar location for groff

echo "checking for groff..."
GROFF_VERSION_HOMEBREW=$(brew list --versions | grep groff)

if [[ -z "$GROFF_VERSION_HOMEBREW" ]]; then
  echo "groff not installed via homebrew" 1>&2
  exit 1
fi
echo "found: $GROFF_VERSION_HOMEBREW"

echo "checking for groff homebrew Cellar..."
GROFF_CELLAR_HOMEBREW=$(brew ls groff | grep "bin/groff" | sed 's;/bin/groff;;')
if [[ -z "$GROFF_CELLAR_HOMEBREW" ]]; then
  echo "could not locate groff cellar!"
  exit 1
fi
echo "found: $GROFF_CELLAR_HOMEBREW"

exit 0

# download minion pro
MINION_PRO_URL="https://font.download/dl/font/minion-pro.zip"

wget $MINION_PRO_URL

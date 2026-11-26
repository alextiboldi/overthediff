#!/bin/sh
# .deb post-install and post-remove; package.sh fills in @MAJOR@. After an install, `otd` points at
# the newest installed major version's otd-<major> (update-alternatives, priority = major). Both
# then refresh the launcher's desktop-entry and icon caches; each cache tool is optional.
set -e
if [ "$1" = configure ]; then
  update-alternatives --install /usr/bin/otd otd /usr/bin/otd-@MAJOR@ @MAJOR@
fi
if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database -q /usr/share/applications || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor || true
fi
exit 0

#!/bin/sh
# .deb post-install and post-remove: refresh the launcher's desktop-entry and icon caches. Each
# tool is optional; a missing one is skipped.
if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database -q /usr/share/applications || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor || true
fi
exit 0

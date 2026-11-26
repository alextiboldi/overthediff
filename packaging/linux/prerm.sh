#!/bin/sh
# .deb pre-remove; package.sh fills in @MAJOR@. Removing this major version drops it from the `otd`
# alternative, which then points at the newest major version still installed, or goes away. An
# upgrade within the major version keeps it.
set -e
if [ "$1" = remove ] || [ "$1" = deconfigure ]; then
  update-alternatives --remove otd /usr/bin/otd-@MAJOR@
fi
exit 0

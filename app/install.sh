#!/bin/sh
# Just an quick script for development to install the release version via the RPM file on Fedora or the DEB file on Ubuntu
echo Prompting for sudo just to be able to install at end of bundle
set -x
sudo -v

pnpm tauri build && pnpm tauri bundle
if test -f /usr/bin/rpm 2>/dev/null; then
    sudo rpm -i --reinstall src-tauri/target/release/bundle/rpm/dme-*.x86_64.rpm
else
    sudo apt reinstall src-tauri/target/release/bundle/rpm/dme-*.x86_64.deb
fi

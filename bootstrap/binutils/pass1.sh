#!/bin/sh

# needed to setup gentoo env
. /etc/profile
env-update

set -e

# this folder is used for indirection, specified as sysroot,
# and linked together there for derived stuff for now
mkdir -p /lfs/tools /build /tmp /out
cd /build

date
set +e
if "$1/configure" --prefix=/lfs/tools --with-sysroot=/lfs/tools --target=x86_64-lfs-linux-gnu --disable-nls --disable-werror; then
  set -e
else
  ret=$!
  cat config.log
  exit $ret
fi
make -j2
make install -j1 DESTDIR=/out
date

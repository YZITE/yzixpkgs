#!/bin/bash

BINUTILS="$1"
PATCH1="$2"
set -e

# this folder is used for indirection, specified as sysroot,
# and linked together there for derived stuff for now
mkdir -p /lfs/tools /build
cd /build

date
"$1/configure" --prefix=/lfs/tools --with-sysroot=/lfs/tools --target=x86_64-lfs-linux-gnu --disable-nls --disable-werror
make -j2
make install -j1
date

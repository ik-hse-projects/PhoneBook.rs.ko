#!/bin/sh
set -e
source ./vars.sh

make
make modules_install

pushd $INITRAMFS_BUILD
chmod +x init
find . -print0 | cpio --null -ov --format=newc > $BUILDS/initramfs.cpio
popd


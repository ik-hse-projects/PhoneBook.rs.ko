#!/bin/sh
source ./vars.sh
/usr/bin/alacritty -e /usr/bin/qemu-system-x86_64 -kernel $LINUX_BUILD/arch/x86_64/boot/bzImage \
  -initrd $BUILDS/initramfs.cpio -nographic \
  -append "console=ttyS0 nokaslr" \
  -s -m 256M $@

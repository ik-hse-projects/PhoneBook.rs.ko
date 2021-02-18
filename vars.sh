# build.sh собирает всё вместе в $BUILDS/initramfs.cpio
export BUILDS=/tmp

# Путь к ядру (должно быть собрано)
export LINUX_BUILD=$BUILDS/linux

# Путь к initramfs, нужно для `make modules_install` и `build.sh`
export INITRAMFS_BUILD=$BUILDS/initramfs

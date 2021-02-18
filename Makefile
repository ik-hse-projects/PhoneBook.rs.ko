export KDIR ?= $(LINUX_BUILD)

CLANG ?= clang
ifeq ($(origin CC),default)
CC := ${CLANG}
endif

all:
	$(MAKE) -C $(KDIR) M=$(CURDIR) CC=$(CC) CONFIG_CC_IS_CLANG=y

clean:
	$(MAKE) -C $(KDIR) M=$(CURDIR) CC=$(CC) clean

modules_install:
	make -C $(KDIR) M=$(CURDIR) modules_install INSTALL_MOD_PATH=$(BUILDS)/initramfs

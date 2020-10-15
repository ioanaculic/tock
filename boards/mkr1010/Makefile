# Makefile for building the tock kernel for the Arduino Nano 33 BLE board.

TOCK_ARCH=cortex-m0
TARGET=thumbv7em-none-eabi
PLATFORM=mkr1010

include ../Makefile.common

ifdef PORT
  FLAGS += --port $(PORT)
endif

# Upload the kernel using bossac
.PHONY: program
program: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
	bossac $(FLAGS) -U -i -e -w $< -R

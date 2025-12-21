# Emberware ZX C Game Makefile
#
# Prerequisites:
#   - wasi-sdk: https://github.com/WebAssembly/wasi-sdk/releases
#
# Usage:
#   make          - Build game.wasm
#   make clean    - Remove build artifacts
#   make run      - Build and run in Emberware player

# Path to wasi-sdk installation
# Override with: make WASI_SDK_PATH=/your/path
WASI_SDK_PATH ?= /opt/wasi-sdk

# Compiler settings
CC = $(WASI_SDK_PATH)/bin/clang
CFLAGS = -O2 -Wall -Wextra -I../../include

# Linker flags for WASM
LDFLAGS = -Wl,--no-entry \
          -Wl,--export=init \
          -Wl,--export=update \
          -Wl,--export=render \
          -Wl,--allow-undefined

# Target and sources
TARGET = game.wasm
SRCS = game.c

.PHONY: all clean run

all: $(TARGET)

$(TARGET): $(SRCS)
	$(CC) $(CFLAGS) $(LDFLAGS) -o $@ $^

clean:
	rm -f $(TARGET)

run: $(TARGET)
	ember run $(TARGET)

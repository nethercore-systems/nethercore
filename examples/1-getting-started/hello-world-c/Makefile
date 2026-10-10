# Optional shortcut. `nether build` needs Clang/LLD, not make or wasi-sdk.
CC = clang
CFLAGS = --target=wasm32-unknown-unknown -ffreestanding -nostdlib -Oz -Wall -Wextra -Werror -I../../../include
LDFLAGS = -Wl,--no-entry -Wl,--export=init -Wl,--export=update -Wl,--export=render -Wl,--allow-undefined

.PHONY: all clean run
all: game.wasm

game.wasm: game.c ../../../include/zx.h
	$(CC) $(CFLAGS) $(LDFLAGS) -o $@ game.c

clean:
	rm -f game.wasm

run:
	nether run

# chip-8

A CHIP-8 emulator written in Rust, built to learn the language.

- Runs on the desktop (window, sound and keyboard) and in the browser (WebAssembly).
- Two quirk presets: `modern` and `original` (COSMAC VIP).
- Built-in debugger: pause, single-step and breakpoints.

## Run it

You need a CHIP-8 ROM (not included). The [CHIP-8 test suite](https://github.com/Timendus/chip8-test-suite) and
[chip8-roms](https://github.com/kripod/chip8-roms) have good ones.

    cargo run -- path/to/game.ch8

Options: `--speed <100-3000>` (instructions per second), `--scale <1|2|4|8|16|32>`,
`--quirks <modern|original>`, `--break <hex address>` (repeatable).

## Controls

The CHIP-8 keypad maps to the left block of your keyboard:

    CHIP-8      Keyboard
    1 2 3 C     1 2 3 4
    4 5 6 D     Q W E R
    7 8 9 E     A S D F
    A 0 B F     Z X C V

`Esc` quits, `Up`/`Down` change the speed, `Space` pauses and `N` runs one instruction while paused.

## In the browser

    cd web
    wasm-pack build --target web
    python3 -m http.server 8000

Then open <http://localhost:8000> and choose a ROM.

## Tests

    cargo test

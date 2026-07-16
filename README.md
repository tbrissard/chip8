# Chip8

A simple chip8 emulator/interpreter written in Rust.

## Build

```bash
git clone https://github.com/tbrissard/chip8.git
cd chip8/
cargo build --release
```

## Usage

(subject to change)

```bash
chip8 run "path_to_chip8_rom"
```

You can find chip8 ROMs [here](https://github.com/dmatlack/chip8/tree/master/roms).

### Command line options

```
--clock-speed <SPEED>    Change the speed at which the emulator runs (in instructions per second)
```

## Todo

- add documentation
- async
- check test coverage and add tests where missing
- emit sound when sound timer is on (or find a way to signal it)
- add support for some chip8 extension (superchip, megachip, etc)
- add keyboard mapping
- state saving

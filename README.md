# Chip8

A simple chip8 emulator/interpreter written in Rust.

[http://i.imgur.com/pcln2f5.gif](http://i.imgur.com/pcln2f5.gif)

## Compilation

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

- display available shortcuts
- look into mpsc channels and async/event-driven
- check test coverage and add tests where missing
- add a "step" keybind
- emit sound when sound timer is on (or find a way to signal it)
- add support for some chip8 extension (superchip, megachip, etc)
- add keyboard mapping

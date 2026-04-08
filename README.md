# Chip8

A simple chip8 emulator/interpreter written in Rust.

<<<<<<< HEAD
![http://i.imgur.com/pcln2f5.gif](http://i.imgur.com/pcln2f5.gif)
=======
[http://i.imgur.com/pcln2f5.gif](http://i.imgur.com/pcln2f5.gif)
>>>>>>> cd8790a (do not try to render gif, simply display imgur link)

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

- organize keyboard display in a 4x4 grid
- display available shortcuts
- look into mpsc channels and async/event-driven
- check test coverage and add tests where missing
- add a "step" keybind
- add support for some chip8 extension (superchip, megachip, etc)
- add keyboard mapping

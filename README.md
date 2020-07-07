# Rust_roguelike
Roguelike (or rogue-like) is a subgenre of role-playing video games characterized by a dungeon crawl through procedurally generated levels, turn-based gameplay, tile-based graphics, and permanent death of the player character. Most roguelikes are based on a high fantasy narrative, reflecting their influence from tabletop role playing games such as Dungeons & Dragons. (Referenced: https://en.wikipedia.org/wiki/Roguelike)


# Installing Rust
To install Rust, go to the [Rust homepage](https://www.rust-lang.org/) and click the Install button.
Creating the roguelike project

Next we need to create the Rust project that will be our roguelike:
```bash
$ cargo new --bin roguelike
$ cd roguelike/
```

# Installing
You must then install the dependencies for [tcod](https://github.com/tomassedovic/tcod-rs) — the Rust bindings for libtcod.

To use tcod-rs, add this to your game's Cargo.toml:
```Rust
[dependencies]
tcod = "0.15"
```

## Building on linux:
```bash
$ sudo apt-get install gcc g++ make libsdl2-dev
$ cd yourgame
$ cargo build --release
$ cargo run --release
```
## Building a dynamic library:
By default, tcod-rs will build the library statically on Linux as including the code into the executable is usually more convenient. To build a dynamic library specify the dynlib feature for tcod-sys in Cargo.toml
```rust
[dependencies.tcod-sys]
version = "*"
features = ["dynlib"]
```

## Prerequisites:
Download the picture "[sprites.png](https://github.com/qiaw99/rust_roguelike/blob/added-hud%2C-weapons-and-graphics/sprites.png)" and put it into the same directory of Cargo.toml.

# Contributers:
- [Thore Brehmer](https://github.com/theroIdond)
- [Qianli Wang](https://github.com/qiaw99)
- Jonny Lam
- David Ly

# License:
This project is licensed under the MIT License - see the LICENSE file for details


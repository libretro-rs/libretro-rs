# libretro-rs

## Getting started

A reference implementation is available in the `libretro-rs-impl` folder, and should be enough to get you started. At a high level, you need to modify your `Cargo.toml`:

```toml
[dependencies]
libretro_rs = "0.1"

[lib]
crate-type = ["cdylib"]
```

Then implement a trait and call a macro:

```rust
use libretro_rs::*;

struct Emulator {
  // ...
}

// Note: RetroCore requires that your type also implements `Default`.
impl RetroCore for Emulator {
  // ...
}

libretro_core!(Emulator);
```

et voilà! Running `cargo build` will produce a shared library (`.so`, `.dll`, etc) that you can use with a libretro front-end:

```shell
$ retroarch --verbose -L libemulator.so /path/to/game.rom
```

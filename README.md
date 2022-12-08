# libretro-rs

## Design Philosophy

The approach to this crate can best be summarized as wanting to expose all functionality, even if not idiomatically. The `libretro` API has quite a bit of functionality, and implementing all of that up front would just cause delays. Instead, all of the raw API is made available to you. If you run into something that isn't explicitly supported, you can always use the raw API.

Another design note is to try and include data that would be available to a C implementation. For example, emulators cannot reasonably be expected to be constructed without parameters (à la `Default`). Therefore, there is a `RetroCore::init` function that can acccess the `RetroEnvironment`, so things like asset paths can be used to construct the emulator. With that in mind, care will be taken to hide API functionality where it isn't allowed.

After the above requirements are met, the last goal is to make the bindings logical and ergonomic. If you're feeling pain anywhere in the API then definitely let us know!

## Getting started

A reference implementation is available in the `example` folder, and should be enough to get you started. At a high level, you need to modify your `Cargo.toml`:

```toml
[dependencies]
libretro-rs = "0.1"

[lib]
crate-type = ["cdylib"]
```

Then implement a trait and call a macro:

```rust
use libretro_rs::*;

struct Emulator {
  // ...
}

impl RetroCore for Emulator {
  // ...
}

libretro_core!(Emulator);
```

et voilà! Running `cargo build` will produce a shared library (`.so`, `.dll`, etc) that you can use with a libretro front-end:

```shell
$ retroarch --verbose -L libemulator.so /path/to/game.rom
```

## Example

The `example` folder contains a working CHIP-8 emulator. You can run it using this command:

```shell
$ cargo build -p example
$ retroarch --verbose -L ./target/debug/libexample.so <game path>
```

Replace `.so` with `.dll` if running on Windows.

extern crate bindgen;

use std::env;
use std::path::PathBuf;

const LIBRETRO_HEADER_FILE: &'static str = "include/libretro.h";

fn main() {
  println!("cargo:rerun-if-changed={}", LIBRETRO_HEADER_FILE);

  let bindings = bindgen::Builder::default()
    .header(LIBRETRO_HEADER_FILE)
    .allowlist_type("^retro_.+$")
    .allowlist_var("^RETRO_.+$")
    .default_enum_style(bindgen::EnumVariation::Rust { non_exhaustive: false })
    .derive_default(true)
    .layout_tests(false)
    .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    .generate()
    .expect("Unable to generate bindings");

  let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
  bindings
    .write_to_file(out_path.join("libretro.rs"))
    .expect("Couldn't write bindings!");
}

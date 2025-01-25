// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2022 Juan Palacios <jpalaciosdev@gmail.com>

extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=cnproc_wrapper.h");

    let cnproc_bindings = bindgen::Builder::default()
        .header("cnproc_wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .use_core()
        .ctypes_prefix("libc") // use the types from libc package
        .raw_line("use libc::*;") // use libc package
        .generate()
        .expect("Unable to generate cnproc bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    cnproc_bindings
        .write_to_file(out_path.join("cnproc_bindings.rs"))
        .expect("Couldn't write cnproc bindings!");
}

extern crate pkg_config;
extern crate gcc;

use std::env;

fn main() {
    let mut gcc_config = gcc::Config::new();
    gcc_config.cpp(true);

    let lib_dir = env::var("PROTOBUF_LIB_DIR").ok();
    let include_dir = env::var("PROTOBUF_INCLUDE_DIR").ok();

    if let Some(ref lib_dir) = lib_dir {
    	println!("cargo:rustc-link-search=native={}", lib_dir);
    }

    if let Some(ref include_dir) = include_dir {
        println!("cargo:include={}", include_dir);
        gcc_config.include(include_dir);
    }

    if lib_dir.is_none() && include_dir.is_none() {
        let protobuf_lib = pkg_config::find_library("protobuf").unwrap();
        for path in protobuf_lib.include_paths {
            gcc_config.include(path);
        }
    }

    println!("cargo:rustc-flags=-l protoc");

    gcc_config.file("src/glue.cpp")
              .compile("librust-protobuf-build-glue.a");
}

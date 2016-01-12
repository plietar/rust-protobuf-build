extern crate pkg_config;
extern crate gcc;

fn main() {
    let protobuf_lib = pkg_config::find_library("protobuf").unwrap();
    println!("cargo:rustc-flags=-l protoc");

    let mut gcc_config = gcc::Config::new();
    gcc_config.cpp(true);
    for path in protobuf_lib.include_paths {
        gcc_config.include(path.as_path());
    }

    gcc_config.file("src/glue.cpp")
              .compile("librust-protobuf-build-glue.a");
}

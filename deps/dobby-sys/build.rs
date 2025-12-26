use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    if target.contains("ios") || target.contains("darwin") {
        // Link against the prebuilt library in tools/ios/dobby_build
        // We assume the project root is at ../../ from deps/dobby-sys
        let root_dir = PathBuf::from(manifest_dir)
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        let lib_dir = root_dir.join("tools").join("ios").join("dobby_build");

        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        println!("cargo:rustc-link-lib=static=dobby");
        println!(
            "cargo:rerun-if-changed={}",
            lib_dir.join("libdobby.a").display()
        );
    }
}

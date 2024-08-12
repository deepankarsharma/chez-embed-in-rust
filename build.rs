use std::env;
use std::path::PathBuf;

fn main() {
    let crate_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let scheme_lib_path = crate_root.join("scheme/ta6le");

    // Add scheme/ta6le to the library search path
    println!(
        "cargo:rustc-link-search=native={}",
        scheme_lib_path.display()
    );

    // Specify the directory where libchez.a is located (keeping the original path as well)
    println!("cargo:rustc-link-search=native=/home/dman/dataspace/scheme/ta6le");

    // Explicitly request static linking
    // println!("cargo:rustc-link-lib=static=chez");
    // If you're on Windows, you might need to use this instead:
    // println!("cargo:rustc-link-lib=static=libchez");

    // Link against dynamic libraries
    println!("cargo:rustc-link-lib=dylib=ncurses");
    println!("cargo:rustc-link-lib=dylib=tinfo"); // tinfo is often needed alongside ncurses
    println!("cargo:rustc-link-lib=dylib=m");
    println!("cargo:rustc-link-lib=dylib=pthread");

    // Specify include directory if necessary
    println!("cargo:rustc-link-search=native=/usr/local/include");

    // Force Cargo to rerun this script if it changes
    println!("cargo:rerun-if-changed=build.rs");
}

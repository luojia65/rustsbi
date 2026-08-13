use std::{env, fs, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo must set OUT_DIR"));
    fs::copy("rustsbi-machine.x", out_dir.join("rustsbi-machine.x"))
        .expect("failed to copy rustsbi-machine.x to OUT_DIR");

    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rerun-if-changed=rustsbi-machine.x");
}

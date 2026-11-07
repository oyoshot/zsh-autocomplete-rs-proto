use std::collections::hash_map::DefaultHasher;
use std::env;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;

fn hash_tree(path: &Path, hash: &mut DefaultHasher) {
    path.hash(hash);
    if path.is_dir() {
        let mut children: Vec<_> = fs::read_dir(path)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();
        children.sort();
        for child in children {
            hash_tree(&child, hash);
        }
    } else {
        fs::read(path).unwrap().hash(hash);
    }
}

fn main() {
    let mut hash = DefaultHasher::new();
    for path in ["src", "shell", "Cargo.toml", "Cargo.lock", "build.rs"] {
        println!("cargo:rerun-if-changed={path}");
        hash_tree(Path::new(path), &mut hash);
    }
    for key in [
        "TARGET",
        "PROFILE",
        "OPT_LEVEL",
        "DEBUG",
        "CARGO_ENCODED_RUSTFLAGS",
    ] {
        env::var_os(key).hash(&mut hash);
    }
    let compiler = std::process::Command::new(env::var_os("RUSTC").unwrap())
        .arg("--version")
        .output()
        .unwrap();
    compiler.stdout.hash(&mut hash);
    println!("cargo:rustc-env=ZACRS_BUILD_ID={:016x}", hash.finish());
}

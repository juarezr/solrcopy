// build.rs

#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;

#[path = "src/helpers.rs"]
mod helpers;
include!("src/args.rs");

fn main() {
    println!("cargo:rerun-if-env-changed=PROFILE");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/args.rs");

    // helpers::print_env_vars();

    build_artifacts();
}

pub(crate) fn build_artifacts() {
    let pkg_name = std::env::var("CARGO_PKG_NAME").expect("Missing $CARGO_PKG_NAME!");
    let pkg_dir = std::env::var("CARGO_MANIFEST_DIR").expect("Missing $CARGO_MANIFEST_DIR");

    let release = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let out_dir = format!("{}/target/{}", pkg_dir, release);
    eprintln!("# ARTIFACT_DIR: {}", out_dir);

    let mut clap = Arguments::clap();
    clap.gen_completions(&pkg_name, clap::Shell::Bash, &out_dir);
    clap.gen_completions(&pkg_name, clap::Shell::Fish, &out_dir);
    clap.gen_completions(pkg_name, clap::Shell::Zsh, &out_dir);
}

// end of build script \\

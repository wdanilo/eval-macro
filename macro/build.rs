#![allow(clippy::unwrap_used)]

fn main() {
    println!("cargo::rustc-check-cfg=cfg(nightly)");
    if rustc_version::version_meta().unwrap().channel == rustc_version::Channel::Nightly {
        println!("cargo:rustc-cfg=nightly");
    }
    println!("cargo:rerun-if-changed=build.rs");
}

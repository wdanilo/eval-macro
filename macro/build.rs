fn main() {
    println!("cargo::rustc-check-cfg=cfg(nightly)");
    if rustc_version::version_meta().unwrap().channel == rustc_version::Channel::Nightly {
        println!("cargo:rustc-cfg=nightly");
    }
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    let dest_path = std::path::Path::new(&out_dir).join("build_constants.rs");
    let code = format!("const OUT_DIR: &str = {:?};", out_dir);
    std::fs::write(&dest_path, code).expect("Failed to write build_constants.rs");
    println!("cargo:rerun-if-changed=build.rs");
}

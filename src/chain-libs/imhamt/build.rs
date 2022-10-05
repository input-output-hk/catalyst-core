use rustc_version::{version_meta, Channel};

fn main() {
    // if compiling with a nightly compiler, enable the "nightly" feature
    if version_meta().unwrap().channel == Channel::Nightly {
        println!("cargo:rustc-cfg=feature=\"nightly\"");
    }
}

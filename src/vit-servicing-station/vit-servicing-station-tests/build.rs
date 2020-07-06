fn main() {
    let vit_bin_name = option_env!("VIT_BIN_NAME").unwrap_or("vit-servicing-station-server");
    println!("cargo:rustc-env=VIT_BIN_NAME={}", vit_bin_name);
    println!("cargo:rustc-env=RUST_BACKTRACE=full");
}

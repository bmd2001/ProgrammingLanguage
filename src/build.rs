fn main() {
    if let Ok(arch) = std::env::var("CARGO_CFG_TARGET_ARCH") {
        println!("cargo:rustc-env=MY_TARGET_ARCH={}", arch);
    }
    if let Ok(os) = std::env::var("CARGO_CFG_TARGET_OS") {
        println!("cargo:rustc-env=MY_TARGET_OS={}", os);
    }
}

fn main() {
    if cfg!(feature = "desktop") || cfg!(feature = "mobile") {
        println!("cargo:rustc-cfg=sqlite_mode");
    }
}

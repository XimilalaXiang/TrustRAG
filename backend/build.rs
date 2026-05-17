fn main() {
    println!("cargo::rustc-check-cfg=cfg(sqlite_mode)");
    if cfg!(feature = "desktop") || cfg!(feature = "mobile") {
        println!("cargo:rustc-cfg=sqlite_mode");
    }
}

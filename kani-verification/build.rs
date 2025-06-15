fn main() {
    println!("cargo::rustc-check-cfg=cfg(kani)");
    
    if std::env::var("KANI").is_ok() {
        println!("cargo::rustc-cfg=kani");
    }
}

fn main() {
    println!("cargo::rerun-if-changed=src/main.rs");

    // Linking OpenFHE
    println!("cargo::rustc-link-arg=-L/usr/local/lib");
    println!("cargo::rustc-link-arg=-lOPENFHEpke");
    println!("cargo::rustc-link-arg=-lOPENFHEbinfhe");
    println!("cargo::rustc-link-arg=-lOPENFHEcore");

    // Linking OpenMP
    println!("cargo::rustc-link-arg=-fopenmp");

    // Necessary to avoid LD_LIBRARY_PATH
    println!("cargo::rustc-link-arg=-Wl,-rpath,/usr/local/lib");
}

use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo::rerun-if-changed=src/main.rs");
    println!("cargo::rerun-if-changed=build.rs");

    // Check if OpenFHE is installed
    let openfhe_root = std::env::var("OPENFHE_ROOT").unwrap_or_else(|_| "/usr/local".to_string());
    let lib_path = format!("{}/lib", openfhe_root);
    let include_path = format!("{}/include", openfhe_root);

    // Verify OpenFHE installation
    if !Path::new(&format!("{}/libOPENFHEcore.so", lib_path)).exists() &&
       !Path::new(&format!("{}/libOPENFHEcore.a", lib_path)).exists() {
        panic!(
            "OpenFHE not found at {}. Please install OpenFHE from https://github.com/MachinaIO/openfhe-development (feat/improve_determinant branch) to /usr/local",
            lib_path
        );
    }

    // Check for pkg-config
    if let Ok(output) = Command::new("pkg-config")
        .args(&["--exists", "openfhe"])
        .output()
    {
        if output.status.success() {
            // Use pkg-config if available
            let libs = Command::new("pkg-config")
                .args(&["--libs", "openfhe"])
                .output()
                .expect("Failed to run pkg-config");
            
            let cflags = Command::new("pkg-config")
                .args(&["--cflags", "openfhe"])
                .output()
                .expect("Failed to run pkg-config");

            println!("cargo::rustc-flags={}", String::from_utf8_lossy(&libs.stdout).trim());
            println!("cargo::rustc-flags={}", String::from_utf8_lossy(&cflags.stdout).trim());
        }
    }

    // Fallback to manual linking
    println!("cargo::rustc-link-search=native={}", lib_path);
    println!("cargo::rustc-link-lib=OPENFHEpke");
    println!("cargo::rustc-link-lib=OPENFHEbinfhe");
    println!("cargo::rustc-link-lib=OPENFHEcore");
    
    // Link OpenMP if available
    if cfg!(target_os = "linux") {
        println!("cargo::rustc-link-lib=gomp");
    } else if cfg!(target_os = "macos") {
        println!("cargo::rustc-link-lib=omp");
    }

    // Set rpath for runtime library loading
    println!("cargo::rustc-link-arg=-Wl,-rpath,{}", lib_path);

    // Set environment variables for dependent crates
    println!("cargo::rustc-env=OPENFHE_ROOT={}", openfhe_root);
    println!("cargo::rustc-env=OPENFHE_LIB_DIR={}", lib_path);
    println!("cargo::rustc-env=OPENFHE_INCLUDE_DIR={}", include_path);
}

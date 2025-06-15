use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo::rerun-if-changed=src/main.rs");
    println!("cargo::rerun-if-changed=build.rs");

    // Enable Kani verification cfg
    println!("cargo::rustc-check-cfg=cfg(kani)");

    // Check if OpenFHE is installed
    let openfhe_root = env::var("OPENFHE_ROOT").unwrap_or_else(|_| "/usr/local".to_string());
    let lib_path = format!("{openfhe_root}/lib");
    let include_path = format!("{openfhe_root}/include");

    // Verify OpenFHE installation - check all required libraries
    let required_libs = ["libOPENFHEcore", "libOPENFHEpke", "libOPENFHEbinfhe"];
    let mut missing_libs = Vec::new();

    for lib in &required_libs {
        let so_path = format!("{lib_path}/{lib}.so");
        let a_path = format!("{lib_path}/{lib}.a");

        if !Path::new(&so_path).exists() && !Path::new(&a_path).exists() {
            missing_libs.push(lib);
        }
    }

    if !missing_libs.is_empty() {
        panic!(
            "OpenFHE libraries not found at {lib_path}: {missing_libs:?}. Please install OpenFHE from https://github.com/MachinaIO/openfhe-development (feat/improve_determinant branch) to /usr/local"
        );
    }

    // Verify OpenFHE headers
    let openfhe_include = format!("{include_path}/openfhe");
    if !Path::new(&openfhe_include).exists() {
        panic!(
            "OpenFHE headers not found at {openfhe_include}. Please install OpenFHE development headers."
        );
    }

    // Set C++ compiler flags for cc-rs and cxx crates
    println!("cargo::rustc-env=CXXFLAGS=-std=c++17 -O2 -DNDEBUG");
    println!("cargo::rustc-env=CXX_FLAGS=-std=c++17 -O2 -DNDEBUG");

    // Disable problematic compiler warnings that cause errors
    env::set_var("CXXFLAGS", "-std=c++17 -O2 -DNDEBUG -Wno-unused-parameter -Wno-unused-function -Wno-missing-field-initializers");
    env::set_var("CXX_FLAGS", "-std=c++17 -O2 -DNDEBUG -Wno-unused-parameter -Wno-unused-function -Wno-missing-field-initializers");

    // Check for pkg-config
    if let Ok(output) = Command::new("pkg-config")
        .args(["--exists", "openfhe"])
        .output()
    {
        if output.status.success() {
            // Use pkg-config if available
            let libs = Command::new("pkg-config")
                .args(["--libs", "openfhe"])
                .output()
                .expect("Failed to run pkg-config");

            let cflags = Command::new("pkg-config")
                .args(["--cflags", "openfhe"])
                .output()
                .expect("Failed to run pkg-config");

            println!(
                "cargo::rustc-flags={}",
                String::from_utf8_lossy(&libs.stdout).trim()
            );
            println!(
                "cargo::rustc-flags={}",
                String::from_utf8_lossy(&cflags.stdout).trim()
            );
        }
    }

    // Fallback to manual linking
    println!("cargo::rustc-link-search=native={lib_path}");
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
    println!("cargo::rustc-link-arg=-Wl,-rpath,{lib_path}");

    // Set environment variables for dependent crates
    println!("cargo::rustc-env=OPENFHE_ROOT={openfhe_root}");
    println!("cargo::rustc-env=OPENFHE_LIB_DIR={lib_path}");
    println!("cargo::rustc-env=OPENFHE_INCLUDE_DIR={include_path}");
}

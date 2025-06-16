use std::{env, path::Path, process::Command};

fn main() {
    println!("cargo::rerun-if-changed=src/main.rs");
    println!("cargo::rerun-if-changed=build.rs");

    // Enable Kani verification cfg
    println!("cargo::rustc-check-cfg=cfg(kani)");

    // Setup OpenFHE environment
    if let Err(e) = setup_openfhe() {
        eprintln!("Warning: OpenFHE setup failed: {e}");
        eprintln!("Build may fail if OpenFHE libraries are required at runtime");
    }
}

fn setup_openfhe() -> Result<(), String> {
    // Check if OpenFHE is installed
    let openfhe_root = env::var("OPENFHE_ROOT").unwrap_or_else(|_| "/usr/local".to_string());
    let lib_path = format!("{openfhe_root}/lib");
    let include_path = format!("{openfhe_root}/include");

    println!("cargo::rustc-env=OPENFHE_ROOT={openfhe_root}");
    println!("cargo::rustc-env=OPENFHE_LIB_DIR={lib_path}");
    println!("cargo::rustc-env=OPENFHE_INCLUDE_DIR={include_path}");

    // For cxx crate: provide include paths
    println!("cargo::rustc-env=DEP_OPENFHE_INCLUDE={include_path}");

    // Additional include paths to try
    let include_paths = vec![
        include_path.clone(),
        format!("{openfhe_root}/include/openfhe"),
        "/usr/include/openfhe".to_string(),
        "/usr/local/include/openfhe".to_string(),
        "/opt/homebrew/include/openfhe".to_string(),
    ];

    // Find a valid include path and check for key headers
    let mut found_include = false;
    for path in &include_paths {
        if Path::new(path).exists() {
            // Check for key OpenFHE headers
            let core_header = format!("{path}/core/include/lattice/lat-hal.h");
            let pke_header = format!("{path}/pke/include/scheme/scheme-id.h");
            let binfhe_header = format!("{path}/binfhe/include/binfhe.h");

            if Path::new(&core_header).exists()
                || Path::new(&pke_header).exists()
                || Path::new(&binfhe_header).exists()
            {
                println!("cargo::include={path}");
                println!("cargo::rustc-env=OPENFHE_INCLUDE_PATH={path}");
                found_include = true;
                break;
            }
        }
    }

    if !found_include {
        eprintln!("Warning: OpenFHE headers not found in any of: {include_paths:?}");
        eprintln!("Please install OpenFHE or set OPENFHE_ROOT environment variable");
        // Continue anyway - might be available through pkg-config or CI cache
    }

    // Verify OpenFHE installation - check all required libraries
    let lib_paths = vec![
        lib_path.clone(),
        "/usr/lib".to_string(),
        "/usr/local/lib".to_string(),
        "/opt/homebrew/lib".to_string(),
        "/usr/lib/x86_64-linux-gnu".to_string(), // Ubuntu path
    ];

    let required_libs = ["libOPENFHEcore", "libOPENFHEpke", "libOPENFHEbinfhe"];
    let mut found_libs = false;
    let mut found_lib_path = String::new();

    for lib_dir in &lib_paths {
        let mut all_found = true;
        for lib in &required_libs {
            let so_path = format!("{lib_dir}/{lib}.so");
            let a_path = format!("{lib_dir}/{lib}.a");
            let dylib_path = format!("{lib_dir}/{lib}.dylib");

            if !Path::new(&so_path).exists()
                && !Path::new(&a_path).exists()
                && !Path::new(&dylib_path).exists()
            {
                all_found = false;
                break;
            }
        }
        if all_found {
            found_libs = true;
            found_lib_path = lib_dir.clone();
            println!("cargo::rustc-link-search=native={lib_dir}");
            break;
        }
    }

    if !found_libs {
        eprintln!("Warning: OpenFHE libraries not found in standard locations");
        eprintln!("Searched in: {lib_paths:?}");
        eprintln!("Please install OpenFHE from https://github.com/MachinaIO/openfhe-development");
        eprintln!("Using fallback library path: {lib_path}");
        println!("cargo::rustc-link-search=native={lib_path}");
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
        // Try to find libomp from Homebrew
        let homebrew_paths = vec![
            "/opt/homebrew/lib".to_string(),
            "/usr/local/lib".to_string(),
        ];

        let mut found_omp = false;
        for lib_dir in &homebrew_paths {
            let omp_lib = format!("{lib_dir}/libomp.dylib");
            if Path::new(&omp_lib).exists() {
                println!("cargo::rustc-link-search=native={lib_dir}");
                println!("cargo::rustc-link-lib=omp");
                found_omp = true;
                break;
            }
        }

        if !found_omp {
            eprintln!("Warning: OpenMP library not found on macOS");
            eprintln!("Consider installing with: brew install libomp");
            // Don't fail the build - OpenFHE might be built without OpenMP
        }
    }

    // Set rpath for runtime library loading
    if !found_lib_path.is_empty() {
        println!("cargo::rustc-link-arg=-Wl,-rpath,{found_lib_path}");
    } else {
        println!("cargo::rustc-link-arg=-Wl,-rpath,{lib_path}");
    }

    Ok(())
}

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
    // Only show verbose warnings if OPENFHE_DEBUG is set
    let verbose = env::var("OPENFHE_DEBUG").is_ok();

    if verbose {
        println!("cargo::warning=Starting OpenFHE setup process");
    }

    // Check if OpenFHE is installed
    let openfhe_root = env::var("OPENFHE_ROOT").unwrap_or_else(|_| "/usr/local".to_string());
    let lib_path = format!("{openfhe_root}/lib");
    let include_path = format!("{openfhe_root}/include");

    if verbose {
        println!("cargo::warning=OPENFHE_ROOT: {openfhe_root}");
        println!("cargo::warning=Library path: {lib_path}");
        println!("cargo::warning=Include path: {include_path}");
    }

    println!("cargo::rustc-env=OPENFHE_ROOT={openfhe_root}");
    println!("cargo::rustc-env=OPENFHE_LIB_DIR={lib_path}");
    println!("cargo::rustc-env=OPENFHE_INCLUDE_DIR={include_path}");

    // For cxx crate: provide include paths
    println!("cargo::rustc-env=DEP_OPENFHE_INCLUDE={include_path}");

    // Check CPATH environment variable for additional include paths
    let mut include_paths = vec![
        include_path.clone(),
        format!("{openfhe_root}/include/openfhe"),
        "/usr/include/openfhe".to_string(),
        "/usr/local/include/openfhe".to_string(),
        "/opt/homebrew/include/openfhe".to_string(),
    ];

    // Add CPATH directories if available
    if let Ok(cpath) = env::var("CPATH") {
        for path in cpath.split(':') {
            include_paths.push(path.to_string());
            include_paths.push(format!("{path}/openfhe"));
        }
    }

    // Additional common include paths for OpenFHE
    include_paths.extend(vec![
        "/usr/local/include".to_string(),
        "/usr/include".to_string(),
        format!("{openfhe_root}/include"),
    ]);

    // Find a valid include path and check for key headers
    let mut found_include = false;
    for path in &include_paths {
        if Path::new(path).exists() {
            // Check for key OpenFHE headers that are referenced in the error messages
            let critical_headers = vec![
                // Primary patterns from CI errors
                format!("{path}/openfhe/core/lattice/hal/lat-backend.h"),
                format!("{path}/openfhe/binfhe/lwe-ciphertext-fwd.h"),
                format!("{path}/openfhe/core/utils/exception.h"),
                // Alternative include patterns
                format!("{path}/openfhe/core/include/lattice/hal/lat-backend.h"),
                format!("{path}/openfhe/binfhe/include/lwe-ciphertext-fwd.h"),
                format!("{path}/openfhe/core/include/utils/exception.h"),
                // Direct directory patterns (fallback)
                format!("{path}/core/lattice/hal/lat-backend.h"),
                format!("{path}/binfhe/lwe-ciphertext-fwd.h"),
                format!("{path}/core/utils/exception.h"),
                // Additional critical OpenFHE headers
                format!("{path}/openfhe/core/lattice/hal/lat-hal.h"),
                format!("{path}/openfhe/pke/include/scheme/scheme-id.h"),
                format!("{path}/openfhe/binfhe/include/binfhe.h"),
            ];

            // Also check for common OpenFHE headers to verify installation
            let common_headers = vec![
                format!("{path}/openfhe/core/include/lattice/lat-hal.h"),
                format!("{path}/openfhe/pke/include/scheme/scheme-id.h"),
                format!("{path}/openfhe/binfhe/include/binfhe.h"),
                format!("{path}/core/include/lattice/lat-hal.h"),
                format!("{path}/pke/include/scheme/scheme-id.h"),
                format!("{path}/binfhe/include/binfhe.h"),
            ];

            let mut found_critical = false;
            let mut found_common = false;

            // Check for critical headers
            for header in &critical_headers {
                if Path::new(header).exists() {
                    found_critical = true;
                    if verbose {
                        println!("cargo::warning=Found critical header: {header}");
                    }
                    break;
                }
            }

            // Check for common headers as fallback
            if !found_critical {
                for header in &common_headers {
                    if Path::new(header).exists() {
                        found_common = true;
                        if verbose {
                            println!("cargo::warning=Found common header: {header}");
                        }
                        break;
                    }
                }
            }

            if found_critical || found_common {
                println!("cargo::rustc-env=OPENFHE_INCLUDE_PATH={path}");

                // Also add openfhe subdirectory if it exists
                let openfhe_subdir = format!("{path}/openfhe");
                if Path::new(&openfhe_subdir).exists() {
                    println!("cargo::rustc-env=OPENFHE_INCLUDE_SUBDIR={openfhe_subdir}");
                    if verbose {
                        println!(
                            "cargo::warning=Also including OpenFHE subdirectory: {openfhe_subdir}"
                        );
                    }
                }

                if verbose {
                    let header_type = if found_critical { "critical" } else { "common" };
                    println!("cargo::warning=Found OpenFHE {header_type} headers in: {path}");

                    // List some of the found headers for debugging
                    println!("cargo::warning=Verified header files:");
                    for header in &critical_headers {
                        if Path::new(header).exists() {
                            println!("cargo::warning=  ✅ {header}");
                        }
                    }
                    for header in &common_headers {
                        if Path::new(header).exists() {
                            println!("cargo::warning=  ✅ {header}");
                        }
                    }
                }
                found_include = true;
                break;
            }
        }
    }

    if !found_include {
        if verbose {
            eprintln!("Warning: OpenFHE headers not found in any of: {include_paths:?}");
            eprintln!("Please install OpenFHE or set OPENFHE_ROOT environment variable");
            println!("cargo::warning=OpenFHE headers not found in: {include_paths:?}");
        }
        // Continue anyway - might be available through pkg-config or CI cache
    } else if verbose {
        println!("cargo::warning=OpenFHE headers found and verified");
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
        if verbose {
            eprintln!("Warning: OpenFHE libraries not found in standard locations");
            eprintln!("Searched in: {lib_paths:?}");
            eprintln!(
                "Please install OpenFHE from https://github.com/MachinaIO/openfhe-development"
            );
            eprintln!("Using fallback library path: {lib_path}");
            println!("cargo::warning=OpenFHE libraries not found, searched in: {lib_paths:?}");
        }
        println!("cargo::rustc-link-search=native={lib_path}");
    } else if verbose {
        println!("cargo::warning=OpenFHE libraries found in: {found_lib_path}");
    }

    // Set C++ compiler flags for cc-rs and cxx crates
    println!("cargo::rustc-env=CXXFLAGS=-std=c++17 -O2 -DNDEBUG");
    println!("cargo::rustc-env=CXX_FLAGS=-std=c++17 -O2 -DNDEBUG");

    // Set include paths for C++ compilation
    if found_include {
        for path in &include_paths {
            if Path::new(path).exists() {
                println!("cargo::rustc-env=CPATH={path}");
                // Also set individual include directories
                let openfhe_subdir = format!("{path}/openfhe");
                if Path::new(&openfhe_subdir).exists() {
                    println!("cargo::rustc-env=CPATH={openfhe_subdir}");
                }
                break; // Use the first valid path
            }
        }
    }

    // Disable problematic compiler warnings that cause errors
    let cxx_flags = "-std=c++17 -O2 -DNDEBUG -Wno-unused-parameter -Wno-unused-function -Wno-missing-field-initializers";
    env::set_var("CXXFLAGS", cxx_flags);
    env::set_var("CXX_FLAGS", cxx_flags);

    // Set additional include paths in environment
    if let Ok(existing_cpath) = env::var("CPATH") {
        env::set_var(
            "CPATH",
            format!("{existing_cpath}:/usr/local/include:/usr/local/include/openfhe"),
        );
    } else {
        env::set_var("CPATH", "/usr/local/include:/usr/local/include/openfhe");
    }

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

    // Add additional library search paths for tarpaulin compatibility
    println!("cargo::rustc-link-search=native=/usr/local/lib");
    println!("cargo::rustc-link-search=native=/usr/lib");
    println!("cargo::rustc-link-search=native=/usr/lib/x86_64-linux-gnu");

    // Link OpenFHE libraries in correct order
    println!("cargo::rustc-link-lib=OPENFHEcore");
    println!("cargo::rustc-link-lib=OPENFHEpke");
    println!("cargo::rustc-link-lib=OPENFHEbinfhe");

    // Additional system libraries that OpenFHE may depend on
    println!("cargo::rustc-link-lib=ntl");
    println!("cargo::rustc-link-lib=gmp");
    println!("cargo::rustc-link-lib=stdc++");

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

    // Set rpath for runtime library loading - enhanced for tarpaulin
    if !found_lib_path.is_empty() {
        println!("cargo::rustc-link-arg=-Wl,-rpath,{found_lib_path}");
        println!("cargo::rustc-link-arg=-Wl,-rpath,/usr/local/lib");
    } else {
        println!("cargo::rustc-link-arg=-Wl,-rpath,{lib_path}");
        println!("cargo::rustc-link-arg=-Wl,-rpath,/usr/local/lib");
    }

    // Additional rpath entries for system libraries
    println!("cargo::rustc-link-arg=-Wl,-rpath,/usr/lib/x86_64-linux-gnu");
    println!("cargo::rustc-link-arg=-Wl,-rpath,/lib/x86_64-linux-gnu");

    // Enable additional linker flags for better compatibility
    println!("cargo::rustc-link-arg=-Wl,--enable-new-dtags");

    // For tarpaulin: ensure libraries are found at runtime
    if env::var("CARGO_TARPAULIN").is_ok() {
        println!(
            "cargo::warning=Detected tarpaulin execution, applying additional linker settings"
        );
        println!("cargo::rustc-link-arg=-Wl,--no-as-needed");
    }

    Ok(())
}

fn main() {
    let target = std::env::var("TARGET").unwrap();
    let wasi = target.contains("wasi");

    let mut cmake_build = cmake::Config::new("../../../../cpp/perspective");
    if wasi {
        let sdk =
            std::env::var("WASI_SDK_PATH").expect("WASI_SDK_PATH must be set for wasi builds");
        println!(
            "cargo:rustc-link-search=native={}/share/wasi-sysroot/lib/wasm32-wasi",
            sdk
        );
        cmake_build
            .define("PSP_ENABLE_WASM", "1")
            .define("PSP_PARALLEL_FOR", "0")
            .define(
                "CMAKE_TOOLCHAIN_FILE",
                format!("{}/share/cmake/wasi-sdk.cmake", sdk),
            )
            .build();
    } else {
        cmake_build
            .define("PSP_WASM_BUILD", "OFF")
            .define("PSP_CPP_BUILD", "ON")
            .build();
    }
    let cmake_out = cmake_build.build();
    let out_path = cmake_out.join("build");
    let out_path_str = out_path.display().to_string();
    println!("cargo:rustc-link-search=native={}", out_path_str);
    println!(
        "cargo:rustc-link-search=native={}",
        out_path.join("re2-build").display().to_string()
    );

    if wasi {
        println!("cargo:rustc-link-lib=static=c++abi");
    }

    let mut builder = cxx_build::bridge("src/ffi.rs");
    builder
        .file("cpp/ffi.cpp")
        .include("cpp")
        .include("/usr/local/include")
        .include("../../../../cpp/perspective/src/include")
        .include(out_path.join("date-src/include"))
        .include(out_path.join("hopscotch-src/include"))
        .include(out_path.join("exprtk-src"))
        .include(out_path.join("re2-src"))
        .include(out_path.join("ordered-map-src/include"))
        .opt_level(3);

    if wasi {
        builder
            .flag("-D_WASI_EMULATED_MMAN")
            .flag("-D_WASI_EMULATED_SIGNAL")
            .flag("-DPSP_ENABLE_WASM");
    } else {
        builder.flag("-DPSP_CPP_BUILD=1");
    }
    builder.flag_if_supported("-std=c++1y");

    builder.compile("bridge");

    println!("cargo:rerun-if-changed=src/ffi.rs");
    println!("cargo:rerun-if-changed=src/cpp_common.rs");
    println!("cargo:rerun-if-changed=cpp/types.h");
    println!("cargo:rerun-if-changed=cpp/ffi.h");
    println!("cargo:rerun-if-changed=cpp/ffi.cpp");

    // TODO: Automate this
    if wasi {
        println!("cargo:rustc-link-search=native=/Users/timothybess/src/wasi-sdk/build/wasi-sdk-20.20g2393be41c8df/share/wasi-sysroot/lib/wasm32-wasi");
    }
    //

    println!("cargo:rustc-link-lib=static=psp");
    println!("cargo:rustc-link-lib=static=re2");
}

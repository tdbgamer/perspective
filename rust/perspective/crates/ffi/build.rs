fn main() {
    println!("cargo:rustc-link-lib=static=c++abi");

    cxx_build::bridge("src/ffi.rs")
        .file("cpp/ffi.cpp")
        .include("cpp")
        .include("/usr/local/include")
        .include("../../../../cpp/perspective/src/include")
        .include("../../../../cpp/perspective/dist/release/date-src/include")
        .include("../../../../cpp/perspective/dist/release/hopscotch-src/include")
        .include("../../../../cpp/perspective/dist/release/exprtk-src")
        .include("../../../../cpp/perspective/dist/release/re2-src")
        .include("../../../../cpp/perspective/dist/release/ordered-map-src/include")
        .cpp_set_stdlib(None)
        .opt_level(3)
        .flag("-D_WASI_EMULATED_MMAN")
        .flag("-D_WASI_EMULATED_SIGNAL")
        .flag("-DPSP_ENABLE_WASM")
        .flag_if_supported("-std=c++1y")
        .compile("bridge");

    println!("cargo:rerun-if-changed=src/ffi.rs");
    println!("cargo:rerun-if-changed=src/cpp_common.rs");
    println!("cargo:rerun-if-changed=cpp/types.h");
    println!("cargo:rerun-if-changed=cpp/ffi.h");
    println!("cargo:rerun-if-changed=cpp/ffi.cpp");

    // TODO: Automate this
    println!("cargo:rustc-link-search=native=/Users/timothybess/src/wasi-sdk/build/wasi-sdk-20.20g2393be41c8df/share/wasi-sysroot/lib/wasm32-wasi");
    //

    println!("cargo:rustc-link-search=native=../../cpp/perspective/dist/release");
    println!("cargo:rustc-link-search=native=../../cpp/perspective/dist/release/re2-build");
    println!("cargo:rustc-link-lib=psp");
    println!("cargo:rustc-link-lib=re2");
}

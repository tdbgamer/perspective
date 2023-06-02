load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

# http_archive(
#     name = "bazel_skylib",
#     sha256 = "66ffd9315665bfaafc96b52278f57c7e2dd09f5ede279ea6d39b2be471e7e3aa",
#     urls = [
#         "https://mirror.bazel.build/github.com/bazelbuild/bazel-skylib/releases/download/1.4.2/bazel-skylib-1.4.2.tar.gz",
#         "https://github.com/bazelbuild/bazel-skylib/releases/download/1.4.2/bazel-skylib-1.4.2.tar.gz",
#     ],
# )

# load("@bazel_skylib//:workspace.bzl", "bazel_skylib_workspace")

# bazel_skylib_workspace()

# ######################
# # rules_js setup #
# ######################

# http_archive(
#     name = "aspect_rules_js",
#     sha256 = "0b69e0967f8eb61de60801d6c8654843076bf7ef7512894a692a47f86e84a5c2",
#     strip_prefix = "rules_js-1.27.1",
#     url = "https://github.com/aspect-build/rules_js/releases/download/v1.27.1/rules_js-v1.27.1.tar.gz",
# )

# load("@aspect_rules_js//js:repositories.bzl", "rules_js_dependencies")

# rules_js_dependencies()

# load("@rules_nodejs//nodejs:repositories.bzl", "DEFAULT_NODE_VERSION", "nodejs_register_toolchains")

# nodejs_register_toolchains(
#     name = "nodejs",
#     node_version = DEFAULT_NODE_VERSION,
# )

# load("@aspect_rules_js//npm:repositories.bzl", "npm_translate_lock")

# npm_translate_lock(
#     name = "npm",
#     pnpm_lock = "//:pnpm-lock.yaml",
#     verify_node_modules_ignored = "//:.bazelignore",
#     yarn_lock = "//:yarn.lock",
# )

# load("@npm//:repositories.bzl", "npm_repositories")

# npm_repositories()

# #######################
# # rules_esbuild setup #
# #######################

# http_archive(
#     name = "aspect_rules_esbuild",
#     sha256 = "2ea31bd97181a315e048be693ddc2815fddda0f3a12ca7b7cc6e91e80f31bac7",
#     strip_prefix = "rules_esbuild-0.14.4",
#     url = "https://github.com/aspect-build/rules_esbuild/releases/download/v0.14.4/rules_esbuild-v0.14.4.tar.gz",
# )

# # Fetches the rules_esbuild dependencies.
# # If you want to have a different version of some dependency,
# # you should fetch it *before* calling this.
# # Alternatively, you can skip calling this function, so long as you've
# # already fetched all the dependencies.
# load("@aspect_rules_esbuild//esbuild:dependencies.bzl", "rules_esbuild_dependencies")

# rules_esbuild_dependencies()

# # Register a toolchain containing esbuild npm package and native bindings
# load("@aspect_rules_esbuild//esbuild:repositories.bzl", "LATEST_VERSION", "esbuild_register_toolchains")

# esbuild_register_toolchains(
#     name = "esbuild",
#     esbuild_version = LATEST_VERSION,
# )

#######################
# rules_rust setup
#######################

http_archive(
    name = "rules_rust",
    sha256 = "50272c39f20a3a3507cb56dcb5c3b348bda697a7d868708449e2fa6fb893444c",
    urls = ["https://github.com/bazelbuild/rules_rust/releases/download/0.22.0/rules_rust-v0.22.0.tar.gz"],
)

load("@rules_rust//rust:repositories.bzl", "rules_rust_dependencies", "rust_register_toolchains")

rules_rust_dependencies()

rust_register_toolchains(
    edition = "2021",
    extra_target_triples = [
        "wasm32-unknown-unknown",
        # "x86_64-unknown-linux-gnu",
        # "aarch64-unknown-linux-gnu",
    ],
    # versions = [ "1.69.0", "nightly/2023-04-07" ],
)

load("@rules_rust//crate_universe:defs.bzl", "crates_repository", "splicing_config", "crate")

crates_repository(
    name = "crate_index",
    cargo_lockfile = "//rust/perspective-viewer:Cargo.lock",
    isolated = False,
    # lockfile = "//:cargo-bazel.lock.json",
    annotations = {
        "web-sys": [crate.annotation(
            rustc_flags = ["--cfg=web_sys_unstable_apis"],
        )],
    },
    manifests = [
        "//rust/perspective-viewer:Cargo.toml",
        "//rust/perspective-viewer:tasks/bundle/Cargo.toml"
        # "//server:Cargo.toml",
    ],
    splicing_config = splicing_config(resolver_version = "2"),
)

load("@crate_index//:defs.bzl", "crate_repositories")

crate_repositories()

load("@rules_rust//wasm_bindgen:repositories.bzl", "rust_wasm_bindgen_dependencies", "rust_wasm_bindgen_register_toolchains")

rust_wasm_bindgen_dependencies()

rust_wasm_bindgen_register_toolchains()

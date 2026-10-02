To build lensisku for hosting on the main lojban.org infrastructure,
run ./build-rust.sh and ./build-npm.sh in this directory.

Rust builds default to nightly-2026-09-29 and the release-fast Cargo profile.
Override with RUST_TOOLCHAIN=1.98.1 CARGO_BUILD_PROFILE=release ./build-rust.sh
for stable Rust and maximum runtime optimization.
Compiled dependencies/incremental artifacts persist in ../target/container;
the deployment executable is copied to ../target/release/lensisku.
Remove ../target/container explicitly for a clean Rust build.

The hosting builder uses Debian Trixie to match the lensisku-containers runtime
and its ICU 76 libraries. LBCS runtime image rebuilds do not compile Rust;
run this helper in each deployed containers/web/src or containers/web-dev/src
checkout. The source bind mount preserves ../target/container between runs.

If things seem wonky, try: rm -rf ~/.local/npm-dotnpm ~/.local/rust-dotcargo

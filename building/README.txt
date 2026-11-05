To build lensisku for hosting on the main lojban.org infrastructure,
run ./build-rust.sh and ./build-npm.sh in this directory.

Frontend builds use a temporary output directory, then publish into frontend/dist.
New assets are installed before HTML; individual files are replaced by atomic rename.
Existing assets with the same name must have identical contents. Old files in dist
are retained so tabs opened before a deployment can still load their lazy chunks.
Do not delete dist as part of normal builds or rollback. After successful publication,
each deploy deletes assets retired for over 90 days. It records the current build's
assets and retirement timestamps in frontend/dist.asset-retention.json (version 1).
Still-used assets are protected regardless of file age. The first deployment gives
untracked legacy assets a 90-day grace period; missing metadata starts a new grace
period. Non-asset public files remain available. Failed builds do not run cleanup.
Concurrent publication is blocked by frontend/dist.publish-lock; after a killed
publisher, confirm it is no longer running before manually removing its stale lock.
This publishes files individually, not the whole release as one atomic operation.
Failed builds do not publish; a failure while publishing can leave a mix of HTML
generations; cleanup only runs after publication and metadata persistence succeed.
No nginx path changes are required. Very old tabs may need a refresh after cleanup.
Run the publisher regression checks with:
  node --test building/publish-frontend.test.mjs

Rust builds default to nightly-2026-09-29 and the release-fast Cargo profile.
The builder image installs mold (faster linking; configured in ../.cargo/config.toml).
On nightly, build-rust-inside.sh also sets -Zthreads=8 (parallel frontend).
Override with RUST_TOOLCHAIN=1.98.1 CARGO_BUILD_PROFILE=release ./build-rust.sh
for stable Rust and maximum runtime optimization (fat LTO; no -Z flags).
After Dockerfile.rust changes, remove the build-rust image so podman rebuilds it:
  podman rmi build-rust
Compiled dependencies/incremental artifacts persist in ../target/container;
the deployment executable is atomically replaced at ../target/release/lensisku.
The running server keeps its old executable; restart it after a successful build
to load the new version. Building does not restart the service.
Migrations are embedded in the executable. The root build.rs tracks the migrations
directory so adding, editing, removing, or renaming migrations rebuilds the app.
Remove ../target/container explicitly for a clean Rust build.

The hosting builder uses Debian Trixie to match the lensisku-containers runtime
and its ICU 76 libraries. LBCS runtime image rebuilds do not compile Rust;
run this helper in each deployed containers/web/src or containers/web-dev/src
checkout. The source bind mount preserves ../target/container between runs.
See archive/lensisku-containers/README-source-rebuild for the operator checklist.

If things seem wonky, try: rm -rf ~/.local/npm-dotnpm ~/.local/rust-dotcargo

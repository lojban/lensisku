# syntax=docker/dockerfile:1

# Keep the builder's libc compatible with the Bookworm runtime. The pinned
# nightly includes the September 2026 rustc/LLVM performance improvements.
FROM rust:1.98.1-bookworm AS backend-builder
ARG RUST_TOOLCHAIN=nightly-2026-09-29
RUN rustup toolchain install "${RUST_TOOLCHAIN}" --profile minimal && \
    rustup default "${RUST_TOOLCHAIN}"
WORKDIR /usr/src/app
# Only native development libraries are needed to compile Tectonic; the TeX
# installation and fonts belong in the runtime stage below.
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev libfontconfig1-dev libfreetype6-dev \
    libicu-dev libpng-dev zlib1g-dev \
    libgraphite2-dev \
    libharfbuzz-dev \
    && rm -rf /var/lib/apt/lists/*
# Set C++ standard to C++17 for dependencies that compile C++ code
ENV CXXFLAGS="-std=c++17"
# Faster recompiles with incremental compilation and no cross-crate fat LTO.
# Override with release when maximum runtime optimization is required.
ARG CARGO_BUILD_PROFILE=release-fast
ARG TARGETARCH

# Copy build inputs, then build with stub sources so this layer
# caches compiled dependencies. When only app code changes, only the final
# cargo build re-runs and recompiles the app (deps come from cache).
COPY Cargo.toml Cargo.lock build.rs ./
COPY .cargo ./.cargo
COPY migrations ./migrations
RUN mkdir -p src test && \
    echo 'fn main() {}' > src/main.rs && \
    echo 'fn main() {}' > test/test.rs

# Build dependencies (and stub binaries). Use BuildKit cache mounts so
# registry, git, and target are reused across builds (much faster rebuilds).
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,id=lensisku-target-${TARGETARCH}-${RUST_TOOLCHAIN}-${CARGO_BUILD_PROFILE},target=/usr/src/app/target,sharing=locked \
    cargo build --locked --bin lensisku --profile "${CARGO_BUILD_PROFILE}"

# Overwrite stubs with real source (only dirs needed for cargo build; excludes frontend, docs, scripts, etc.).
COPY src ./src
COPY test ./test
COPY locales ./locales
# Ensure Cargo replaces the cached stub even when COPY preserves old mtimes.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,id=lensisku-target-${TARGETARCH}-${RUST_TOOLCHAIN}-${CARGO_BUILD_PROFILE},target=/usr/src/app/target,sharing=locked \
    touch src/main.rs && \
    cargo build --locked --bin lensisku --profile "${CARGO_BUILD_PROFILE}" && \
    cp /usr/src/app/target/${CARGO_BUILD_PROFILE}/lensisku /usr/src/app/lensisku-out

# Build stage for Vue.js frontend
FROM node:24-alpine AS frontend-builder
WORKDIR /usr/src/app
# Copy package.json, lockfile, and pnpm allowBuilds (esbuild, vue-demi postinstall)
COPY frontend/package.json ./
COPY frontend/pnpm-lock.yaml ./
COPY frontend/pnpm-workspace.yaml ./
# Use Corepack with the packageManager pin (avoids Alpine ENOEXEC from pnpm self-switch)
ENV COREPACK_ENABLE_DOWNLOAD_PROMPT=0
RUN corepack enable && \
    corepack prepare "$(node -p "require('./package.json').packageManager")" --activate
# Install dependencies
RUN pnpm install --frozen-lockfile
# Copy the rest of the frontend code
COPY frontend .
# Build the frontend
RUN pnpm run build

# Final stage
FROM debian:bookworm-slim
WORKDIR /usr/src/app

# Install necessary dependencies
RUN apt-get update && apt-get install -y \
    texlive-xetex \
    texlive-fonts-recommended \
    texlive-fonts-extra \
    texlive-latex-extra \
    texlive-lang-chinese \
    texlive-lang-japanese \
    texlive-lang-other \
    fonts-noto-cjk fonts-noto-cjk-extra \
    fonts-noto-core fonts-noto-extra \
    fonts-linuxlibertine \
    libgraphite2-dev \
    libharfbuzz-dev \
    && rm -rf /var/lib/apt/lists/*


# Copy the built artifacts from the previous stages
COPY --from=backend-builder /usr/src/app/lensisku-out .
COPY --from=frontend-builder /usr/src/app/dist /var/www/html

# Scripts (e.g. import_valsi_sounds) and tools to run them
COPY scripts ./scripts
# libopus0: dynamic lib for Opus encoding (valsi TTS → audio/ogg); linked at build, loaded at runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
    nginx \
    python3 \
    python3-psycopg2 \
    libopus0 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy Nginx configuration
COPY nginx.conf /etc/nginx/nginx.conf

# Expose the port the app runs on
EXPOSE 80

# Start Nginx and the backend server
CMD ["sh", "-c", "service nginx start && exec ./lensisku"]

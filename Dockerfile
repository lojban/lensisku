# Build stage for Rust backend (pin version for reproducible, faster layer cache)
FROM rust:1-bookworm as backend-builder
WORKDIR /usr/src/app
# Install XeLaTeX and required fonts (single layer, no-recommends where safe)
RUN apt-get update && apt-get install -y --no-install-recommends \
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
ENV CXXFLAGS="-std=c++17"
# Use release-fast by default for faster image builds; override with build-arg for production.
ARG CARGO_BUILD_PROFILE=release-fast

# Copy only manifest and lockfile, then build with stub sources so this layer
# caches compiled dependencies. When only app code changes, only the final
# cargo build re-runs and recompiles the app (deps come from cache).
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src test && \
    echo 'fn main() {}' > src/main.rs && \
    echo 'fn main() {}' > test/test.rs

# Build dependencies (and stub binaries). Use BuildKit cache mounts so
# registry, git, and target are reused across builds (much faster rebuilds).
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/src/app/target \
    cargo build --release --profile ${CARGO_BUILD_PROFILE}

# Overwrite stubs with real source (only dirs needed for cargo build; excludes frontend, docs, scripts, etc.).
COPY src ./src
COPY test ./test
COPY migrations ./migrations
COPY .cargo ./.cargo
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/src/app/target \
    cargo build --release --profile ${CARGO_BUILD_PROFILE} && \
    cp /usr/src/app/target/${CARGO_BUILD_PROFILE}/lensisku /usr/src/app/lensisku-out

# Build stage for Vue.js frontend (pin version for reproducible builds)
FROM node:24-alpine AS frontend-builder
WORKDIR /usr/src/app
RUN apk add --no-cache curl && \
    corepack enable && corepack prepare pnpm@9 --activate
ENV PNPM_HOME="/root/.local/share/pnpm" PATH="/root/.local/share/pnpm:$PATH"
COPY frontend/package.json frontend/pnpm-lock.yaml ./
# Cache pnpm store across builds for much faster dependency installs
RUN --mount=type=cache,target=/root/.local/share/pnpm/store \
    pnpm install --frozen-lockfile
COPY frontend .
RUN pnpm run build

# Final stage (slim base, single apt layer for smaller image and cache)
FROM debian:bookworm-slim
WORKDIR /usr/src/app

RUN apt-get update && apt-get install -y --no-install-recommends \
    nginx-light \
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
    python3 \
    python3-psycopg2 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=backend-builder /usr/src/app/lensisku-out .
COPY --from=frontend-builder /usr/src/app/dist /var/www/html
COPY scripts ./scripts

# Copy Nginx configuration
COPY nginx.conf /etc/nginx/nginx.conf

# Expose the port the app runs on
EXPOSE 80

# Start Nginx and the backend server
CMD service nginx start && ./lensisku

# Build stage for Rust backend
FROM rust:latest as backend-builder
WORKDIR /usr/src/app
# Install XeLaTeX and required fonts
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
# Set C++ standard to C++17 for dependencies that compile C++ code
ENV CXXFLAGS="-std=c++17"
# Optional: set to release-fast for faster Docker builds (less optimized binary)
ARG CARGO_BUILD_PROFILE=release

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

# Build stage for Vue.js frontend
FROM node:24-alpine as frontend-builder
WORKDIR /usr/src/app
# Copy package.json and pnpm-lock.yaml
COPY frontend/package.json ./
COPY frontend/pnpm-lock.yaml ./
# Install pnpm using standalone installer (avoids npm registry network issues)
# Try corepack first, fallback to standalone installer if needed
RUN apk add curl && \
    (corepack enable && corepack prepare pnpm@latest --activate || \
     curl -fsSL https://get.pnpm.io/install.sh | sh -)
ENV PATH="/root/.local/share/pnpm:$PATH"
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
RUN apt-get update && apt-get install -y --no-install-recommends \
    python3 \
    python3-psycopg2 \
    && rm -rf /var/lib/apt/lists/*

# Copy Nginx configuration
COPY nginx.conf /etc/nginx/nginx.conf

# Expose the port the app runs on
EXPOSE 80

# Start Nginx and the backend server
CMD service nginx start && ./lensisku

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
COPY . .
# Set C++ standard to C++17 for dependencies that compile C++ code
# This is required for tectonic_xetex_layout which uses ICU headers
# that require C++17 features (auto template parameters)
ENV CXXFLAGS="-std=c++17"
RUN cargo build --release

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
COPY --from=backend-builder /usr/src/app/target/release/lensisku .
COPY --from=frontend-builder /usr/src/app/dist /var/www/html

# Copy Nginx configuration
COPY nginx.conf /etc/nginx/nginx.conf

# Expose the port the app runs on
EXPOSE 80

# Start Nginx and the backend server
CMD service nginx start && ./lensisku

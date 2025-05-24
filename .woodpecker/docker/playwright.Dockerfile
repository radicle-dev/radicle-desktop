FROM mcr.microsoft.com/playwright:v1.52.0-noble

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.84-x86_64-unknown-linux-gnu
ENV PATH="/root/.cargo/bin:${PATH}"

RUN apt-get update && apt-get install -y \
    build-essential \
    jq \
    curl \
    git \
    wget \
    file \
    zstd \
    libxdo-dev \
    libssl-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libwebkit2gtk-4.1-0=2.44.0-2 \
    libwebkit2gtk-4.1-dev=2.44.0-2 \
    libjavascriptcoregtk-4.1-0=2.44.0-2 \
    libjavascriptcoregtk-4.1-dev=2.44.0-2 \
    gir1.2-javascriptcoregtk-4.1=2.44.0-2 \
    gir1.2-webkit2-4.1=2.44.0-2

SHELL ["/bin/bash", "-c"]

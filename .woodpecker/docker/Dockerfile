FROM devraymondsh/ubuntu-rust:24.04-1.84

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

RUN rustup component add rustfmt clippy
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - && apt-get install -y nodejs

SHELL ["/bin/bash", "-c"]

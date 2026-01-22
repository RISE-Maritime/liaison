# Use the official Ubuntu 22.04 LTS image
FROM node:20

ARG CLAUDE_CODE_VERSION=latest

# Set environment variables
ENV DEBIAN_FRONTEND=noninteractive
ENV ZENOH_VERSION=1.3.3
ENV PROTOBUF_VERSION=3.20.1
ENV LIBZIP_VERSION=1.10.1
ENV SPDLOG_VERSION=v1.15.1

# Install required packages
RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    git \
    curl \
    wget \
    less \
    git \
    procps \
    sudo \
    fzf \
    zsh \
    man-db \
    unzip \
    gnupg2 \
    gh \
    iptables \
    ipset \
    iproute2 \
    dnsutils \
    aggregate \
    jq \
    nano \
    zlib1g-dev \
    libbz2-dev \
    liblzma-dev \
    libzstd-dev \
    libssl-dev \
    pkg-config \
    python3 \
    python3-pip \
    python3-venv \
    sudo \
    && apt-get clean && rm -rf /var/lib/apt/list


# Ensure default node user has access to /usr/local/share
RUN mkdir -p /usr/local/share/npm-global && \
  chown -R node:node /usr/local/share

# Set `DEVCONTAINER` environment variable to help with orientation
ENV DEVCONTAINER=true

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain 1.75.0 -y \
    && echo 'source $HOME/.cargo/env' >> ~/.bashrc

# Ensure the path is set for future commands in the Dockerfile
ENV PATH="/root/.cargo/bin:$PATH"

# Verify Rust and Cargo installation
RUN rustup --version && cargo --version


WORKDIR /workspace

# Build and install Zenoh-c
RUN git clone --depth 1 --branch ${ZENOH_VERSION} https://github.com/eclipse-zenoh/zenoh-c.git /workspace/zenoh-c
WORKDIR /workspace/zenoh-c/build
RUN cmake ../ -DCMAKE_INSTALL_PREFIX=/usr/local -DBUILD_SHARED_LIBS=FALSE && \
    cmake --build . --target install

# Build and install Zenoh-cpp
RUN git clone --depth 1 --branch ${ZENOH_VERSION} https://github.com/eclipse-zenoh/zenoh-cpp.git /workspace/zenoh-cpp
WORKDIR /workspace/zenoh-cpp/build
RUN cmake ../ -DCMAKE_INSTALL_PREFIX=/usr/local -DBUILD_SHARED_LIBS=FALSE && \
    cmake --build . --target install 

# Build and install Spdlog
RUN git clone --depth 1 --branch ${SPDLOG_VERSION} https://github.com/gabime/spdlog.git /workspace/spdlog
WORKDIR /workspace/spdlog/build
RUN cmake ../ -DCMAKE_INSTALL_PREFIX=/usr/local -DSPDLOG_BUILD_SHARED=FALSE && \
    cmake --build . --target install
    
# Download and build Protobuf from source
RUN curl -LO https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOBUF_VERSION}/protobuf-cpp-${PROTOBUF_VERSION}.tar.gz && \
    tar -xzf protobuf-cpp-${PROTOBUF_VERSION}.tar.gz && \
    cd protobuf-${PROTOBUF_VERSION} && \
    ./configure CXXFLAGS=-fPIC && \
    make -j$(nproc) && \
    make install && \
    ldconfig

# Download and build libzip from source
RUN wget https://libzip.org/download/libzip-${LIBZIP_VERSION}.tar.gz && \
    tar -xvf libzip-${LIBZIP_VERSION}.tar.gz && \
    cd libzip-${LIBZIP_VERSION} && \
    mkdir build && \
    cd build && \
    cmake .. \
        -DBUILD_SHARED_LIBS=OFF \          
        -DENABLE_GNUTLS=OFF \             
        -DENABLE_MBEDTLS=OFF \             
        -DENABLE_OPENSSL=OFF \              
        -DENABLE_BZIP2=OFF \               
        -DENABLE_LZMA=OFF \                
        -DENABLE_ZSTD=OFF \                
    && make -j$(nproc) \
    && make install
RUN rm -rf /libzip-${LIBZIP_VERSION} /libzip-${LIBZIP_VERSION}.tar.gz

# Set up non-root user
USER node

# Install global packages
ENV NPM_CONFIG_PREFIX=/usr/local/share/npm-global
ENV PATH=$PATH:/usr/local/share/npm-global/bin

# Set the default shell to zsh rather than sh
ENV SHELL=/bin/zsh

# Set the default editor and visual
ENV EDITOR=nano
ENV VISUAL=nano

# Install Claude
RUN curl -fsSL https://claude.ai/install.sh | bash

# Install uv and Python tools
RUN curl -LsSf https://astral.sh/uv/install.sh | sh && \
    /home/node/.local/bin/uv tool install black && \
    /home/node/.local/bin/uv tool install specify-cli --from git+https://github.com/github/spec-kit.git && \
    /home/node/.local/bin/uv venv /home/node/.venv && \
    /home/node/.local/bin/uv pip install --python /home/node/.venv/bin/python fmpy

# Add uv, Python tools, and venv to PATH
ENV PATH="/home/node/.venv/bin:/home/node/.cargo/bin:/home/node/.local/bin:$PATH"

# Copy and set up firewall script
COPY init-firewall.sh /usr/local/bin/
USER root
RUN chmod +x /usr/local/bin/init-firewall.sh && \
  echo "node ALL=(root) NOPASSWD: /usr/local/bin/init-firewall.sh" > /etc/sudoers.d/node-firewall && \
  chmod 0440 /etc/sudoers.d/node-firewall
USER node


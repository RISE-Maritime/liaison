FROM mcr.microsoft.com/devcontainers/cpp:ubuntu-22.04

# Set non-interactive installation mode
ARG DEBIAN_FRONTEND=noninteractive

# Update and install necessary packages
RUN echo "deb [trusted=yes] https://download.eclipse.org/zenoh/debian-repo/ /" | sudo tee -a /etc/apt/sources.list > /dev/null 
RUN apt-get update && apt-get install -y \
    build-essential \
    git \
    protobuf-compiler \
    libprotobuf-dev \
    cmake \
    zenoh \
    # Cleanup apt cache to reduce image size
    && rm -rf /var/lib/apt/lists/*

# Install Rust and Zenoh-C
# See: https://github.com/eclipse-zenoh/zenoh-c/issues/317#issuecomment-2052461738

ARG ZENOH_VERSION=0.10.1-rc
ARG ZENOHC_VERSION=${ZENOH_VERSION}
ARG RUST_VERSION=1.72
ARG RUST_PROFILE=minimal
ARG RUST_TOOLCHAIN=${RUST_VERSION}-x86_64-unknown-linux-gnu
ENV CARGO_HOME="/usr/local/cargo"
ENV RUSTUP_HOME="/usr/local/rustup"
ENV PATH "/usr/local/cargo/bin:/usr/local/rustup:${PATH}"

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --no-modify-path --profile ${RUST_PROFILE} --default-toolchain ${RUST_VERSION}

RUN git clone --depth 1 --branch ${ZENOHC_VERSION} https://github.com/eclipse-zenoh/zenoh-c.git /tmp/zenoh-c && \
    cmake -S/tmp/zenoh-c -B/tmp/zenoh-c/build \
        -DCMAKE_BUILD_TYPE=Release \
        -DZENOHC_CARGO_CHANNEL=${RUST_TOOLCHAIN} \
        -DZENOHC_INSTALL_STATIC_LIBRARY=TRUE \
        && \
    cmake --build /tmp/zenoh-c/build/ --target install -j$(nproc) && \
    rm -rf /tmp/zenoh-c

# Copy your project files into the Docker image
COPY . /workspaces

# Set the default command for the container
CMD ["bash"]



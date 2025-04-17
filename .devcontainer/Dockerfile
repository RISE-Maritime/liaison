# Use the official Ubuntu 22.04 LTS image
FROM ubuntu:22.04

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
    zlib1g-dev \
    libbz2-dev \
    liblzma-dev \
    libzstd-dev \
    libssl-dev \
    pkg-config \
    python3 \
    python3-pip \
    python3-venv \
    sudo
RUN rm -rf /var/lib/apt/lists/*


# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain 1.75.0 -y \
    && echo 'source $HOME/.cargo/env' >> ~/.bashrc

# Ensure the path is set for future commands in the Dockerfile
ENV PATH="/root/.cargo/bin:$PATH"

# Verify Rust and Cargo installation
RUN rustup --version && cargo --version

WORKDIR /workspaces

# Build and install Zenoh-c
RUN git clone --depth 1 --branch ${ZENOH_VERSION} https://github.com/eclipse-zenoh/zenoh-c.git /workspaces/zenoh-c
WORKDIR /workspaces/zenoh-c/build
RUN cmake ../ -DCMAKE_INSTALL_PREFIX=/usr/local -DBUILD_SHARED_LIBS=FALSE && \
    cmake --build . --target install

# Build and install Zenoh-cpp
RUN git clone --depth 1 --branch ${ZENOH_VERSION} https://github.com/eclipse-zenoh/zenoh-cpp.git /workspaces/zenoh-cpp
WORKDIR /workspaces/zenoh-cpp/build
RUN cmake ../ -DCMAKE_INSTALL_PREFIX=/usr/local -DBUILD_SHARED_LIBS=FALSE && \
    cmake --build . --target install 

# Build and install Spdlog
RUN git clone --depth 1 --branch ${SPDLOG_VERSION} https://github.com/gabime/spdlog.git /workspaces/spdlog
WORKDIR /workspaces/spdlog/build
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

# Python and FMPy
RUN python3 -m pip install fmpy[complete]==0.3.20

# Create a non-root user 
ENV USERNAME=developer
RUN useradd -m -s /bin/bash $USERNAME
RUN echo "$USERNAME ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers

USER ${USERNAME}

# Copy the workspace files to the container
COPY . /workspaces

CMD ["/bin/bash"]

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

# Set the working directory
WORKDIR /workspace

# Copy your project files into the Docker image
COPY . /workspace

# Set the default command for the container
CMD ["bash"]



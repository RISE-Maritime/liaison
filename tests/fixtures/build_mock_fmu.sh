#!/bin/bash
# Build script for Mock FMU
# This creates a minimal FMU structure for testing

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
BUILD_DIR="${SCRIPT_DIR}/build"
FMU_DIR="${BUILD_DIR}/MockFMU"

# Clean and create directories
rm -rf "${BUILD_DIR}"
mkdir -p "${FMU_DIR}/binaries/linux64"
mkdir -p "${FMU_DIR}/resources"

# Detect OS and architecture
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    PLATFORM="linux64"
    LIB_EXT="so"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    PLATFORM="darwin64"
    LIB_EXT="dylib"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
    PLATFORM="win64"
    LIB_EXT="dll"
else
    echo "Unsupported platform: $OSTYPE"
    exit 1
fi

# Update platform directory
mkdir -p "${FMU_DIR}/binaries/${PLATFORM}"

echo "Building Mock FMU for ${PLATFORM}..."

# Compile the shared library
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    gcc -shared -fPIC -o "${FMU_DIR}/binaries/${PLATFORM}/MockFMU.${LIB_EXT}" \
        "${SCRIPT_DIR}/mock_fmu.c"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    clang -shared -fPIC -o "${FMU_DIR}/binaries/${PLATFORM}/MockFMU.${LIB_EXT}" \
        "${SCRIPT_DIR}/mock_fmu.c"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
    gcc -shared -o "${FMU_DIR}/binaries/${PLATFORM}/MockFMU.${LIB_EXT}" \
        "${SCRIPT_DIR}/mock_fmu.c"
fi

# Copy model description
cp "${SCRIPT_DIR}/modelDescription.xml" "${FMU_DIR}/"

# Create the FMU archive
cd "${BUILD_DIR}"
zip -r MockFMU.fmu MockFMU/

echo "Mock FMU built successfully: ${BUILD_DIR}/MockFMU.fmu"
echo "Shared library: ${FMU_DIR}/binaries/${PLATFORM}/MockFMU.${LIB_EXT}"

# List contents
echo ""
echo "FMU Contents:"
unzip -l MockFMU.fmu

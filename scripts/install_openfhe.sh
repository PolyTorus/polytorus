#!/bin/bash

# OpenFHE Installation and Verification Script for PolyTorus
# This script installs the MachinaIO fork of OpenFHE for Diamond IO integration

set -e  # Exit on any error

echo "🔧 OpenFHE Installation Script for PolyTorus"
echo "============================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
OPENFHE_REPO="https://github.com/MachinaIO/openfhe-development.git"
OPENFHE_BRANCH="feat/improve_determinant"
INSTALL_PREFIX="/usr/local"
BUILD_DIR="/tmp/openfhe-build"

echo -e "${BLUE}📋 Configuration:${NC}"
echo "  Repository: $OPENFHE_REPO"
echo "  Branch: $OPENFHE_BRANCH"
echo "  Install prefix: $INSTALL_PREFIX"
echo ""

# Check if running with sudo for system installation
if [ "$INSTALL_PREFIX" = "/usr/local" ] && [ "$EUID" -ne 0 ]; then
    echo -e "${YELLOW}⚠️  Warning: Installing to /usr/local requires sudo privileges${NC}"
    echo "Please run with sudo or install to a user directory"
    exit 1
fi

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check dependencies
echo -e "${BLUE}🔍 Checking dependencies...${NC}"

# Required tools
REQUIRED_TOOLS=("git" "cmake" "make" "gcc" "g++")
for tool in "${REQUIRED_TOOLS[@]}"; do
    if ! command_exists "$tool"; then
        echo -e "${RED}❌ $tool is not installed${NC}"
        exit 1
    else
        echo -e "${GREEN}✅ $tool${NC}"
    fi
done

# Check for required libraries
echo -e "${BLUE}🔍 Checking system libraries...${NC}"

# Function to check library
check_lib() {
    if ldconfig -p | grep -q "$1"; then
        echo -e "${GREEN}✅ $1${NC}"
    else
        echo -e "${YELLOW}⚠️  $1 not found (may need: apt-get install $2)${NC}"
    fi
}

check_lib "libgmp" "libgmp-dev"
check_lib "libntl" "libntl-dev"
check_lib "libboost" "libboost-all-dev"

# Clean up previous build
if [ -d "$BUILD_DIR" ]; then
    echo -e "${YELLOW}🧹 Cleaning up previous build...${NC}"
    rm -rf "$BUILD_DIR"
fi

# Clone OpenFHE
echo -e "${BLUE}📥 Cloning OpenFHE...${NC}"
git clone "$OPENFHE_REPO" "$BUILD_DIR"
cd "$BUILD_DIR"
git checkout "$OPENFHE_BRANCH"

# Get commit info
COMMIT_HASH=$(git rev-parse --short HEAD)
echo -e "${GREEN}📌 Using commit: $COMMIT_HASH${NC}"

# Create build directory
mkdir -p build
cd build

# Configure with CMake
echo -e "${BLUE}⚙️  Configuring build...${NC}"
cmake -DCMAKE_INSTALL_PREFIX="$INSTALL_PREFIX" \
      -DCMAKE_BUILD_TYPE=Release \
      -DBUILD_UNITTESTS=OFF \
      -DBUILD_EXAMPLES=OFF \
      -DBUILD_BENCHMARKS=OFF \
      -DWITH_OPENMP=ON \
      -DCMAKE_CXX_STANDARD=17 \
      ..

# Build
echo -e "${BLUE}🔨 Building OpenFHE (this may take a while)...${NC}"
NPROC=$(nproc 2>/dev/null || echo 4)
echo "Using $NPROC parallel jobs"
make -j"$NPROC"

# Install
echo -e "${BLUE}📦 Installing OpenFHE...${NC}"
make install

# Update library cache
echo -e "${BLUE}🔄 Updating library cache...${NC}"
if [ "$INSTALL_PREFIX" = "/usr/local" ]; then
    ldconfig
fi

# Verification
echo -e "${BLUE}🧪 Verifying installation...${NC}"

# Check if libraries exist
LIBS=("libOPENFHEcore" "libOPENFHEpke" "libOPENFHEbinfhe")
for lib in "${LIBS[@]}"; do
    if ls "$INSTALL_PREFIX/lib/${lib}"* >/dev/null 2>&1; then
        echo -e "${GREEN}✅ $lib${NC}"
    else
        echo -e "${RED}❌ $lib not found${NC}"
        exit 1
    fi
done

# Check if headers exist
if [ -d "$INSTALL_PREFIX/include/openfhe" ]; then
    echo -e "${GREEN}✅ Headers installed${NC}"
else
    echo -e "${RED}❌ Headers not found${NC}"
    exit 1
fi

# Check pkg-config
if [ -f "$INSTALL_PREFIX/lib/pkgconfig/openfhe.pc" ]; then
    echo -e "${GREEN}✅ pkg-config file${NC}"
else
    echo -e "${YELLOW}⚠️  pkg-config file not found${NC}"
fi

# Clean up build directory
echo -e "${BLUE}🧹 Cleaning up...${NC}"
cd /
rm -rf "$BUILD_DIR"

# Environment setup
echo -e "${BLUE}🌍 Environment setup:${NC}"
echo "export OPENFHE_ROOT=$INSTALL_PREFIX"
echo "export LD_LIBRARY_PATH=$INSTALL_PREFIX/lib:\$LD_LIBRARY_PATH"
echo "export PKG_CONFIG_PATH=$INSTALL_PREFIX/lib/pkgconfig:\$PKG_CONFIG_PATH"

echo ""
echo -e "${GREEN}🎉 OpenFHE installation completed successfully!${NC}"
echo ""
echo -e "${BLUE}📋 Installation summary:${NC}"
echo "  Install path: $INSTALL_PREFIX"
echo "  Commit: $COMMIT_HASH"
echo "  Libraries: $INSTALL_PREFIX/lib/libOPENFHE*"
echo "  Headers: $INSTALL_PREFIX/include/openfhe/"
echo ""
echo -e "${YELLOW}💡 Next steps:${NC}"
echo "1. Add the environment variables above to your shell profile"
echo "2. Run 'cargo build' to build PolyTorus with OpenFHE support"
echo "3. Run 'cargo test diamond' to test Diamond IO integration"

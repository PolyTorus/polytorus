# PolyTorus - Post-Quantum Blockchain Platform
# Multi-stage Docker build with OpenFHE support

FROM ubuntu:22.04 AS openfhe-builder

# Install system dependencies for OpenFHE
RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    git \
    pkg-config \
    libssl-dev \
    autoconf \
    automake \
    libtool \
    libgmp-dev \
    libntl-dev \
    libboost-all-dev \
    libgmp3-dev \
    libmpfr-dev \
    libfftw3-dev \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Clone and build OpenFHE
WORKDIR /tmp
RUN git clone https://github.com/MachinaIO/openfhe-development.git && \
    cd openfhe-development && \
    git checkout feat/improve_determinant && \
    mkdir build && \
    cd build && \
    cmake -DCMAKE_INSTALL_PREFIX=/usr/local \
          -DCMAKE_BUILD_TYPE=Release \
          -DBUILD_UNITTESTS=OFF \
          -DBUILD_EXAMPLES=OFF \
          -DBUILD_BENCHMARKS=OFF \
          -DWITH_OPENMP=ON \
          -DCMAKE_CXX_STANDARD=17 \
          -DCMAKE_CXX_FLAGS="-O2 -DNDEBUG -Wno-unused-parameter -Wno-unused-function -Wno-missing-field-initializers" \
          .. && \
    make -j$(nproc) && \
    make install && \
    mkdir -p /usr/local/lib/pkgconfig && \
    ldconfig

# Runtime stage
FROM ubuntu:22.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    libgmp10 \
    libntl-dev \
    libboost-system-dev \
    libboost-filesystem-dev \
    libmpfr6 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy OpenFHE libraries from builder stage
COPY --from=openfhe-builder /usr/local/lib/libOPENFHE* /usr/local/lib/
COPY --from=openfhe-builder /usr/local/include/openfhe/ /usr/local/include/openfhe/
COPY --from=openfhe-builder /usr/local/lib/pkgconfig/ /usr/local/lib/pkgconfig/

# Update library cache
RUN ldconfig

# Install Rust nightly
RUN apt-get update && apt-get install -y curl && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly-2025-01-01 && \
    rm -rf /var/lib/apt/lists/*

ENV PATH="/root/.cargo/bin:${PATH}"
ENV LD_LIBRARY_PATH="/usr/local/lib"
ENV PKG_CONFIG_PATH="/usr/local/lib/pkgconfig"
ENV OPENFHE_ROOT="/usr/local"
ENV CXXFLAGS="-std=c++17 -O2 -DNDEBUG -Wno-unused-parameter -Wno-unused-function -Wno-missing-field-initializers"
ENV CXX_FLAGS="-std=c++17 -O2 -DNDEBUG -Wno-unused-parameter -Wno-unused-function -Wno-missing-field-initializers"

# Create app directory
WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./

# Create dummy source to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo 'pub fn add(left: usize, right: usize) -> usize { left + right }' > src/lib.rs

# Build dependencies (cached layer)
RUN cargo build --release && \
    rm -rf src

# Copy source code
COPY src/ ./src/
COPY examples/ ./examples/
COPY tests/ ./tests/
COPY config/ ./config/
COPY contracts/ ./contracts/

# Build the application
RUN cargo build --release

# Verify OpenFHE integration
RUN cargo test diamond --release

# Create non-root user
RUN useradd -m -u 1000 polytorus
USER polytorus

# Set the startup command
EXPOSE 8080
CMD ["./target/release/polytorus"]

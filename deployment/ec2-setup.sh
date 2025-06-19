#!/bin/bash

# PolyTorus EC2 Deployment Script
# This script sets up a PolyTorus testnet node on an EC2 instance

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}PolyTorus EC2 Testnet Setup${NC}"
echo "=================================="

# Check if running as root
if [[ $EUID -eq 0 ]]; then
   echo -e "${RED}This script should not be run as root${NC}"
   exit 1
fi

# Update system
echo -e "${YELLOW}Updating system packages...${NC}"
sudo apt-get update && sudo apt-get upgrade -y

# Install system dependencies
echo -e "${YELLOW}Installing system dependencies...${NC}"
sudo apt-get install -y \
    curl \
    git \
    build-essential \
    cmake \
    libgmp-dev \
    libntl-dev \
    libboost-all-dev \
    pkg-config \
    htop \
    ufw

# Install Rust
echo -e "${YELLOW}Installing Rust...${NC}"
if ! command -v rustc &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    rustup default nightly
fi

# Install Docker
echo -e "${YELLOW}Installing Docker...${NC}"
if ! command -v docker &> /dev/null; then
    curl -fsSL https://get.docker.com -o get-docker.sh
    sudo sh get-docker.sh
    sudo usermod -aG docker $USER
    rm get-docker.sh
fi

# Install Docker Compose
echo -e "${YELLOW}Installing Docker Compose...${NC}"
if ! command -v docker-compose &> /dev/null; then
    sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
    sudo chmod +x /usr/local/bin/docker-compose
fi

# Clone PolyTorus repository
echo -e "${YELLOW}Cloning PolyTorus repository...${NC}"
if [ ! -d "polytorus" ]; then
    git clone https://github.com/PolyTorus/polytorus.git
fi
cd polytorus

# Install OpenFHE
echo -e "${YELLOW}Installing OpenFHE...${NC}"
if [ ! -d "/usr/local/include/openfhe" ]; then
    sudo ./scripts/install_openfhe.sh
fi

# Set environment variables
echo -e "${YELLOW}Setting up environment...${NC}"
echo 'export OPENFHE_ROOT=/usr/local' >> ~/.bashrc
echo 'export LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH' >> ~/.bashrc
echo 'export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH' >> ~/.bashrc
source ~/.bashrc

# Build PolyTorus
echo -e "${YELLOW}Building PolyTorus...${NC}"
export OPENFHE_ROOT=/usr/local
export LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH
export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH
cargo build --release

# Setup firewall
echo -e "${YELLOW}Configuring firewall...${NC}"
sudo ufw allow ssh
sudo ufw allow 8000/tcp   # P2P
sudo ufw allow 8080/tcp   # HTTP API
sudo ufw allow 8545/tcp   # RPC
sudo ufw allow 8900/tcp   # Discovery
echo "y" | sudo ufw enable

# Create directories
echo -e "${YELLOW}Creating data directories...${NC}"
mkdir -p ~/polytorus-data ~/polytorus-logs

# Copy configuration
echo -e "${YELLOW}Setting up configuration...${NC}"
cp ec2-config/ec2-testnet.toml ~/polytorus-testnet.toml

# Get public IP and update configuration
PUBLIC_IP=$(curl -s https://ipinfo.io/ip)
echo -e "${GREEN}Instance public IP: ${PUBLIC_IP}${NC}"

# Create systemd service
echo -e "${YELLOW}Creating systemd service...${NC}"
sudo tee /etc/systemd/system/polytorus.service > /dev/null <<EOF
[Unit]
Description=PolyTorus Blockchain Node
After=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=/home/$USER/polytorus
Environment=OPENFHE_ROOT=/usr/local
Environment=LD_LIBRARY_PATH=/usr/local/lib
Environment=PKG_CONFIG_PATH=/usr/local/lib/pkgconfig
Environment=RUST_LOG=info
ExecStart=/home/$USER/polytorus/target/release/polytorus --modular-start --config /home/$USER/polytorus-testnet.toml --http-port 8080 --data-dir /home/$USER/polytorus-data
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# Enable service
sudo systemctl daemon-reload
sudo systemctl enable polytorus

echo -e "${GREEN}Setup completed!${NC}"
echo ""
echo "To start the node:"
echo "  sudo systemctl start polytorus"
echo ""
echo "To check status:"
echo "  sudo systemctl status polytorus"
echo ""
echo "To view logs:"
echo "  sudo journalctl -u polytorus -f"
echo ""
echo "Node will be available at:"
echo "  HTTP API: http://${PUBLIC_IP}:8080"
echo "  RPC: http://${PUBLIC_IP}:8545"
echo "  P2P: ${PUBLIC_IP}:8000"
echo ""
echo -e "${YELLOW}Remember to update bootstrap nodes in the configuration with other node IPs!${NC}"
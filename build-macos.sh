#!/bin/bash
# build-macos.sh - Build script for Skyscope Solana MEV Bot on macOS
# Creates a universal binary (x86_64 + arm64) and DMG installer

set -e # Exit on error

# Configuration
APP_NAME="Skyscope Solana MEV Bot"
APP_IDENTIFIER="ai.skyscope.mevbot"
APP_VERSION="1.0.0"
NODE_VERSION="18"
RUST_VERSION="1.76.0"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Print section header
section() {
  echo -e "\n${BLUE}==>${NC} $1"
}

# Print success message
success() {
  echo -e "${GREEN}✓${NC} $1"
}

# Print error message and exit
error() {
  echo -e "${RED}Error:${NC} $1"
  exit 1
}

# Check if running on macOS
if [[ "$(uname)" != "Darwin" ]]; then
  error "This script must be run on macOS"
fi

# Create build directory
BUILD_DIR="$(pwd)/build"
mkdir -p "$BUILD_DIR"

section "Checking system requirements"

# Check for Homebrew
if ! command -v brew &> /dev/null; then
  echo "Installing Homebrew..."
  /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)" || error "Failed to install Homebrew"
fi

# Check for Rust
if ! command -v rustup &> /dev/null; then
  echo "Installing Rust..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain $RUST_VERSION || error "Failed to install Rust"
  source "$HOME/.cargo/env"
else
  echo "Updating Rust..."
  rustup update || error "Failed to update Rust"
fi

# Check for Node.js (use nvm to avoid compilation issues)
if ! command -v nvm &> /dev/null; then
  echo "Installing nvm (Node Version Manager)..."
  curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash || error "Failed to install nvm"
  
  # Source nvm
  export NVM_DIR="$HOME/.nvm"
  [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
fi

# Install Node.js using nvm
if command -v nvm &> /dev/null; then
  echo "Installing Node.js $NODE_VERSION using nvm..."
  nvm install $NODE_VERSION || error "Failed to install Node.js"
  nvm use $NODE_VERSION || error "Failed to use Node.js $NODE_VERSION"
else
  # Fallback to Homebrew if nvm is not available
  echo "Installing Node.js using Homebrew..."
  brew install node@$NODE_VERSION || error "Failed to install Node.js"
  export PATH="/usr/local/opt/node@$NODE_VERSION/bin:$PATH"
fi

# Install Tauri CLI
section "Installing Tauri CLI"
cargo install tauri-cli --version "^1.5.0" || error "Failed to install Tauri CLI"
success "Tauri CLI installed"

# Install frontend dependencies
section "Installing frontend dependencies"
if [ ! -d "frontend" ]; then
  error "Frontend directory not found. Make sure you're in the project root."
fi

# Create node_modules directory if it doesn't exist
mkdir -p frontend/node_modules

cd frontend
npm install || error "Failed to install frontend dependencies"
cd ..
success "Frontend dependencies installed"

# Build frontend
section "Building frontend"
cd frontend
npm run build || error "Failed to build frontend"
cd ..
success "Frontend built"

# Build Tauri app
section "Building Tauri app for macOS (universal binary)"
# Set environment variables for universal binary
export MACOSX_DEPLOYMENT_TARGET=10.15
export CARGO_BUILD_TARGET="x86_64-apple-darwin aarch64-apple-darwin"

# Build the app
cargo tauri build --target universal-apple-darwin || error "Failed to build Tauri app"
success "Tauri app built"

# Check for code signing certificate
IDENTITY=$(security find-identity -v -p codesigning | grep "Developer ID Application" | head -1 | cut -d '"' -f 2 || echo "")

if [ -n "$IDENTITY" ]; then
  section "Code signing app with identity: $IDENTITY"
  
  # Sign the app
  codesign --force --deep --sign "$IDENTITY" "target/universal-apple-darwin/release/bundle/macos/$APP_NAME.app" || error "Failed to sign app"
  success "App signed"
  
  # Create DMG with create-dmg if available
  if command -v create-dmg &> /dev/null; then
    section "Creating DMG installer"
    
    # Create DMG
    create-dmg \
      --volname "$APP_NAME" \
      --volicon "src-tauri/icons/icon.icns" \
      --window-pos 200 120 \
      --window-size 800 400 \
      --icon-size 100 \
      --icon "$APP_NAME.app" 200 190 \
      --hide-extension "$APP_NAME.app" \
      --app-drop-link 600 185 \
      "$BUILD_DIR/$APP_NAME-$APP_VERSION.dmg" \
      "target/universal-apple-darwin/release/bundle/macos/$APP_NAME.app" || error "Failed to create DMG"
    
    # Sign the DMG
    codesign --force --sign "$IDENTITY" "$BUILD_DIR/$APP_NAME-$APP_VERSION.dmg" || error "Failed to sign DMG"
    success "DMG installer created and signed"
  else
    echo "create-dmg not found, skipping DMG creation"
    echo "To install: brew install create-dmg"
    
    # Copy the app to build directory
    cp -r "target/universal-apple-darwin/release/bundle/macos/$APP_NAME.app" "$BUILD_DIR/"
  fi
else
  echo "No code signing certificate found, skipping code signing"
  
  # Copy the app to build directory
  cp -r "target/universal-apple-darwin/release/bundle/macos/$APP_NAME.app" "$BUILD_DIR/"
fi

section "Build completed successfully!"
echo "App bundle: $BUILD_DIR/$APP_NAME.app"
if [ -f "$BUILD_DIR/$APP_NAME-$APP_VERSION.dmg" ]; then
  echo "DMG installer: $BUILD_DIR/$APP_NAME-$APP_VERSION.dmg"
fi

echo -e "\n${GREEN}Skyscope Solana MEV Bot has been built successfully!${NC}"
echo "To run the app, open: $BUILD_DIR/$APP_NAME.app"

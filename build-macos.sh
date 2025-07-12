#!/bin/bash
# =============================================================================
# Skyscope Solana MEV Bot - macOS Build Script
# =============================================================================
# This script automates the build process for creating a universal macOS app
# with a dark-themed GUI for the Skyscope Solana MEV Bot.
# 
# Features:
# - Checks and installs required dependencies
# - Builds the React frontend with dark theme
# - Compiles the Rust backend
# - Packages everything into a universal macOS app bundle
# - Handles code signing if certificates are available
# =============================================================================

# Set strict error handling
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# App information
APP_NAME="Skyscope Solana MEV Bot"
APP_VERSION="1.0.0"
APP_BUNDLE_ID="io.skyscope.solanamevbot"
APP_COPYRIGHT="© $(date +%Y) Skyscope Sentinel Intelligence"

# Directories
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
ROOT_DIR="$SCRIPT_DIR"
FRONTEND_DIR="$ROOT_DIR/frontend"
DIST_DIR="$ROOT_DIR/dist"
BUILD_DIR="$ROOT_DIR/target"
MACOS_BUILD_DIR="$BUILD_DIR/bundle/macos"

# Log file
LOG_FILE="$ROOT_DIR/build-macos.log"

# Function to log messages
log() {
  local level=$1
  local message=$2
  local color=$NC
  
  case $level in
    "INFO") color=$GREEN ;;
    "WARN") color=$YELLOW ;;
    "ERROR") color=$RED ;;
    "STEP") color=$BLUE ;;
    "SUCCESS") color=$PURPLE ;;
  esac
  
  echo -e "${color}[$level] $message${NC}"
  echo "[$level] $message" >> "$LOG_FILE"
}

# Function to show progress
show_progress() {
  local message=$1
  local progress=$2
  local width=50
  local filled=$((progress * width / 100))
  local empty=$((width - filled))
  
  printf "\r${CYAN}[%-${width}s] %d%% ${message}${NC}" "$(printf '#%.0s' $(seq 1 $filled))" "$progress"
}

# Function to check if a command exists
command_exists() {
  command -v "$1" &> /dev/null
}

# Function to check and install dependencies
check_dependencies() {
  log "STEP" "Checking dependencies..."
  
  # Check for Homebrew
  if ! command_exists brew; then
    log "INFO" "Installing Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
  else
    log "INFO" "Homebrew is already installed."
  fi
  
  # Check for Rust
  if ! command_exists rustc; then
    log "INFO" "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
  else
    log "INFO" "Rust is already installed. Updating..."
    rustup update
  fi
  
  # Check for Tauri CLI
  if ! command_exists cargo-tauri; then
    log "INFO" "Installing Tauri CLI..."
    cargo install tauri-cli
  else
    log "INFO" "Tauri CLI is already installed."
  fi
  
  # Check for additional build dependencies
  log "INFO" "Installing additional build dependencies..."
  brew install cmake pkg-config openssl@3
  
  # Install frontend dependencies
  if [ ! -d "$FRONTEND_DIR/node_modules" ]; then
    log "INFO" "Installing frontend dependencies..."
    cd "$FRONTEND_DIR" || exit 1
    npm install
    cd "$ROOT_DIR" || exit 1
  fi
  
  log "SUCCESS" "All dependencies are installed."
}

# Function to setup the project structure if it doesn't exist
setup_project_structure() {
  log "STEP" "Setting up project structure..."
  
  # Create directories if they don't exist
  mkdir -p "$DIST_DIR"
  mkdir -p "$FRONTEND_DIR"
  
  # Check if we need to initialize Tauri project
  if [ ! -f "$ROOT_DIR/tauri.conf.json" ]; then
    log "INFO" "Initializing Tauri project..."
    cargo tauri init --app-name "Skyscope Solana MEV Bot" \
      --window-title "Skyscope Solana MEV Bot" \
      --dist-dir "../dist" \
      --before-dev-command "npm run dev" \
      --before-build-command "npm run build" \
      --dev-path "http://localhost:3000"
  fi
  
  # Check if we need to create React app
  if [ ! -f "$FRONTEND_DIR/package.json" ]; then
    log "INFO" "Creating React app..."
    npx create-react-app "$FRONTEND_DIR" --template typescript
    
    # Add required dependencies for the project
    cd "$FRONTEND_DIR" || exit 1
    npm install @tauri-apps/api @tauri-apps/cli styled-components chart.js react-chartjs-2 react-qr-code
    npm install -D @types/styled-components
    cd "$ROOT_DIR" || exit 1
  fi
  
  log "SUCCESS" "Project structure set up successfully."
}

# Function to build the React frontend
build_frontend() {
  log "STEP" "Building React frontend..."
  
  cd "$FRONTEND_DIR" || exit 1
  
  # Update theme to dark mode in index.css
  cat > src/index.css << EOL
:root {
  --background-color: #121212;
  --surface-color: #1e1e1e;
  --primary-color: #bb86fc;
  --secondary-color: #03dac6;
  --error-color: #cf6679;
  --on-background-color: #ffffff;
  --on-surface-color: #ffffff;
  --on-primary-color: #000000;
  --on-secondary-color: #000000;
  --on-error-color: #000000;
  --border-radius: 8px;
}

body {
  margin: 0;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen',
    'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue',
    sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  background-color: var(--background-color);
  color: var(--on-background-color);
}

code {
  font-family: source-code-pro, Menlo, Monaco, Consolas, 'Courier New',
    monospace;
}

button, input, select, textarea {
  border-radius: var(--border-radius);
  padding: 8px 16px;
  border: none;
  background-color: var(--surface-color);
  color: var(--on-surface-color);
  font-size: 14px;
}

button {
  background-color: var(--primary-color);
  color: var(--on-primary-color);
  cursor: pointer;
  transition: background-color 0.3s;
}

button:hover {
  opacity: 0.9;
}

.progress-bar {
  width: 100%;
  height: 8px;
  background-color: var(--surface-color);
  border-radius: var(--border-radius);
  overflow: hidden;
  margin: 10px 0;
}

.progress-bar-fill {
  height: 100%;
  background-color: var(--primary-color);
  transition: width 0.3s ease;
}
EOL
  
  # Build the frontend
  log "INFO" "Running npm build..."
  npm run build
  
  # Copy build to dist directory
  rm -rf "$DIST_DIR"
  mkdir -p "$DIST_DIR"
  cp -r build/* "$DIST_DIR/"
  
  cd "$ROOT_DIR" || exit 1
  log "SUCCESS" "Frontend built successfully."
}

# Function to build the Rust backend
build_backend() {
  log "STEP" "Building Rust backend..."
  
  # Make sure we're using the latest Rust
  rustup update
  
  # Build the backend in release mode
  cargo build --release
  
  log "SUCCESS" "Backend built successfully."
}

# Function to build the Tauri app
build_tauri_app() {
  log "STEP" "Building Tauri app..."
  
  # Build the Tauri app
  cargo tauri build
  
  log "SUCCESS" "Tauri app built successfully."
}

# Function to create a universal binary
create_universal_binary() {
  log "STEP" "Creating universal binary..."
  
  # Check if we have both Intel and Apple Silicon builds
  if [ -d "$BUILD_DIR/x86_64-apple-darwin/release" ] && [ -d "$BUILD_DIR/aarch64-apple-darwin/release" ]; then
    log "INFO" "Creating universal binary from Intel and Apple Silicon builds..."
    
    # Create universal binary using lipo
    lipo -create -output "$BUILD_DIR/universal/release/skyscope-solana-mev-bot" \
      "$BUILD_DIR/x86_64-apple-darwin/release/skyscope-solana-mev-bot" \
      "$BUILD_DIR/aarch64-apple-darwin/release/skyscope-solana-mev-bot"
  else
    log "WARN" "Both Intel and Apple Silicon builds not found. Creating architecture-specific build only."
  fi
  
  log "SUCCESS" "Universal binary created successfully."
}

# Function to package the app
package_app() {
  log "STEP" "Packaging macOS app..."
  
  # Create app bundle structure
  mkdir -p "$MACOS_BUILD_DIR/$APP_NAME.app/Contents/MacOS"
  mkdir -p "$MACOS_BUILD_DIR/$APP_NAME.app/Contents/Resources"
  
  # Copy binary
  if [ -f "$BUILD_DIR/universal/release/skyscope-solana-mev-bot" ]; then
    cp "$BUILD_DIR/universal/release/skyscope-solana-mev-bot" "$MACOS_BUILD_DIR/$APP_NAME.app/Contents/MacOS/"
  else
    cp "$BUILD_DIR/release/skyscope-solana-mev-bot" "$MACOS_BUILD_DIR/$APP_NAME.app/Contents/MacOS/"
  fi
  
  # Create Info.plist
  cat > "$MACOS_BUILD_DIR/$APP_NAME.app/Contents/Info.plist" << EOL
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>skyscope-solana-mev-bot</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon.icns</string>
    <key>CFBundleIdentifier</key>
    <string>${APP_BUNDLE_ID}</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>${APP_VERSION}</string>
    <key>CFBundleVersion</key>
    <string>${APP_VERSION}</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSHumanReadableCopyright</key>
    <string>${APP_COPYRIGHT}</string>
</dict>
</plist>
EOL
  
  # Create default icon if not exists
  if [ ! -f "$ROOT_DIR/assets/AppIcon.icns" ]; then
    log "WARN" "No app icon found. Using default icon."
    # This would ideally create a default icon, but for now we'll skip
  else
    cp "$ROOT_DIR/assets/AppIcon.icns" "$MACOS_BUILD_DIR/$APP_NAME.app/Contents/Resources/"
  fi
  
  log "SUCCESS" "App packaged successfully."
}

# Function to sign the app
sign_app() {
  log "STEP" "Checking for code signing..."
  
  # Check if we have a valid signing identity
  SIGNING_IDENTITY=$(security find-identity -v -p codesigning | grep -o '"[^"]*"' | head -1 | tr -d '"')
  
  if [ -n "$SIGNING_IDENTITY" ]; then
    log "INFO" "Found signing identity: $SIGNING_IDENTITY"
    log "INFO" "Signing app bundle..."
    
    # Sign the app
    codesign --force --deep --sign "$SIGNING_IDENTITY" "$MACOS_BUILD_DIR/$APP_NAME.app"
    
    # Verify signature
    codesign --verify --verbose "$MACOS_BUILD_DIR/$APP_NAME.app"
    
    log "SUCCESS" "App signed successfully."
  else
    log "WARN" "No valid signing identity found. App will not be signed."
  fi
}

# Function to create DMG installer
create_dmg() {
  log "STEP" "Creating DMG installer..."
  
  # Check if create-dmg is installed
  if ! command_exists create-dmg; then
    log "INFO" "Installing create-dmg..."
    brew install create-dmg
  fi
  
  # Create DMG
  DMG_FILE="$DIST_DIR/${APP_NAME// /-}-$APP_VERSION.dmg"
  
  create-dmg \
    --volname "$APP_NAME" \
    --volicon "$ROOT_DIR/assets/AppIcon.icns" \
    --window-pos 200 120 \
    --window-size 800 400 \
    --icon-size 100 \
    --icon "$APP_NAME.app" 200 190 \
    --hide-extension "$APP_NAME.app" \
    --app-drop-link 600 185 \
    "$DMG_FILE" \
    "$MACOS_BUILD_DIR/" \
    || log "WARN" "DMG creation failed. Continuing without DMG."
  
  if [ -f "$DMG_FILE" ]; then
    log "SUCCESS" "DMG installer created successfully: $DMG_FILE"
  fi
}

# Function to clean up
cleanup() {
  log "STEP" "Cleaning up..."
  
  # Ask if user wants to clean up build files
  read -p "Do you want to clean up temporary build files? (y/N): " -n 1 -r
  echo
  if [[ $REPLY =~ ^[Yy]$ ]]; then
    rm -rf "$BUILD_DIR/debug"
    log "INFO" "Temporary build files cleaned up."
  fi
  
  log "SUCCESS" "Build process completed."
}

# Main function
main() {
  echo "========================================================"
  echo "  Skyscope Solana MEV Bot - macOS Build Script"
  echo "========================================================"
  echo
  
  # Initialize log file
  echo "Build started at $(date)" > "$LOG_FILE"
  
  # Check dependencies
  check_dependencies
  
  # Setup project structure
  setup_project_structure
  
  # Build frontend
  build_frontend
  
  # Build backend
  build_backend
  
  # Build Tauri app
  build_tauri_app
  
  # Create universal binary
  create_universal_binary
  
  # Package the app
  package_app
  
  # Sign the app
  sign_app
  
  # Create DMG installer
  create_dmg
  
  # Clean up
  cleanup
  
  log "SUCCESS" "Build completed successfully!"
  log "INFO" "You can find the app at: $MACOS_BUILD_DIR/$APP_NAME.app"
  
  if [ -f "$DIST_DIR/${APP_NAME// /-}-$APP_VERSION.dmg" ]; then
    log "INFO" "DMG installer: $DIST_DIR/${APP_NAME// /-}-$APP_VERSION.dmg"
  fi
  
  echo
  echo "========================================================"
  echo "  Build process completed successfully!"
  echo "========================================================"
}

# Run the main function
main

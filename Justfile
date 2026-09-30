# Justfile for cosmic-applet-caffeine
# Run 'just --list' to see all available commands

# Installation paths (configurable)
prefix := "/usr"
bin_dir := prefix / "bin"
share_dir := prefix / "share"
applications_dir := share_dir / "applications"
icons_dir := share_dir / "icons/hicolor/scalable/apps"

# Local user installation paths
local_bin := env_var("HOME") / ".local/bin"
local_apps := env_var("HOME") / ".local/share/applications"

# Application metadata
app_name := "cosmic-applet-caffeine"
desktop_file := "com.github.cosmic-applet-caffeine.desktop"
icon_files := "caffeine-cup-full.svg caffeine-cup-empty.svg"

# Default recipe - show help
default:
    @just --list

# Build the project in release mode
build:
    @echo "Building cosmic-applet-caffeine..."
    cargo build --release
    @echo "Build complete! Binary located at: target/release/{{app_name}}"

# Build in debug mode for development
build-debug:
    @echo "Building in debug mode..."
    cargo build
    @echo "Debug build complete!"

# Install to system directories (requires sudo)
install: build
    @echo ""
    @echo "Installing cosmic-applet-caffeine to system directories..."
    @echo "=================================================="
    @echo ""
    @echo "Installing binary to {{bin_dir}}/{{app_name}}..."
    sudo install -Dm755 target/release/{{app_name}} {{bin_dir}}/{{app_name}}
    @echo "Installing desktop entry to {{applications_dir}}/{{desktop_file}}..."
    sudo install -Dm644 assets/{{desktop_file}} {{applications_dir}}/{{desktop_file}}
    @echo "Installing icons to {{icons_dir}}..."
    for icon in {{icon_files}}; do sudo install -Dm644 "assets/$icon" "{{icons_dir}}/$icon"; done
    @echo ""
    @echo "Updating icon cache..."
    -sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
    @echo ""
    @echo "=================================================="
    @echo "Installation complete!"
    @echo ""
    @echo "NEXT STEPS:"
    @echo "  1. Log out and log back in, OR restart the COSMIC panel"
    @echo "  2. Open Settings > Desktop > Panel"
    @echo "  3. Add 'Caffeine' applet to your panel"
    @echo ""

# Install to user's local directories (no sudo required)
install-local: build
    @echo ""
    @echo "Installing cosmic-applet-caffeine to user directories..."
    @echo "=================================================="
    @echo ""
    @mkdir -p {{local_bin}}
    @echo "Installing binary to {{local_bin}}/{{app_name}}..."
    install -Dm755 target/release/{{app_name}} {{local_bin}}/{{app_name}}
    @mkdir -p {{local_apps}}
    @echo "Installing desktop entry to {{local_apps}}/{{desktop_file}}..."
    install -Dm644 assets/{{desktop_file}} {{local_apps}}/{{desktop_file}}
    @echo ""
    @echo "NOTE: Icons will be installed system-wide (requires sudo)..."
    sudo mkdir -p {{icons_dir}}
    for icon in {{icon_files}}; do sudo install -Dm644 "assets/$icon" "{{icons_dir}}/$icon"; done
    -sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
    @echo ""
    @echo "Updating desktop database..."
    -update-desktop-database {{local_apps}} 2>/dev/null || true
    @echo ""
    @echo "=================================================="
    @echo "Installation complete!"
    @echo ""
    @echo "NEXT STEPS:"
    @echo "  1. Make sure ~/.local/bin is in your PATH"
    @echo "  2. Log out and log back in, OR restart the COSMIC panel"
    @echo "  3. Open Settings > Desktop > Panel"
    @echo "  4. Add 'Caffeine' applet to your panel"
    @echo ""

# Uninstall from system directories (requires sudo)
uninstall:
    @echo ""
    @echo "Uninstalling cosmic-applet-caffeine from system directories..."
    @echo "======================================================="
    @echo ""
    @echo "Removing binary..."
    -sudo rm -f {{bin_dir}}/{{app_name}}
    @echo "Removing desktop entry..."
    -sudo rm -f {{applications_dir}}/{{desktop_file}}
    @echo "Removing icons..."
    -for icon in {{icon_files}}; do sudo rm -f "{{icons_dir}}/$icon"; done
    @echo ""
    @echo "Updating icon cache..."
    -sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
    @echo ""
    @echo "======================================================="
    @echo "Uninstallation complete!"
    @echo "You may need to restart the COSMIC panel or log out/in."
    @echo ""

# Uninstall from user's local directories
uninstall-local:
    @echo ""
    @echo "Uninstalling cosmic-applet-caffeine from user directories..."
    @echo "======================================================"
    @echo ""
    @echo "Removing binary..."
    -rm -f {{local_bin}}/{{app_name}}
    @echo "Removing desktop entry..."
    -rm -f {{local_apps}}/{{desktop_file}}
    @echo "Removing icons..."
    -for icon in {{icon_files}}; do sudo rm -f "{{icons_dir}}/$icon"; done
    @echo ""
    @echo "Updating icon cache..."
    -sudo gtk-update-icon-cache -f -t /usr/share/icons/hicolor 2>/dev/null || true
    @echo ""
    @echo "======================================================"
    @echo "Uninstallation complete!"
    @echo ""

# Package the application as a .rpm file
package: build
    @echo "Creating .rpm package..."
    cargo generate-rpm
    @echo ""
    @echo "Package created at: target/generate-rpm/"
    @ls -la target/generate-rpm/*.rpm 2>/dev/null || echo "No .rpm files found"

# Run the application (for testing)
run:
    cargo run

# Run with debug logging
run-debug:
    RUST_LOG=debug cargo run

# Clean build artifacts
clean:
    @echo "Cleaning build artifacts..."
    cargo clean
    @echo "Clean complete!"

# Run tests
test:
    cargo test

# Check for compilation errors without building
check:
    cargo check

# Format code
fmt:
    cargo fmt

# Run clippy linter
lint:
    cargo clippy -- -D warnings

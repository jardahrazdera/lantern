# Installation Guide

## System Requirements

### Operating System
- **Linux with systemd** (required)
- Tested on: Arch Linux, Ubuntu 22.04+, Fedora 38+, Debian 12+

### Required Dependencies
```bash
# Arch Linux
sudo pacman -S iproute2 wireless-tools systemd-networkd

# Ubuntu/Debian
sudo apt install iproute2 wireless-tools systemd-networkd

# Fedora/RHEL
sudo dnf install iproute wireless-tools systemd-networkd
```

### Optional Dependencies (for full feature support)
```bash
# WiFi hotspot support
sudo pacman -S hostapd dnsmasq       # Arch
sudo apt install hostapd dnsmasq     # Ubuntu/Debian
sudo dnf install hostapd dnsmasq     # Fedora

# WireGuard VPN support  
sudo pacman -S wireguard-tools       # Arch
sudo apt install wireguard-tools     # Ubuntu/Debian
sudo dnf install wireguard-tools     # Fedora

# Enterprise WiFi support
sudo pacman -S wpa_supplicant        # Arch
sudo apt install wpasupplicant       # Ubuntu/Debian
sudo dnf install wpa_supplicant      # Fedora
```

## Installation Methods

### Method 1: Download Pre-built Binary (Recommended)

1. **Download the latest release:**
   ```bash
   wget https://github.com/jardahrazdera/lantern/releases/latest/download/lantern
   chmod +x lantern
   sudo mv lantern /usr/local/bin/
   ```

2. **Verify installation:**
   ```bash
   lantern --version
   ```

### Method 2: Build from Source

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **Clone and build:**
   ```bash
   git clone https://github.com/jardahrazdera/lantern.git
   cd lantern
   cargo build --release
   sudo cp target/release/lantern /usr/local/bin/
   ```

3. **Verify installation:**
   ```bash
   lantern --version
   ```

### Method 3: Package Managers

#### Arch Linux (AUR)
```bash
# Using yay
yay -S lantern

# Using paru  
paru -S lantern

# Manual with makepkg
git clone https://aur.archlinux.org/lantern.git
cd lantern
makepkg -si
```

#### Ubuntu/Debian (DEB package)
```bash
wget https://github.com/jardahrazdera/lantern/releases/latest/download/lantern_0.1.0_amd64.deb
sudo dpkg -i lantern_0.1.0_amd64.deb
sudo apt-get install -f  # Fix any dependency issues
```

#### Fedora/RHEL (RPM package)
```bash
wget https://github.com/jardahrazdera/lantern/releases/latest/download/lantern-0.1.0-1.x86_64.rpm
sudo rpm -i lantern-0.1.0-1.x86_64.rpm
```

## Post-Installation Setup

### 1. Enable systemd-networkd (Required)
```bash
sudo systemctl enable --now systemd-networkd
sudo systemctl enable --now systemd-resolved
```

### 2. Configure Network Management
Disable conflicting network managers:
```bash
# If using NetworkManager (Ubuntu default)
sudo systemctl disable NetworkManager
sudo systemctl stop NetworkManager

# If using netplan (Ubuntu)
sudo systemctl disable systemd-networkd
# Edit /etc/netplan/ configs to use networkd renderer
```

### 3. Set up Permissions
Lantern requires root privileges for network configuration:
```bash
# Run with sudo
sudo lantern

# Or create a wrapper script
echo '#!/bin/bash\nsudo /usr/local/bin/lantern "$@"' | sudo tee /usr/local/bin/lantern-sudo
sudo chmod +x /usr/local/bin/lantern-sudo
```

## Verification

### Basic Functionality Test
```bash
# Test CLI mode (no root needed for version check)
lantern --version
lantern --help

# Test with root privileges
sudo lantern --cli
```

### Feature Test
```bash
# Run full TUI (requires root)
sudo lantern

# Expected features:
# ✓ Interface list with status
# ✓ WiFi scanning (w key)
# ✓ Network configuration (e key)
# ✓ Hotspot creation (h key)
```

## Troubleshooting

### Common Issues

#### "Permission denied" errors
**Problem:** Running without root privileges
**Solution:** Always run with `sudo lantern`

#### "TUI mode not available"
**Problem:** Terminal doesn't support required features
**Solution:** Use CLI mode: `sudo lantern --cli`

#### "Command not found: ip/iw/iwconfig"
**Problem:** Missing network tools
**Solution:** Install required dependencies (see above)

#### "systemd-networkd not running"
**Problem:** systemd-networkd service not enabled
**Solution:** 
```bash
sudo systemctl enable --now systemd-networkd
sudo systemctl status systemd-networkd
```

#### WiFi scanning fails
**Problem:** Wireless interface permissions or drivers
**Solution:**
```bash
# Check wireless interface
sudo iw dev
sudo rfkill list

# Ensure interface is up
sudo ip link set wlan0 up
```

### Logs and Debugging

View systemd-networkd logs:
```bash
sudo journalctl -u systemd-networkd -f
```

Check network interface status:
```bash
sudo networkctl status
sudo systemctl status systemd-networkd
```

## Uninstallation

### Remove Binary
```bash
sudo rm /usr/local/bin/lantern
```

### Remove Package
```bash
# AUR
sudo pacman -R lantern

# Debian/Ubuntu
sudo apt remove lantern

# Fedora/RHEL  
sudo rpm -e lantern
```

### Clean Configuration
```bash
# Remove any created network configs (optional)
sudo rm -f /etc/systemd/network/10-lantern-*.network
sudo rm -f /etc/systemd/network/25-lantern-*.network
sudo systemctl reload systemd-networkd
```

## Getting Help

- **Documentation:** [README.md](README.md)
- **Issues:** [GitHub Issues](https://github.com/jardahrazdera/lantern/issues)
- **Discussions:** [GitHub Discussions](https://github.com/jardahrazdera/lantern/discussions)

## Next Steps

After installation, see [README.md](README.md) for usage instructions and [examples/](examples/) for configuration examples.
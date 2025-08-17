# Changelog

All notable changes to Lantern will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-08-17

### 🎉 Initial Release: Modern TUI Network Manager

First public release of Lantern - a modern TUI for Linux network interface management with comprehensive WiFi support.

### ✨ Features

#### Core Network Management
- **Network interface management** with DHCP and static IP configuration
- **IPv6 support** with SLAAC, DHCPv6, and manual configuration
- **WireGuard VPN integration** with key generation and management
- **systemd-networkd integration** for persistent configuration
- **Real-time monitoring** with interface statistics and status

#### Advanced WiFi Management
- **Complete WiFi scanning and connection** with WPA/WPA2/WPA3 support
- **Enterprise WiFi support** (802.1X, PEAP, TTLS, TLS authentication)
- **WiFi hotspot creation** with SSID, password, and channel configuration
- **WiFi diagnostics** with detailed connection information and real-time statistics
- **WiFi connection history** and auto-connect functionality
- **Hidden network support** for enterprise environments

#### User Interface & Experience
- **Interactive TUI** built with ratatui for responsive terminal interface
- **CLI mode** for headless systems and automation (`--cli` flag)
- **Command-line arguments** support (`--help`, `--version`, `--cli`)
- **Professional help system** with detailed feature descriptions
- **Non-blocking operations** for responsive UI during network operations
- **Intuitive key bindings** for efficient navigation

#### Documentation & Quality
- **Comprehensive documentation** with installation and usage guides
- **Test suite** with 8 automated tests covering core functionality
- **GitHub Actions CI/CD** pipeline with automated testing and releases
- **Multi-platform builds** (x86_64-linux-gnu, x86_64-linux-musl)
- **Security-conscious** development with secret detection tests

### 🔧 Technical Details

#### Architecture
- **Rust 2021** with modern async/await patterns
- **Performance optimized** (3.9MB binary, <10MB RAM, <1% CPU)
- **Comprehensive error handling** with context-aware messages
- **Modular design** with clean separation of concerns

#### Dependencies
- **ratatui** for terminal UI framework
- **tokio** for async runtime
- **crossterm** for cross-platform terminal handling
- **clap** for command-line argument parsing
- **systemd integration** via networkctl and systemd-networkd

### 📋 System Requirements

- **Linux with systemd** (Arch, Ubuntu 22.04+, Fedora 38+, Debian 12+)
- **Root privileges** for network configuration
- **Terminal emulator** with TUI support (or use `--cli` mode)

#### Required Dependencies
- `iproute2`, `wireless-tools`, `systemd-networkd`

#### Optional Dependencies  
- `hostapd`, `dnsmasq` (for WiFi hotspot)
- `wireguard-tools` (for VPN support)
- `wpa_supplicant` (for Enterprise WiFi)

### 🚀 Installation

```bash
# Download and install
wget https://github.com/jardahrazdera/lantern/releases/download/v0.1.0/lantern-linux-x86_64.tar.gz
tar -xzf lantern-linux-x86_64.tar.gz
sudo cp lantern-linux-x86_64/lantern /usr/local/bin/

# Run
sudo lantern
```

### 📊 Performance Metrics

- **Binary size**: 3.9MB (optimized release build)
- **Memory usage**: <10MB RAM during normal operation  
- **CPU usage**: <1% during idle monitoring
- **Build time**: ~30 seconds on modern hardware
- **Test coverage**: 8 integration/unit tests

### 🎯 Key Bindings

#### Main Interface
- `↑↓/jk`: Navigate interfaces
- `Enter`: Toggle details
- `e`: Edit interface configuration
- `w`: WiFi management
- `h`: Create WiFi hotspot
- `u`: Toggle interface up/down
- `r`: Refresh
- `q`: Quit

#### WiFi Dialog
- `r`: Scan networks
- `a`: Toggle auto-connect
- `e`: Enterprise WiFi setup
- `d`: WiFi diagnostics
- `Enter`: Connect to network

### 🔧 Usage Examples

#### Basic Interface Configuration
```bash
sudo lantern              # Start TUI
# Press 'e' to edit interface
# Configure DHCP or static IP
# Press 's' to save
```

#### WiFi Connection
```bash
sudo lantern              # Start TUI  
# Press 'w' for WiFi dialog
# Press 'r' to scan
# Select network and press Enter
# Enter password if required
```

#### WiFi Hotspot
```bash
sudo lantern              # Start TUI
# Press 'h' for hotspot dialog
# Configure SSID, password, channel
# Press Enter to create
```

---

## Release Planning

### [0.2.0] - Planned (4-6 weeks)
- VLAN configuration (802.1Q)
- Network bonding/teaming
- Basic bandwidth monitoring with graphs
- Enhanced profile management

### [0.3.0] - Planned (3-4 weeks)  
- Bridge interfaces
- Advanced monitoring and diagnostics
- Custom color themes
- Package distribution (AUR, DEB, RPM)

---

## Contributing

This is the first public release! Contributions are welcome:

- **Issues**: [GitHub Issues](https://github.com/jardahrazdera/lantern/issues)
- **Features**: See [ROADMAP.md](ROADMAP.md) for planned features
- **Documentation**: Help improve guides and examples

## License

GPL-3.0 - see [LICENSE](LICENSE) file for details.
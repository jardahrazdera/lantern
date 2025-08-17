# Lantern 🏮

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

A modern, lightweight TUI (Terminal User Interface) application for managing network interfaces on Linux systems. Lantern provides an intuitive interface for configuring network settings through systemd-networkd, offering a clean alternative to traditional network management tools.

## ✨ Features

### Core Networking
- **Real-time interface monitoring** - Live statistics for RX/TX bytes, packets, and errors
- **Static and DHCP configuration** - Easy switching between automatic and manual network setup
- **IPv4 and IPv6 support** - Full dual-stack networking with modern protocols
- **systemd-networkd integration** - Native support for systemd-based network management

### Advanced Capabilities
- **WiFi management** - Network scanning, WPA/WPA2/WPA3 authentication, and connection management
- **VPN support** - WireGuard configuration and management
- **Interface control** - Bring interfaces up/down with real-time status updates
- **Configuration persistence** - All settings persist across reboots via systemd-networkd

### User Experience
- **Intuitive TUI** - Clean, responsive terminal interface built with Ratatui
- **Keyboard navigation** - Vim-like keybindings for efficient operation
- **Real-time updates** - Live network statistics and status monitoring
- **Error handling** - Comprehensive error messages and recovery suggestions

## 🚀 Quick Start

### Prerequisites

Lantern requires a Linux system with:
- **systemd-networkd** enabled
- **Root privileges** for network configuration
- **System tools**: `ip`, `iw`, `wg` (for wireless and VPN features)

### Installation

#### From Source
```bash
# Clone the repository
git clone https://github.com/yourusername/lantern.git
cd lantern

# Build the release version
cargo build --release

# Install (optional)
sudo cp target/release/lantern /usr/local/bin/
```

#### System Dependencies
```bash
# Arch Linux
sudo pacman -S iproute2 wireless-tools wireguard-tools

# Ubuntu/Debian
sudo apt install iproute2 wireless-tools wireguard-tools

# Fedora
sudo dnf install iproute wireless-tools wireguard-tools
```

### Enable systemd-networkd
```bash
# Enable and start systemd-networkd
sudo systemctl enable --now systemd-networkd
sudo systemctl enable --now systemd-resolved
```

## 🎯 Usage

### Launch Lantern
```bash
# Run with root privileges (required for network configuration)
sudo lantern
```

**If you encounter terminal issues:**
Some terminal emulators or SSH sessions may have compatibility issues with TUI applications under sudo. If you see terminal setup errors, try:

```bash
# Switch to root user first, then run
su - root
lantern

# Or for SSH sessions
ssh -t user@host sudo lantern
```

### Interface Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Lantern - Network Interface Manager       │
├─────────────────────────┬───────────────────────────────────┤
│ Interfaces [↑/↓ to nav] │ Statistics [Enter for details]    │
│                         │                                   │
│ eth0      UP    192.168.│ Network Statistics                │
│ wlan0     DOWN  No IP   │                                   │
│ lo        UP    127.0.0.│ ↓ RX: 45.2 MB                    │
│                         │   Packets: 1,234                 │
│                         │   Errors: 0                      │
│                         │                                   │
│                         │ ↑ TX: 12.8 MB                    │
│                         │   Packets: 892                   │
│                         │   Errors: 0                      │
└─────────────────────────┴───────────────────────────────────┘
│ ^Q: Quit | ^R: Refresh | e: Edit | u: Up/Down | Enter: Details │
└─────────────────────────────────────────────────────────────────┘
```

### Key Bindings

| Key | Action |
|-----|--------|
| `↑/↓` or `j/k` | Navigate interfaces |
| `Enter` | Toggle details/statistics view |
| `e` | Edit interface configuration |
| `u` | Toggle interface up/down |
| `Ctrl+R` | Refresh interface data |
| `Ctrl+Q` | Quit application |

### Configuration Dialog

When editing an interface (`e` key):

```
┌──────────────── Edit eth0 ────────────────┐
│                                           │
│ DHCP: [✓] Enabled (press 'd' to toggle)  │
│                                           │
│ ┌─────────────────────────────────────┐   │
│ │ IP Address                          │   │
│ │ 192.168.1.100/24                   │   │
│ └─────────────────────────────────────┘   │
│                                           │
│ ┌─────────────────────────────────────┐   │
│ │ Gateway                             │   │
│ │ 192.168.1.1                        │   │
│ └─────────────────────────────────────┘   │
│                                           │
│ ┌─────────────────────────────────────┐   │
│ │ DNS Servers (comma separated)       │   │
│ │ 8.8.8.8, 1.1.1.1                   │   │
│ └─────────────────────────────────────┘   │
│                                           │
│     Tab: Next field | s: Save | Esc: Cancel     │
└───────────────────────────────────────────┘
```

## 🔧 Configuration

### Network Profiles

Lantern creates systemd-networkd configuration files in `/etc/systemd/network/`:

- **Ethernet**: `10-<interface>.network`
- **WiFi**: `25-<interface>.network` + wpa_supplicant config
- **VPN**: `50-<interface>.netdev` and `50-<interface>.network`

### Example Configuration

Static IP configuration for eth0:
```ini
[Match]
Name=eth0

[Network]
Address=192.168.1.100/24
Gateway=192.168.1.1
DNS=8.8.8.8
DNS=1.1.1.1

[Link]
RequiredForOnline=yes
```

## 🛠️ Advanced Features

### WiFi Management
- **Network Scanning**: Discover available WiFi networks
- **Security Support**: WPA/WPA2/WPA3, WEP, and open networks
- **Connection Management**: Save and manage WiFi profiles

### IPv6 Support
- **Dual-stack networking**: Simultaneous IPv4 and IPv6
- **SLAAC and DHCPv6**: Automatic IPv6 configuration
- **Privacy extensions**: Enhanced privacy for IPv6 addresses

### VPN/WireGuard
- **Native WireGuard support**: Create and manage VPN connections
- **Key generation**: Automatic cryptographic key creation
- **Configuration import**: Support for standard WireGuard config files

## 🚨 Troubleshooting

### Common Issues

**"Permission denied" errors**
```bash
# Ensure you're running as root
sudo lantern
```

**"systemd-networkd not running"**
```bash
# Enable and start systemd-networkd
sudo systemctl enable --now systemd-networkd
sudo systemctl enable --now systemd-resolved
```

**"Command not found: ip/iw/wg"**
```bash
# Install required system tools (see Installation section)
```

**Interface changes not applying**
```bash
# Restart systemd-networkd
sudo systemctl restart systemd-networkd

# Or reload configuration
sudo networkctl reload
sudo networkctl reconfigure <interface>
```

### Logs and Debugging

```bash
# Check systemd-networkd status
sudo systemctl status systemd-networkd

# View network logs
sudo journalctl -u systemd-networkd -f

# Check interface status
networkctl status

# Verify configuration files
ls -la /etc/systemd/network/
```

## 🎯 Design Philosophy

Lantern follows these core principles:

1. **Simplicity**: Clean, intuitive interface focused on essential networking tasks
2. **Integration**: Native systemd-networkd support for modern Linux systems
3. **Performance**: Optimized for low resource usage and responsive operation
4. **Reliability**: Comprehensive error handling and graceful failure recovery

## 🔧 System Requirements

- **OS**: Linux with systemd
- **Memory**: < 10MB RAM usage
- **CPU**: Minimal CPU usage (< 1% during normal operation)
- **Storage**: 1.9MB binary size
- **Network**: systemd-networkd and systemd-resolved

## 🤝 Contributing

Contributions are welcome! Please feel free to submit issues, feature requests, or pull requests.

### Development Setup
```bash
git clone https://github.com/yourusername/lantern.git
cd lantern
cargo build
cargo test
```

### Code Structure
- `src/main.rs` - Application entry point and event loop
- `src/app.rs` - Application state management
- `src/network.rs` - Network interface operations
- `src/systemd.rs` - systemd-networkd configuration
- `src/ui.rs` - Terminal user interface
- `src/config.rs` - Configuration management
- `src/utils.rs` - Utility functions

## 📄 License

This project is licensed under the GNU General Public License v3.0 - see the [LICENCE](LICENCE) file for details.

## 🙏 Acknowledgments

- Built with [Ratatui](https://github.com/ratatui-org/ratatui) for the excellent TUI framework
- Inspired by `nmtui` and other network management tools
- Thanks to the Rust community for the amazing ecosystem

---

**Note**: Lantern is designed as an alternative to NetworkManager for users who prefer systemd-networkd. It provides a modern, efficient approach to network management on systemd-based Linux distributions.
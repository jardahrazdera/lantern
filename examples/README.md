# Lantern Configuration Examples

This directory contains comprehensive examples for configuring network interfaces with Lantern and systemd-networkd.

## 📂 Directory Structure

### [basic/](basic/) - Basic Ethernet Configuration
- **Static IP setup** - Home and office network configurations
- **DHCP configuration** - Automatic IP assignment with custom DNS
- **Common network ranges** - Typical setups for different environments

### [wifi/](wifi/) - WiFi Configuration  
- **WPA2/WPA3 Personal** - Home and office WiFi setup
- **Enterprise WiFi** - WPA2-Enterprise with RADIUS authentication
- **Open and hidden networks** - Various security configurations
- **wpa_supplicant integration** - Complete WiFi stack setup

### [ipv6/](ipv6/) - IPv6 Configuration
- **Dual-stack networking** - IPv4 + IPv6 configuration
- **SLAAC and DHCPv6** - Automatic IPv6 configuration
- **Static IPv6 addresses** - Manual IPv6 setup
- **Privacy extensions** - Enhanced IPv6 privacy

### [wireguard/](wireguard/) - VPN Configuration
- **Client configurations** - Full tunnel and split tunnel setups
- **Site-to-site VPN** - Office network connections
- **Multi-peer setup** - Complex VPN topologies
- **Key generation** - Security best practices

### [advanced/](advanced/) - Advanced Networking
- **Interface bonding** - High availability setups
- **VLAN configuration** - 802.1Q VLAN setup
- **Bridge networking** - Container and VM networking
- **Policy routing** - Complex routing scenarios

## 🚀 Quick Start

### 1. Choose Your Scenario
Browse the directories above to find the configuration that matches your needs.

### 2. Copy Configuration Files
```bash
# Example: Static home network
sudo cp examples/basic/static-home.network /etc/systemd/network/10-eth0.network

# Example: Home WiFi
sudo cp examples/wifi/home-wpa2.conf /etc/wpa_supplicant/wpa_supplicant-wlan0.conf
sudo cp examples/wifi/wlan-dhcp.network /etc/systemd/network/25-wlan0.network
```

### 3. Apply Configuration
```bash
# Reload systemd-networkd
sudo systemctl reload systemd-networkd

# Reconfigure specific interface
sudo networkctl reconfigure eth0

# For WiFi, enable wpa_supplicant
sudo systemctl enable --now wpa_supplicant@wlan0.service
```

### 4. Verify Setup
```bash
# Check interface status
sudo networkctl status

# Test connectivity
ping 8.8.8.8

# View IP addresses
ip addr show
```

## 🎯 Using with Lantern

### Interactive Configuration
1. Launch Lantern: `sudo lantern`
2. Select your network interface
3. Press `e` to edit configuration
4. Follow the TUI prompts to configure:
   - DHCP vs Static IP
   - IP address and subnet
   - Gateway and DNS servers
5. Press `s` to save configuration

### Benefits of Lantern + Examples
- **Visual interface** for network configuration
- **Real-time validation** of settings
- **Automatic file generation** following best practices
- **Error prevention** through guided setup
- **Live statistics** and monitoring

## 📋 Configuration File Locations

### systemd-networkd Files
```
/etc/systemd/network/
├── 10-eth0.network          # Ethernet configuration
├── 25-wlan0.network         # WiFi network configuration  
├── 50-wg0.netdev           # WireGuard interface definition
├── 50-wg0.network          # WireGuard network settings
└── 50-br0.netdev           # Bridge interface
```

### WiFi Configuration
```
/etc/wpa_supplicant/
└── wpa_supplicant-wlan0.conf  # WiFi credentials and settings
```

## 🛠️ Common Operations

### Enable systemd-networkd
```bash
sudo systemctl enable --now systemd-networkd
sudo systemctl enable --now systemd-resolved
```

### Restart Networking
```bash
# Reload all configurations
sudo systemctl restart systemd-networkd

# Reconfigure specific interface
sudo networkctl reconfigure eth0

# Force DHCP renewal
sudo networkctl renew eth0
```

### Troubleshooting
```bash
# Check service status
sudo systemctl status systemd-networkd

# View logs
sudo journalctl -u systemd-networkd -f

# List all interfaces
sudo networkctl list

# Get detailed interface info
sudo networkctl status eth0
```

## 📖 Configuration Syntax

### Basic Network File Structure
```ini
[Match]
Name=interface_name

[Network]
DHCP=yes|no|ipv4|ipv6
Address=IP/PREFIX
Gateway=IP_ADDRESS
DNS=IP_ADDRESS

[Link]
RequiredForOnline=yes|no
MTUBytes=1500
```

### Common Parameters

| Parameter | Description | Example |
|-----------|-------------|---------|
| `Address` | Static IP address | `192.168.1.100/24` |
| `Gateway` | Default gateway | `192.168.1.1` |
| `DNS` | DNS server | `8.8.8.8` |
| `DHCP` | DHCP mode | `yes`, `ipv4`, `ipv6` |
| `MTUBytes` | Interface MTU | `1500`, `9000` |

## 🔧 Best Practices

### Security
- Use strong WiFi passwords (WPA2/WPA3)
- Enable IPv6 privacy extensions
- Regularly rotate WireGuard keys
- Limit WireGuard AllowedIPs to necessary networks

### Performance  
- Use jumbo frames (MTU 9000) for high-performance networks
- Configure interface bonding for redundancy
- Optimize routing metrics for multi-interface setups
- Use appropriate TCP congestion control algorithms

### Maintenance
- Keep configuration files organized with descriptive names
- Document custom configurations
- Regular testing of backup interfaces and routes
- Monitor interface statistics and error rates

## 📚 Additional Resources

- [systemd-networkd Documentation](https://www.freedesktop.org/software/systemd/man/systemd-networkd.html)
- [WireGuard Documentation](https://www.wireguard.com/quickstart/)
- [IPv6 Configuration Guide](https://wiki.archlinux.org/title/IPv6)
- [WiFi Security Best Practices](https://www.wi-fi.org/security-update-2020)

## 🤝 Contributing Examples

Have a useful configuration? Contribute by:
1. Creating a new example file
2. Adding documentation with usage instructions  
3. Testing the configuration thoroughly
4. Submitting a pull request

Examples should be:
- **Well-documented** with clear comments
- **Tested** on real systems
- **Secure** following best practices
- **Practical** for real-world use cases
# Usage Guide

## Quick Start

### Basic Usage
```bash
# Start Lantern TUI (requires root)
sudo lantern

# CLI mode (for headless systems)
sudo lantern --cli

# Show help
lantern --help

# Show version
lantern --version
```

## TUI Interface

### Main Interface Layout
```
┌─ Interfaces ─────────────────┬─ Details ─────────────────────┐
│ > eth0     [UP]   192.168.1.5│ Interface: eth0               │
│   wlan0    [UP]   WiFi: Home │ State: UP                     │
│   docker0  [DOWN] No IP      │ IPv4: 192.168.1.5/24          │
│                              │ Gateway: 192.168.1.1          │
│                              │ DNS: 192.168.1.1              │
│                              │                               │
│                              │ Statistics:                   │
│                              │ RX: 1.2 GB  TX: 856 MB        │
└──────────────────────────────┴───────────────────────────────┘
Keys: ↑↓/jk: Navigate  Enter: Details  e: Edit  w: WiFi  q: Quit
```

### Key Bindings

#### Navigation
- `↑↓` or `j/k` - Navigate interface list
- `Enter` - Toggle detailed view for selected interface
- `Tab` - Navigate between input fields (in dialogs)
- `Esc` - Close dialogs/go back

#### Interface Management  
- `e` - Edit interface configuration (IP, DNS, etc.)
- `u` - Toggle interface up/down state
- `r` - Refresh interface list
- `Ctrl+R` - Force refresh all data

#### WiFi Management
- `w` - Open WiFi dialog
- `h` - Create WiFi hotspot

#### System
- `q` or `Ctrl+C` - Quit application

### WiFi Management

#### Scanning and Connecting
1. Press `w` to open WiFi dialog
2. Press `r` to scan for networks
3. Use `↑↓` to select network
4. Press `Enter` to connect
5. Enter password if required

#### WiFi Dialog Keys
- `r` - Refresh/scan networks
- `a` - Toggle auto-connect for selected network
- `e` - Configure Enterprise WiFi (802.1X)
- `d` - Show detailed WiFi diagnostics
- `Enter` - Connect to selected network
- `Esc` - Close WiFi dialog

#### Enterprise WiFi Setup
1. In WiFi dialog, press `e` for Enterprise mode
2. Configure authentication:
   - **Method**: PEAP, TTLS, or TLS
   - **Phase 2**: MSCHAPv2, PAP, CHAP, etc.
   - **Identity**: Your username
   - **Password**: Your password
   - **Certificates**: CA cert, client cert, private key paths
3. Press `Enter` to connect

#### Hotspot Creation
1. Press `h` to open hotspot dialog
2. Configure:
   - **SSID**: Network name
   - **Password**: WPA2 password (8+ characters)
   - **Channel**: WiFi channel (1-11, use Space to cycle)
3. Press `Enter` to create hotspot

### Interface Configuration

#### Edit Interface (Press `e`)
```
┌─ Edit Interface: eth0 ────────────────────────┐
│                                               │
│ ○ DHCP Configuration                          │
│ ● Static Configuration                        │
│                                               │
│ IP Address: [192.168.1.100/24        ]        │
│ Gateway:    [192.168.1.1             ]        │
│ DNS Server: [8.8.8.8                 ]        │
│                                               │
│ [Save]                    [Cancel]            │
└───────────────────────────────────────────────┘
```

#### Configuration Options
- **DHCP Mode**: Automatic IP configuration
- **Static Mode**: Manual IP, gateway, and DNS
- **Space**: Toggle between DHCP/Static
- **Tab**: Navigate between fields
- **s**: Save configuration
- **Esc**: Cancel changes

## CLI Mode

### Command-line Usage
```bash
# View network interfaces (no root required for info)
sudo lantern --cli

# Example output:
# 📡 Network Interfaces:
#    Interface    State    IP Address      RX         TX
#    ------------------------------------------------------------
#    eth0         UP       192.168.1.5     1.2GB      856MB
#    wlan0        UP       192.168.2.100   245MB      128MB
#    docker0      DOWN     172.17.0.1      0B         0B
```

## Configuration Examples

### Static IP Configuration
1. Select interface and press `e`
2. Select "Static Configuration" (press Space if needed)
3. Fill in:
   - IP Address: `192.168.1.100/24`
   - Gateway: `192.168.1.1`
   - DNS Server: `8.8.8.8`
4. Press `s` to save

### WiFi Network Connection
1. Press `w` for WiFi dialog
2. Press `r` to scan
3. Select your network
4. Press `Enter`
5. Enter password
6. Press `Enter` to connect

### Enterprise WiFi (Corporate/University)
1. Press `w` for WiFi dialog  
2. Select enterprise network
3. Press `e` for Enterprise mode
4. Configure:
   - Method: `PEAP`
   - Phase 2: `MSCHAPv2`
   - Identity: `username@domain.com`
   - Password: `your_password`
5. Press `Enter` to connect

### WiFi Hotspot Creation
1. Press `h` for hotspot dialog
2. Configure:
   - SSID: `MyHotspot`
   - Password: `MyPassword123`
   - Channel: `6` (press Space to cycle)
3. Press `Enter` to start

## Advanced Features

### WiFi Diagnostics
- In WiFi dialog, press `d` for detailed connection info
- Shows signal strength, link quality, connection time
- Network statistics (packets, errors, retries)
- Press `r` to refresh data

### Auto-connect WiFi
- In WiFi dialog, press `a` to toggle auto-connect
- Lantern remembers networks and auto-connects
- Useful for frequently used networks

### Profile Management
- Configurations are automatically saved
- systemd-networkd integration ensures persistence
- Network settings survive reboots

## Troubleshooting

### Interface Won't Come Up
1. Check if interface exists: `ip link show`
2. Try bringing up manually: `sudo ip link set eth0 up`
3. Check systemd-networkd: `sudo systemctl status systemd-networkd`

### WiFi Scanning Fails
1. Ensure WiFi interface is up: `sudo ip link set wlan0 up`
2. Check for RF kill: `sudo rfkill list`
3. Verify wireless drivers: `lsmod | grep -i wifi`

### Can't Connect to WiFi
1. Verify password is correct
2. Check signal strength (move closer to router)
3. Try manual connection: `sudo wpa_supplicant -i wlan0 -c /etc/wpa_supplicant/wpa_supplicant.conf`

### DHCP Not Working
1. Check DHCP server is running on network
2. Verify interface configuration: `sudo networkctl status eth0`
3. Restart systemd-networkd: `sudo systemctl restart systemd-networkd`

### Enterprise WiFi Issues
1. Verify credentials with network administrator
2. Check certificate paths are correct
3. Ensure certificates have proper permissions
4. Test with other Enterprise WiFi tools first

## Tips and Best Practices

### Performance
- Use `Ctrl+R` sparingly - it triggers full refresh
- WiFi scanning can take 5-10 seconds
- Interface statistics update automatically

### Security
- Always use WPA2/WPA3 for WiFi
- Use strong passwords for hotspots (12+ characters)
- Be cautious with Enterprise WiFi certificates

### System Integration
- Lantern integrates with systemd-networkd
- Configuration persists across reboots
- Compatible with systemd-resolved for DNS

### Workflow Recommendations
1. Use auto-connect for frequently used WiFi networks
2. Keep Enterprise WiFi certificates in `/etc/ssl/certs/`
3. Use static IP for servers, DHCP for workstations
4. Test hotspot functionality before relying on it

## Getting Help

### Built-in Help
- `lantern --help` - Command-line options
- In TUI: Press `?` for context-sensitive help (coming soon)

### External Resources
- [Installation Guide](INSTALL.md)
- [Examples Directory](examples/)
- [GitHub Issues](https://github.com/jardahrazdera/lantern/issues)

### Log Files
- systemd-networkd: `sudo journalctl -u systemd-networkd`
- WiFi connections: `sudo journalctl -u wpa_supplicant@*`
- General network: `sudo dmesg | grep -i network`
# Basic Ethernet Configuration Examples

This directory contains examples for basic ethernet interface configuration using systemd-networkd.

## Static IP Configuration

### Example: Home Network Setup
```ini
# /etc/systemd/network/10-eth0.network
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

### Example: Server Configuration
```ini
# /etc/systemd/network/10-eth0.network
[Match]
Name=eth0

[Network]
Address=10.0.1.50/24
Gateway=10.0.1.1
DNS=10.0.1.1
DNS=1.1.1.1

[Link]
RequiredForOnline=yes
MTUBytes=1500
```

## DHCP Configuration

### Example: Automatic Configuration
```ini
# /etc/systemd/network/10-eth0.network
[Match]
Name=eth0

[Network]
DHCP=yes

[DHCP]
UseDNS=yes
UseRoutes=yes
UseTimezone=yes

[Link]
RequiredForOnline=yes
```

### Example: DHCP with Static DNS
```ini
# /etc/systemd/network/10-eth0.network
[Match]
Name=eth0

[Network]
DHCP=yes
DNS=1.1.1.1
DNS=8.8.8.8

[DHCP]
UseDNS=no
UseRoutes=yes

[Link]
RequiredForOnline=yes
```

## Usage in Lantern

1. Launch Lantern as root: `sudo lantern`
2. Select your ethernet interface (e.g., eth0)
3. Press `e` to edit configuration
4. Toggle DHCP with `d` or configure static settings
5. Fill in IP address, gateway, and DNS servers
6. Press `s` to save configuration
7. Press `Ctrl+R` to refresh and see changes

## Common Network Ranges

| Use Case | Network Range | Gateway | DNS |
|----------|---------------|---------|-----|
| Home Router | 192.168.1.0/24 | 192.168.1.1 | 192.168.1.1, 8.8.8.8 |
| Office Network | 10.0.0.0/24 | 10.0.0.1 | 10.0.0.1, 1.1.1.1 |
| Small Business | 172.16.1.0/24 | 172.16.1.1 | 172.16.1.1, 8.8.8.8 |
| Lab Environment | 192.168.100.0/24 | 192.168.100.1 | 1.1.1.1, 8.8.8.8 |
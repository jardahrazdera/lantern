# IPv6 Configuration Examples

This directory contains examples for IPv6 configuration with systemd-networkd.

## Dual-Stack (IPv4 + IPv6)

### Auto-configuration with Router Advertisements
```ini
# /etc/systemd/network/10-eth0.network
[Match]
Name=eth0

[Network]
# IPv4 configuration
DHCP=ipv4
# IPv6 auto-configuration
IPv6AcceptRA=yes

[Link]
RequiredForOnline=yes
```

### Static IPv4 + Auto IPv6
```ini
# /etc/systemd/network/10-eth0.network
[Match]
Name=eth0

[Network]
# Static IPv4
Address=192.168.1.100/24
Gateway=192.168.1.1
DNS=192.168.1.1

# IPv6 auto-configuration
IPv6AcceptRA=yes
IPv6PrivacyExtensions=yes

[Link]
RequiredForOnline=yes
```

## IPv6-Only Network

### SLAAC (Stateless Address Autoconfiguration)
```ini
# /etc/systemd/network/10-eth0.network
[Match]
Name=eth0

[Network]
IPv6AcceptRA=yes
IPv6PrivacyExtensions=yes
DNS=2001:4860:4860::8888
DNS=2001:4860:4860::8844

[Link]
RequiredForOnline=yes
```

### DHCPv6 (Stateful)
```ini
# /etc/systemd/network/10-eth0.network
[Match]
Name=eth0

[Network]
DHCP=ipv6
IPv6AcceptRA=yes

[DHCPv6]
UseDNS=yes

[Link]
RequiredForOnline=yes
```

## Static IPv6 Configuration

### Manual IPv6 Address
```ini
# /etc/systemd/network/10-eth0.network
[Match]
Name=eth0

[Network]
# IPv4 settings
Address=192.168.1.100/24
Gateway=192.168.1.1

# Static IPv6 settings
Address=2001:db8:1::100/64
Gateway=2001:db8:1::1
DNS=2001:4860:4860::8888
DNS=2001:4860:4860::8844

[Link]
RequiredForOnline=yes
```

### Link-Local Only
```ini
# /etc/systemd/network/10-eth0.network
[Match]
Name=eth0

[Network]
IPv6AcceptRA=no
LinkLocalAddressing=ipv6

[Link]
RequiredForOnline=yes
```

## IPv6 Privacy and Security

### Privacy Extensions Enabled
```ini
# /etc/systemd/network/10-eth0.network
[Match]
Name=eth0

[Network]
DHCP=ipv4
IPv6AcceptRA=yes
# Enable privacy extensions for better anonymity
IPv6PrivacyExtensions=yes

[Link]
RequiredForOnline=yes
```

### Disable IPv6 (if needed)
```ini
# /etc/systemd/network/10-eth0.network
[Match]
Name=eth0

[Network]
DHCP=ipv4
# Explicitly disable IPv6
IPv6AcceptRA=no
LinkLocalAddressing=ipv4

[Link]
RequiredForOnline=yes
```

## Usage in Lantern

1. Launch Lantern as root: `sudo lantern`
2. Select your interface
3. Press `e` to edit configuration
4. Configure IPv4 settings as needed
5. Use IPv6 configuration options:
   - Enable IPv6 auto-configuration
   - Set static IPv6 addresses
   - Configure IPv6 DNS servers
6. Press `s` to save configuration

## Common IPv6 Addresses

| Service | IPv6 Address |
|---------|--------------|
| Google DNS | 2001:4860:4860::8888, 2001:4860:4860::8844 |
| Cloudflare DNS | 2606:4700:4700::1111, 2606:4700:4700::1001 |
| Quad9 DNS | 2620:fe::fe, 2620:fe::9 |

## Testing IPv6

```bash
# Test IPv6 connectivity
ping6 google.com

# Check IPv6 addresses
ip -6 addr show

# Test DNS resolution
nslookup google.com 2001:4860:4860::8888

# Check routing
ip -6 route show
```
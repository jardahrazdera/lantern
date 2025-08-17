# WireGuard VPN Configuration Examples

This directory contains examples for WireGuard VPN configuration using systemd-networkd.

## Basic Client Configuration

### Simple VPN Client
```ini
# /etc/systemd/network/50-wg0.netdev
[NetDev]
Name=wg0
Kind=wireguard
Description=WireGuard VPN tunnel

[WireGuard]
PrivateKey=YOUR_PRIVATE_KEY_HERE
ListenPort=51820

[WireGuardPeer]
PublicKey=SERVER_PUBLIC_KEY_HERE
Endpoint=vpn.example.com:51820
AllowedIPs=0.0.0.0/0, ::/0
PersistentKeepalive=25
```

```ini
# /etc/systemd/network/50-wg0.network
[Match]
Name=wg0

[Network]
Address=10.0.0.2/24
DNS=1.1.1.1
DNS=8.8.8.8

[Link]
RequiredForOnline=no
```

### Site-to-Site VPN
```ini
# /etc/systemd/network/50-wg-office.netdev
[NetDev]
Name=wg-office
Kind=wireguard
Description=Office VPN tunnel

[WireGuard]
PrivateKey=YOUR_PRIVATE_KEY_HERE

[WireGuardPeer]
PublicKey=OFFICE_SERVER_PUBLIC_KEY
Endpoint=office.company.com:51820
AllowedIPs=10.1.0.0/24, 10.2.0.0/24
PersistentKeepalive=25
```

```ini
# /etc/systemd/network/50-wg-office.network
[Match]
Name=wg-office

[Network]
Address=10.0.1.10/24

[Route]
Destination=10.1.0.0/24
Gateway=10.0.1.1

[Route]
Destination=10.2.0.0/24
Gateway=10.0.1.1

[Link]
RequiredForOnline=no
```

## Advanced Configurations

### Multi-Peer Setup
```ini
# /etc/systemd/network/50-wg-multi.netdev
[NetDev]
Name=wg-multi
Kind=wireguard
Description=Multi-peer WireGuard

[WireGuard]
PrivateKey=YOUR_PRIVATE_KEY_HERE
ListenPort=51820

# Peer 1: Main VPN server
[WireGuardPeer]
PublicKey=SERVER1_PUBLIC_KEY
Endpoint=server1.vpn.com:51820
AllowedIPs=10.0.1.0/24
PersistentKeepalive=25

# Peer 2: Office network
[WireGuardPeer]
PublicKey=SERVER2_PUBLIC_KEY
Endpoint=office.company.com:51821
AllowedIPs=192.168.100.0/24
PersistentKeepalive=25
```

### Mobile/Roaming Client
```ini
# /etc/systemd/network/50-wg-mobile.netdev
[NetDev]
Name=wg-mobile
Kind=wireguard
Description=Mobile WireGuard client

[WireGuard]
PrivateKey=MOBILE_PRIVATE_KEY

[WireGuardPeer]
PublicKey=VPN_SERVER_PUBLIC_KEY
Endpoint=vpn.example.com:51820
AllowedIPs=0.0.0.0/0
PersistentKeepalive=25
```

```ini
# /etc/systemd/network/50-wg-mobile.network
[Match]
Name=wg-mobile

[Network]
Address=10.0.2.5/24
DNS=1.1.1.1
DNS=8.8.8.8

# Route all traffic through VPN
[Route]
Gateway=10.0.2.1
GatewayOnLink=yes

[Link]
RequiredForOnline=no
MTUBytes=1420
```

## Key Generation

### Generate Keys
```bash
# Generate private key
wg genkey > private.key

# Generate public key from private key
wg pubkey < private.key > public.key

# Generate pre-shared key (optional, for extra security)
wg genpsk > preshared.key

# Display keys
echo "Private Key: $(cat private.key)"
echo "Public Key: $(cat public.key)"
echo "Pre-shared Key: $(cat preshared.key)"
```

### Secure Key Storage
```bash
# Set proper permissions
sudo chmod 600 /etc/systemd/network/*.netdev
sudo chown root:systemd-network /etc/systemd/network/*.netdev
```

## Usage in Lantern

1. Launch Lantern as root: `sudo lantern`
2. Press `w` to create new WireGuard interface (if implemented)
3. Or manually configure:
   - Generate keys using `wg genkey` and `wg pubkey`
   - Create .netdev and .network files
   - Use Lantern to manage the interface

## Standard WireGuard Config Conversion

### From Standard Format
```ini
# Standard wg0.conf format
[Interface]
PrivateKey=YOUR_PRIVATE_KEY
Address=10.0.0.2/24
DNS=1.1.1.1

[Peer]
PublicKey=SERVER_PUBLIC_KEY
Endpoint=vpn.example.com:51820
AllowedIPs=0.0.0.0/0
PersistentKeepalive=25
```

### To systemd-networkd Format
Split into two files as shown in the examples above.

## Testing WireGuard

```bash
# Check interface status
sudo wg show

# Check systemd-networkd status
sudo networkctl status wg0

# Test connectivity
ping 10.0.0.1

# Check routing
ip route show table all | grep wg0

# Monitor logs
sudo journalctl -u systemd-networkd -f
```

## Common Port Numbers

| Use Case | Port | Notes |
|----------|------|-------|
| Default | 51820 | Standard WireGuard port |
| Multiple tunnels | 51821-51830 | Avoid conflicts |
| Corporate | 443 | Bypass firewalls |
| Alternative | 1194 | OpenVPN standard port |

## Security Best Practices

1. **Use strong private keys** - Generated with `wg genkey`
2. **Limit AllowedIPs** - Only route necessary traffic
3. **Regular key rotation** - Change keys periodically
4. **Firewall rules** - Restrict access to WireGuard ports
5. **Monitor connections** - Use `wg show` regularly
# Advanced Networking Examples

This directory contains examples for advanced networking scenarios.

## Multi-Interface Setup

### Ethernet + WiFi Failover
```ini
# /etc/systemd/network/10-eth0.network (Priority interface)
[Match]
Name=eth0

[Network]
DHCP=yes

[DHCP]
RouteMetric=100

[Link]
RequiredForOnline=yes
```

```ini
# /etc/systemd/network/25-wlan0.network (Backup interface)
[Match]
Name=wlan0

[Network]
DHCP=yes

[DHCP]
RouteMetric=200

[Link]
RequiredForOnline=no
```

### Interface Bonding
```ini
# /etc/systemd/network/50-bond0.netdev
[NetDev]
Name=bond0
Kind=bond
Description=Bonded network interface

[Bond]
Mode=active-backup
MIIMonitorSec=100
```

```ini
# /etc/systemd/network/10-eth0.network
[Match]
Name=eth0

[Network]
Bond=bond0

[Link]
RequiredForOnline=no
```

```ini
# /etc/systemd/network/10-eth1.network
[Match]
Name=eth1

[Network]
Bond=bond0

[Link]
RequiredForOnline=no
```

```ini
# /etc/systemd/network/50-bond0.network
[Match]
Name=bond0

[Network]
DHCP=yes

[Link]
RequiredForOnline=yes
```

## VLAN Configuration

### 802.1Q VLAN Setup
```ini
# /etc/systemd/network/50-vlan100.netdev
[NetDev]
Name=vlan100
Kind=vlan
Description=VLAN 100

[VLAN]
Id=100
```

```ini
# /etc/systemd/network/10-eth0.network
[Match]
Name=eth0

[Network]
VLAN=vlan100

[Link]
RequiredForOnline=yes
```

```ini
# /etc/systemd/network/50-vlan100.network
[Match]
Name=vlan100

[Network]
Address=192.168.100.10/24
Gateway=192.168.100.1
DNS=192.168.100.1

[Link]
RequiredForOnline=yes
```

## Bridge Configuration

### Bridge Setup for VMs/Containers
```ini
# /etc/systemd/network/50-br0.netdev
[NetDev]
Name=br0
Kind=bridge
Description=Bridge for VMs
```

```ini
# /etc/systemd/network/10-eth0.network
[Match]
Name=eth0

[Network]
Bridge=br0

[Link]
RequiredForOnline=no
```

```ini
# /etc/systemd/network/50-br0.network
[Match]
Name=br0

[Network]
DHCP=yes
IPForward=yes

[Link]
RequiredForOnline=yes
```

## Complex Routing

### Policy-Based Routing
```ini
# /etc/systemd/network/10-eth0.network
[Match]
Name=eth0

[Network]
Address=192.168.1.100/24
Gateway=192.168.1.1

# Route table for specific traffic
[RoutingPolicyRule]
Table=100
From=192.168.1.0/24

[Route]
Gateway=192.168.1.1
Table=100

[Link]
RequiredForOnline=yes
```

### Multi-Homed Setup
```ini
# /etc/systemd/network/10-eth0.network (ISP 1)
[Match]
Name=eth0

[Network]
Address=203.0.113.10/24
Gateway=203.0.113.1

[Route]
Gateway=203.0.113.1
Table=1

[RoutingPolicyRule]
Table=1
Priority=100

[Link]
RequiredForOnline=yes
```

```ini
# /etc/systemd/network/10-eth1.network (ISP 2)
[Match]
Name=eth1

[Network]
Address=198.51.100.20/24
Gateway=198.51.100.1

[Route]
Gateway=198.51.100.1
Table=2

[RoutingPolicyRule]
Table=2
Priority=200

[Link]
RequiredForOnline=yes
```

## Network Namespaces Integration

### Isolated Network Namespace
```bash
# Create namespace
sudo ip netns add isolated

# Move interface to namespace
sudo ip link set wlan1 netns isolated

# Configure in namespace
sudo ip netns exec isolated ip addr add 192.168.10.1/24 dev wlan1
sudo ip netns exec isolated ip link set wlan1 up
```

```ini
# /etc/systemd/network/isolated.network
[Match]
Name=veth-isolated

[Network]
Address=192.168.10.2/24
Gateway=192.168.10.1

[Link]
RequiredForOnline=no
```

## High Availability Setup

### LACP Bonding
```ini
# /etc/systemd/network/50-bond-lacp.netdev
[NetDev]
Name=bond0
Kind=bond
Description=LACP bonded interface

[Bond]
Mode=802.3ad
TransmitHashPolicy=layer3+4
LACPTransmitRate=fast
MIIMonitorSec=100
```

### Redundant Gateway
```ini
# /etc/systemd/network/10-eth0.network
[Match]
Name=eth0

[Network]
Address=192.168.1.100/24

# Primary gateway
[Route]
Gateway=192.168.1.1
Metric=100

# Backup gateway
[Route]
Gateway=192.168.1.2
Metric=200

[Link]
RequiredForOnline=yes
```

## Performance Tuning

### High-Performance Interface
```ini
# /etc/systemd/network/10-eth0.network
[Match]
Name=eth0

[Network]
Address=10.0.0.100/24
Gateway=10.0.0.1

[Link]
RequiredForOnline=yes
# Jumbo frames for performance
MTUBytes=9000
# Optimize for performance
TCPCongestionControl=bbr
```

### Low-Latency Configuration
```ini
# /etc/systemd/network/10-gaming.network
[Match]
Name=eth0

[Network]
Address=192.168.1.100/24
Gateway=192.168.1.1
DNS=1.1.1.1

[DHCP]
# Minimize DHCP overhead
UseTimezone=no
UseNTP=no

[Link]
RequiredForOnline=yes
# Optimize MTU
MTUBytes=1500
```

## Testing Advanced Configurations

```bash
# Check interface status
sudo networkctl status

# Verify routing tables
ip route show table all

# Test connectivity
ping -I eth0 8.8.8.8

# Monitor traffic
sudo tcpdump -i any

# Check bond status
cat /proc/net/bonding/bond0

# Verify VLAN
ip -d link show

# Test namespace
sudo ip netns exec isolated ping 8.8.8.8
```

## Troubleshooting

```bash
# Check systemd-networkd logs
sudo journalctl -u systemd-networkd -f

# Restart networking
sudo systemctl restart systemd-networkd

# Force interface reconfiguration
sudo networkctl reconfigure eth0

# Check interface details
sudo networkctl status eth0

# Verify configuration syntax
sudo systemd-analyze verify /etc/systemd/network/*.network
```
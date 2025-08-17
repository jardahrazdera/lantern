# WiFi Configuration Examples

This directory contains examples for WiFi configuration using systemd-networkd + wpa_supplicant.

## WPA2 Personal (Most Common)

### Home WiFi with DHCP
```ini
# /etc/systemd/network/25-wlan0.network
[Match]
Name=wlan0

[Network]
DHCP=yes

[Link]
RequiredForOnline=yes
```

```ini
# /etc/wpa_supplicant/wpa_supplicant-wlan0.conf
ctrl_interface=/run/wpa_supplicant
update_config=1
country=US

network={
    ssid="YourHomeWiFi"
    psk="your_wifi_password"
    key_mgmt=WPA-PSK
}
```

### Office WiFi with Static IP
```ini
# /etc/systemd/network/25-wlan0.network
[Match]
Name=wlan0

[Network]
Address=192.168.1.150/24
Gateway=192.168.1.1
DNS=192.168.1.1
DNS=8.8.8.8

[Link]
RequiredForOnline=yes
```

## WPA3 (Modern Security)

```ini
# /etc/wpa_supplicant/wpa_supplicant-wlan0.conf
ctrl_interface=/run/wpa_supplicant
update_config=1
country=US

network={
    ssid="ModernWiFi"
    psk="secure_password_2023"
    key_mgmt=SAE
    ieee80211w=2
}
```

## Open Network (No Security)

```ini
# /etc/wpa_supplicant/wpa_supplicant-wlan0.conf
ctrl_interface=/run/wpa_supplicant
update_config=1
country=US

network={
    ssid="FreeWiFi"
    key_mgmt=NONE
}
```

## Hidden Network

```ini
# /etc/wpa_supplicant/wpa_supplicant-wlan0.conf
ctrl_interface=/run/wpa_supplicant
update_config=1
country=US

network={
    ssid="HiddenNetwork"
    psk="hidden_password"
    key_mgmt=WPA-PSK
    scan_ssid=1
}
```

## Enterprise WiFi (WPA2-Enterprise)

```ini
# /etc/wpa_supplicant/wpa_supplicant-wlan0.conf
ctrl_interface=/run/wpa_supplicant
update_config=1
country=US

network={
    ssid="CorporateWiFi"
    key_mgmt=WPA-EAP
    eap=PEAP
    identity="username@company.com"
    password="your_password"
    phase2="auth=MSCHAPV2"
}
```

## Usage in Lantern

1. Launch Lantern as root: `sudo lantern`
2. Select your WiFi interface (e.g., wlan0)
3. Press `w` to open WiFi dialog (if implemented)
4. Scan for networks and select one
5. Enter password and configure IP settings
6. Save configuration

## Manual Setup Steps

1. **Create wpa_supplicant config:**
   ```bash
   sudo cp examples/wifi/home-wpa2.conf /etc/wpa_supplicant/wpa_supplicant-wlan0.conf
   sudo chmod 600 /etc/wpa_supplicant/wpa_supplicant-wlan0.conf
   ```

2. **Create network config:**
   ```bash
   sudo cp examples/wifi/dhcp.network /etc/systemd/network/25-wlan0.network
   ```

3. **Enable and start services:**
   ```bash
   sudo systemctl enable wpa_supplicant@wlan0.service
   sudo systemctl start wpa_supplicant@wlan0.service
   sudo systemctl reload systemd-networkd
   ```

4. **Check status:**
   ```bash
   sudo networkctl status wlan0
   sudo wpa_cli -i wlan0 status
   ```
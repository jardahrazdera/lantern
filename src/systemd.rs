// src/systemd.rs
#![allow(dead_code)] // Many methods are for future features or CLI mode
use crate::network::{Ipv6Config, WifiCredentials, WifiSecurity, WireGuardConfig};
use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Clone)]
pub struct SystemdNetworkConfig;

impl SystemdNetworkConfig {
    pub fn new() -> Self {
        Self
    }

    pub async fn create_config(
        &self,
        interface: &str,
        dhcp: bool,
        ip: Option<String>,
        gateway: Option<String>,
        dns: Option<Vec<String>>,
    ) -> Result<()> {
        let config_dir = Path::new("/etc/systemd/network");
        if !config_dir.exists() {
            fs::create_dir_all(config_dir)?;
        }

        let config_file = config_dir.join(format!("10-{}.network", interface));

        let mut config = String::new();
        config.push_str(&format!("[Match]\nName={}\n\n", interface));
        config.push_str("[Network]\n");

        if dhcp {
            config.push_str("DHCP=yes\n");
        } else {
            if let Some(ip_addr) = ip {
                config.push_str(&format!("Address={}\n", ip_addr));
            }
            if let Some(gw) = gateway {
                config.push_str(&format!("Gateway={}\n", gw));
            }
            if let Some(dns_servers) = dns {
                for server in dns_servers {
                    config.push_str(&format!("DNS={}\n", server));
                }
            }
        }

        config.push_str("\n[Link]\n");
        config.push_str("RequiredForOnline=yes\n");

        fs::write(config_file, config)?;

        // Reload systemd-networkd
        Command::new("/usr/bin/networkctl").arg("reload").output()?;

        Command::new("/usr/bin/networkctl")
            .args(&["reconfigure", interface])
            .output()?;

        Ok(())
    }

    pub async fn remove_config(&self, interface: &str) -> Result<()> {
        let config_file =
            Path::new("/etc/systemd/network").join(format!("10-{}.network", interface));
        if config_file.exists() {
            fs::remove_file(config_file)?;

            Command::new("/usr/bin/networkctl").arg("reload").output()?;
        }
        Ok(())
    }

    pub async fn create_wifi_config(
        &self,
        interface: &str,
        credentials: &WifiCredentials,
        dhcp: bool,
        ip: Option<String>,
        gateway: Option<String>,
        dns: Option<Vec<String>>,
    ) -> Result<()> {
        // Create wpa_supplicant configuration
        self.create_wpa_supplicant_config(interface, credentials)
            .await?;

        // Create systemd-networkd configuration
        let config_dir = Path::new("/etc/systemd/network");
        if !config_dir.exists() {
            fs::create_dir_all(config_dir)?;
        }

        let config_file = config_dir.join(format!("25-{}.network", interface));

        let mut config = String::new();
        config.push_str(&format!("[Match]\nName={}\n\n", interface));
        config.push_str("[Network]\n");

        if dhcp {
            config.push_str("DHCP=yes\n");
        } else {
            if let Some(ip_addr) = ip {
                config.push_str(&format!("Address={}\n", ip_addr));
            }
            if let Some(gw) = gateway {
                config.push_str(&format!("Gateway={}\n", gw));
            }
            if let Some(dns_servers) = dns {
                for server in dns_servers {
                    config.push_str(&format!("DNS={}\n", server));
                }
            }
        }

        // Add WiFi-specific configuration
        config.push_str("\n[Link]\n");
        config.push_str("RequiredForOnline=yes\n");

        fs::write(config_file, config)?;

        // Reload systemd-networkd
        Command::new("/usr/bin/networkctl").arg("reload").output()?;

        Ok(())
    }

    pub async fn create_enterprise_wifi_config(
        &self,
        interface: &str,
        _credentials: &WifiCredentials,
        dhcp: bool,
        ip: Option<String>,
        gateway: Option<String>,
        dns: Option<Vec<String>>,
    ) -> Result<()> {
        // Create systemd-networkd configuration (same as regular WiFi)
        let config_dir = Path::new("/etc/systemd/network");
        if !config_dir.exists() {
            fs::create_dir_all(config_dir)?;
        }

        let config_file = config_dir.join(format!("25-{}.network", interface));

        let mut config = String::new();
        config.push_str(&format!("[Match]\nName={}\n\n", interface));
        config.push_str("[Network]\n");

        if dhcp {
            config.push_str("DHCP=yes\n");
        } else {
            if let Some(ip_addr) = ip {
                config.push_str(&format!("Address={}\n", ip_addr));
            }
            if let Some(gw) = gateway {
                config.push_str(&format!("Gateway={}\n", gw));
            }
            if let Some(dns_servers) = dns {
                for server in dns_servers {
                    config.push_str(&format!("DNS={}\n", server));
                }
            }
        }

        // Add WiFi-specific configuration for Enterprise
        config.push_str("\n[Link]\n");
        config.push_str("RequiredForOnline=yes\n");

        fs::write(config_file, config)?;

        // Reload systemd-networkd
        Command::new("/usr/bin/networkctl").arg("reload").output()?;

        Ok(())
    }

    async fn create_wpa_supplicant_config(
        &self,
        interface: &str,
        credentials: &WifiCredentials,
    ) -> Result<()> {
        let wpa_dir = Path::new("/etc/wpa_supplicant");
        if !wpa_dir.exists() {
            fs::create_dir_all(wpa_dir)?;
        }

        let wpa_config_file = wpa_dir.join(format!("wpa_supplicant-{}.conf", interface));

        let mut wpa_config = String::new();
        wpa_config.push_str("ctrl_interface=/run/wpa_supplicant\n");
        wpa_config.push_str("update_config=1\n");
        wpa_config.push_str("country=US\n\n");

        // Add network configuration
        wpa_config.push_str("network={\n");
        wpa_config.push_str(&format!("    ssid=\"{}\"\n", credentials.ssid));

        if credentials.hidden {
            wpa_config.push_str("    scan_ssid=1\n");
        }

        match &credentials.security {
            WifiSecurity::Open => {
                wpa_config.push_str("    key_mgmt=NONE\n");
            }
            WifiSecurity::WEP => {
                if let Some(ref password) = credentials.password {
                    wpa_config.push_str(&format!("    wep_key0=\"{}\"\n", password));
                    wpa_config.push_str("    key_mgmt=NONE\n");
                    wpa_config.push_str("    wep_tx_keyidx=0\n");
                }
            }
            WifiSecurity::WPA | WifiSecurity::WPA2 => {
                if let Some(ref password) = credentials.password {
                    wpa_config.push_str(&format!("    psk=\"{}\"\n", password));
                }
                wpa_config.push_str("    key_mgmt=WPA-PSK\n");
            }
            WifiSecurity::WPA3 => {
                if let Some(ref password) = credentials.password {
                    wpa_config.push_str(&format!("    psk=\"{}\"\n", password));
                }
                wpa_config.push_str("    key_mgmt=SAE\n");
                wpa_config.push_str("    ieee80211w=2\n");
            }
            WifiSecurity::Enterprise => {
                // Enterprise configuration handled separately
                return Err(anyhow::anyhow!(
                    "Enterprise WiFi requires separate configuration method"
                ));
            }
        }

        wpa_config.push_str("}\n");

        fs::write(wpa_config_file, wpa_config)?;

        // Enable and start wpa_supplicant for this interface
        Command::new("/usr/bin/systemctl")
            .args(&["enable", &format!("wpa_supplicant@{}.service", interface)])
            .output()?;

        Command::new("/usr/bin/systemctl")
            .args(&["restart", &format!("wpa_supplicant@{}.service", interface)])
            .output()?;

        Ok(())
    }

    pub async fn disconnect_wifi(&self, interface: &str) -> Result<()> {
        // Stop wpa_supplicant
        Command::new("/usr/bin/systemctl")
            .args(&["stop", &format!("wpa_supplicant@{}.service", interface)])
            .output()?;

        Command::new("/usr/bin/systemctl")
            .args(&["disable", &format!("wpa_supplicant@{}.service", interface)])
            .output()?;

        // Remove wpa_supplicant config
        let wpa_config_file =
            Path::new("/etc/wpa_supplicant").join(format!("wpa_supplicant-{}.conf", interface));
        if wpa_config_file.exists() {
            fs::remove_file(wpa_config_file)?;
        }

        // Remove systemd-networkd config
        self.remove_config(interface).await?;

        Ok(())
    }

    pub async fn create_ipv6_config(
        &self,
        interface: &str,
        ipv6_config: &Ipv6Config,
        dhcp: bool,
        ip: Option<String>,
        gateway: Option<String>,
        dns: Option<Vec<String>>,
    ) -> Result<()> {
        let config_dir = Path::new("/etc/systemd/network");
        if !config_dir.exists() {
            fs::create_dir_all(config_dir)?;
        }

        let config_file = config_dir.join(format!("20-{}.network", interface));

        let mut config = String::new();
        config.push_str(&format!("[Match]\nName={}\n\n", interface));
        config.push_str("[Network]\n");

        // IPv4 configuration (if provided)
        if dhcp {
            config.push_str("DHCP=yes\n");
        } else {
            if let Some(ip_addr) = ip {
                config.push_str(&format!("Address={}\n", ip_addr));
            }
            if let Some(gw) = gateway {
                config.push_str(&format!("Gateway={}\n", gw));
            }
            if let Some(dns_servers) = dns {
                for server in dns_servers {
                    config.push_str(&format!("DNS={}\n", server));
                }
            }
        }

        // IPv6 configuration
        if ipv6_config.enable_ipv6 {
            if ipv6_config.dhcpv6 {
                config.push_str("DHCP=ipv6\n");
            }

            for addr in &ipv6_config.addresses {
                config.push_str(&format!("Address={}\n", addr));
            }

            if let Some(ref gw) = ipv6_config.gateway {
                config.push_str(&format!("Gateway={}\n", gw));
            }

            for dns in &ipv6_config.dns_servers {
                config.push_str(&format!("DNS={}\n", dns));
            }

            // IPv6 specific settings
            config.push_str(&format!("IPv6AcceptRA={}\n", ipv6_config.accept_ra));
            config.push_str(&format!(
                "IPv6PrivacyExtensions={}\n",
                if ipv6_config.privacy_extensions {
                    "yes"
                } else {
                    "no"
                }
            ));
        } else {
            config.push_str("IPv6AcceptRA=no\n");
        }

        config.push_str("\n[Link]\n");
        config.push_str("RequiredForOnline=yes\n");

        fs::write(config_file, config)?;

        // Reload systemd-networkd
        Command::new("/usr/bin/networkctl").arg("reload").output()?;

        Command::new("/usr/bin/networkctl")
            .args(&["reconfigure", interface])
            .output()?;

        Ok(())
    }

    pub async fn configure_ipv6_sysctl(
        &self,
        interface: &str,
        ipv6_config: &Ipv6Config,
    ) -> Result<()> {
        // Configure IPv6 via sysctl for immediate effect
        if ipv6_config.enable_ipv6 {
            // Enable IPv6 on interface
            Command::new("/usr/bin/sysctl")
                .args(&["-w", &format!("net.ipv6.conf.{}.disable_ipv6=0", interface)])
                .output()?;

            // Configure Router Advertisement acceptance
            Command::new("/usr/bin/sysctl")
                .args(&[
                    "-w",
                    &format!(
                        "net.ipv6.conf.{}.accept_ra={}",
                        interface,
                        if ipv6_config.accept_ra { "1" } else { "0" }
                    ),
                ])
                .output()?;

            // Configure privacy extensions
            Command::new("/usr/bin/sysctl")
                .args(&[
                    "-w",
                    &format!(
                        "net.ipv6.conf.{}.use_tempaddr={}",
                        interface,
                        if ipv6_config.privacy_extensions {
                            "2"
                        } else {
                            "0"
                        }
                    ),
                ])
                .output()?;
        } else {
            // Disable IPv6 on interface
            Command::new("/usr/bin/sysctl")
                .args(&["-w", &format!("net.ipv6.conf.{}.disable_ipv6=1", interface)])
                .output()?;
        }

        Ok(())
    }

    pub async fn add_ipv6_address(&self, interface: &str, address: &str) -> Result<()> {
        Command::new("/usr/bin/ip")
            .args(&["-6", "addr", "add", address, "dev", interface])
            .output()?;
        Ok(())
    }

    pub async fn remove_ipv6_address(&self, interface: &str, address: &str) -> Result<()> {
        Command::new("/usr/bin/ip")
            .args(&["-6", "addr", "del", address, "dev", interface])
            .output()?;
        Ok(())
    }

    pub async fn add_ipv6_route(
        &self,
        interface: &str,
        destination: &str,
        gateway: Option<&str>,
    ) -> Result<()> {
        let mut args = vec!["-6", "route", "add", destination, "dev", interface];

        if let Some(gw) = gateway {
            args.extend(&["via", gw]);
        }

        Command::new("/usr/bin/ip").args(&args).output()?;
        Ok(())
    }

    pub async fn remove_ipv6_route(&self, interface: &str, destination: &str) -> Result<()> {
        Command::new("/usr/bin/ip")
            .args(&["-6", "route", "del", destination, "dev", interface])
            .output()?;
        Ok(())
    }

    // WireGuard methods
    pub async fn create_wireguard_config(&self, config: &WireGuardConfig) -> Result<()> {
        // Create the .netdev file for WireGuard interface
        self.create_wireguard_netdev(config).await?;

        // Create the .network file for the interface
        self.create_wireguard_network(config).await?;

        // Reload systemd-networkd
        Command::new("/usr/bin/systemctl")
            .args(&["reload", "systemd-networkd"])
            .output()?;

        Ok(())
    }

    async fn create_wireguard_netdev(&self, config: &WireGuardConfig) -> Result<()> {
        let netdev_dir = Path::new("/etc/systemd/network");
        if !netdev_dir.exists() {
            fs::create_dir_all(netdev_dir)?;
        }

        let netdev_file = netdev_dir.join(format!("50-{}.netdev", config.interface_name));

        let mut netdev_config = String::new();
        netdev_config.push_str(&format!("[NetDev]\nName={}\n", config.interface_name));
        netdev_config.push_str("Kind=wireguard\n");
        netdev_config.push_str("Description=WireGuard tunnel\n\n");

        netdev_config.push_str("[WireGuard]\n");
        netdev_config.push_str(&format!("PrivateKey={}\n", config.private_key));

        if let Some(port) = config.listen_port {
            netdev_config.push_str(&format!("ListenPort={}\n", port));
        }

        // Add peers
        for peer in &config.peers {
            netdev_config.push_str("\n[WireGuardPeer]\n");
            netdev_config.push_str(&format!("PublicKey={}\n", peer.public_key));

            if let Some(ref preshared_key) = peer.preshared_key {
                netdev_config.push_str(&format!("PresharedKey={}\n", preshared_key));
            }

            if let Some(ref endpoint) = peer.endpoint {
                netdev_config.push_str(&format!("Endpoint={}\n", endpoint));
            }

            for allowed_ip in &peer.allowed_ips {
                netdev_config.push_str(&format!("AllowedIPs={}\n", allowed_ip));
            }

            if let Some(keepalive) = peer.persistent_keepalive {
                netdev_config.push_str(&format!("PersistentKeepalive={}\n", keepalive));
            }
        }

        fs::write(netdev_file, netdev_config)?;
        Ok(())
    }

    async fn create_wireguard_network(&self, config: &WireGuardConfig) -> Result<()> {
        let network_dir = Path::new("/etc/systemd/network");
        let network_file = network_dir.join(format!("50-{}.network", config.interface_name));

        let mut network_config = String::new();
        network_config.push_str(&format!("[Match]\nName={}\n\n", config.interface_name));
        network_config.push_str("[Network]\n");

        // Add IP addresses
        for address in &config.addresses {
            network_config.push_str(&format!("Address={}\n", address));
        }

        // Add DNS servers
        for dns in &config.dns {
            network_config.push_str(&format!("DNS={}\n", dns));
        }

        // Add routes for allowed IPs
        for peer in &config.peers {
            for allowed_ip in &peer.allowed_ips {
                if allowed_ip != "0.0.0.0/0" && allowed_ip != "::/0" {
                    network_config.push_str(&format!("Route={}\n", allowed_ip));
                }
            }
        }

        network_config.push_str("\n[Link]\n");
        network_config.push_str("RequiredForOnline=no\n");

        if let Some(mtu) = config.mtu {
            network_config.push_str(&format!("MTUBytes={}\n", mtu));
        }

        fs::write(network_file, network_config)?;
        Ok(())
    }

    pub async fn remove_wireguard_config(&self, interface_name: &str) -> Result<()> {
        let network_dir = Path::new("/etc/systemd/network");

        // Remove .netdev file
        let netdev_file = network_dir.join(format!("50-{}.netdev", interface_name));
        if netdev_file.exists() {
            fs::remove_file(netdev_file)?;
        }

        // Remove .network file
        let network_file = network_dir.join(format!("50-{}.network", interface_name));
        if network_file.exists() {
            fs::remove_file(network_file)?;
        }

        // Reload systemd-networkd
        Command::new("/usr/bin/systemctl")
            .args(&["reload", "systemd-networkd"])
            .output()?;

        Ok(())
    }

    pub async fn create_wireguard_from_config_file(
        &self,
        config_path: &str,
        interface_name: &str,
    ) -> Result<()> {
        // Parse existing WireGuard config file and convert to systemd-networkd
        let config_content = fs::read_to_string(config_path)?;
        let config = self.parse_wireguard_config(&config_content, interface_name)?;
        self.create_wireguard_config(&config).await
    }

    fn parse_wireguard_config(
        &self,
        content: &str,
        interface_name: &str,
    ) -> Result<WireGuardConfig> {
        let mut config = WireGuardConfig {
            interface_name: interface_name.to_string(),
            private_key: String::new(),
            public_key: String::new(),
            listen_port: None,
            addresses: vec![],
            dns: vec![],
            mtu: None,
            peers: vec![],
            auto_connect: false,
        };

        let mut current_section = "";
        let mut current_peer = None;

        for line in content.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                if let Some(peer) = current_peer.take() {
                    config.peers.push(peer);
                }
                current_section = &line[1..line.len() - 1];
                if current_section == "Peer" {
                    current_peer = Some(crate::network::WireGuardPeer {
                        public_key: String::new(),
                        preshared_key: None,
                        endpoint: None,
                        allowed_ips: vec![],
                        persistent_keepalive: None,
                        name: None,
                    });
                }
                continue;
            }

            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim();
                let value = line[eq_pos + 1..].trim();

                match current_section {
                    "Interface" => match key {
                        "PrivateKey" => config.private_key = value.to_string(),
                        "ListenPort" => config.listen_port = value.parse().ok(),
                        "Address" => config.addresses.push(value.to_string()),
                        "DNS" => config
                            .dns
                            .extend(value.split(',').map(|s| s.trim().to_string())),
                        "MTU" => config.mtu = value.parse().ok(),
                        _ => {}
                    },
                    "Peer" => {
                        if let Some(ref mut peer) = current_peer {
                            match key {
                                "PublicKey" => peer.public_key = value.to_string(),
                                "PresharedKey" => peer.preshared_key = Some(value.to_string()),
                                "Endpoint" => peer.endpoint = Some(value.to_string()),
                                "AllowedIPs" => peer
                                    .allowed_ips
                                    .extend(value.split(',').map(|s| s.trim().to_string())),
                                "PersistentKeepalive" => {
                                    peer.persistent_keepalive = value.parse().ok()
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if let Some(peer) = current_peer {
            config.peers.push(peer);
        }

        // Generate public key from private key if not set
        if config.public_key.is_empty() && !config.private_key.is_empty() {
            let output = Command::new("/bin/sh")
                .args(&["-c", &format!("echo '{}' | wg pubkey", config.private_key)])
                .output()?;
            config.public_key = String::from_utf8_lossy(&output.stdout).trim().to_string();
        }

        Ok(config)
    }
}

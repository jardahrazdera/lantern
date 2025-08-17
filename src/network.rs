// src/network.rs
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::fs;
use std::path::Path;
use crate::iwd::{IwdManager};

#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("Command '{command}' failed: {details}")]
    CommandFailed { command: String, details: String },
    
    #[error("Interface '{interface}' not found")]
    InterfaceNotFound { interface: String },
    
    #[error("WiFi operation failed: {details}")]
    WiFiError { details: String },
    
    #[error("WireGuard operation failed: {details}")]
    WireGuardError { details: String },
    
    #[error("Invalid network configuration: {details}")]
    InvalidConfig { details: String },
    
    #[error("Permission denied: {operation} requires root privileges")]
    PermissionDenied { operation: String },
    
    #[error("System resource not available: {resource}")]
    ResourceUnavailable { resource: String },

    #[error("Hotspot error: {details}")]
    HotspotError { details: String },
    
    #[error("Enterprise WiFi error: {details}")]
    EnterpriseWiFiError { details: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interface {
    pub name: String,
    pub mac_address: String,
    pub state: String,
    pub mtu: u32,
    pub ipv4_addresses: Vec<String>,
    pub ipv6_addresses: Vec<String>,
    pub ipv6_info: Option<Ipv6Info>,
    pub gateway: Option<String>,
    pub ipv6_gateway: Option<String>,
    pub dns_servers: Vec<String>,
    pub stats: InterfaceStats,
    pub wifi_info: Option<WifiInfo>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InterfaceStats {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub rx_errors: u64,
    pub tx_errors: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiInfo {
    pub current_network: Option<WifiNetwork>,
    pub signal_strength: Option<i32>,
    pub frequency: Option<u32>,
    pub channel: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedWifiInfo {
    pub ssid: String,
    pub bssid: String,
    pub signal_strength: i32,
    pub signal_quality: Option<u32>, // Signal quality percentage
    pub frequency: u32,
    pub channel: u32,
    pub tx_power: Option<i32>,
    pub link_speed: Option<u32>, // Mbps
    pub security: WifiSecurity,
    pub encryption: Vec<String>,
    pub connected_time: Option<std::time::Duration>,
    pub tx_packets: u64,
    pub rx_packets: u64,
    pub tx_bytes: u64,
    pub rx_bytes: u64,
    pub tx_errors: u64,
    pub rx_errors: u64,
    pub tx_dropped: u64,
    pub rx_dropped: u64,
    pub tx_retries: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiNetwork {
    pub ssid: String,
    pub bssid: String,
    pub signal_strength: i32,
    pub frequency: u32,
    pub channel: u32,
    pub security: WifiSecurity,
    pub encryption: Vec<String>,
    pub connected: bool,
    pub in_history: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WifiSecurity {
    Open,
    WEP,
    WPA,
    WPA2,
    WPA3,
    Enterprise,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiCredentials {
    pub ssid: String,
    pub password: Option<String>,
    pub security: WifiSecurity,
    pub hidden: bool,
    pub enterprise: Option<EnterpriseCredentials>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseCredentials {
    pub auth_method: EnterpriseAuthMethod,
    pub username: String,
    pub password: String,
    pub identity: Option<String>,
    pub ca_cert: Option<String>,
    pub client_cert: Option<String>,
    pub private_key: Option<String>,
    pub private_key_password: Option<String>,
    pub phase2_auth: Option<Phase2AuthMethod>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EnterpriseAuthMethod {
    PEAP,
    TTLS,
    TLS,
    PWD,
    LEAP,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Phase2AuthMethod {
    MSCHAPV2,
    PAP,
    CHAP,
    GTC,
    MD5,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ipv6Info {
    pub addresses: Vec<Ipv6Address>,
    pub default_route: Option<String>,
    pub dns_servers: Vec<String>,
    pub accept_ra: bool,
    pub privacy_extensions: bool,
    pub dhcpv6_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ipv6Address {
    pub address: String,
    pub prefix_length: u8,
    pub scope: Ipv6Scope,
    pub flags: Vec<String>,
    pub preferred_lifetime: Option<u32>,
    pub valid_lifetime: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Ipv6Scope {
    Global,
    LinkLocal,
    SiteLocal,
    UniqueLocal,
    Loopback,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ipv6Config {
    pub enable_ipv6: bool,
    pub addresses: Vec<String>,
    pub gateway: Option<String>,
    pub dns_servers: Vec<String>,
    pub accept_ra: bool,
    pub privacy_extensions: bool,
    pub dhcpv6: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireGuardConfig {
    pub interface_name: String,
    pub private_key: String,
    pub public_key: String,
    pub listen_port: Option<u16>,
    pub addresses: Vec<String>,
    pub dns: Vec<String>,
    pub mtu: Option<u16>,
    pub peers: Vec<WireGuardPeer>,
    pub auto_connect: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireGuardPeer {
    pub public_key: String,
    pub preshared_key: Option<String>,
    pub endpoint: Option<String>,
    pub allowed_ips: Vec<String>,
    pub persistent_keepalive: Option<u16>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireGuardStatus {
    pub interface: String,
    pub public_key: String,
    pub listen_port: Option<u16>,
    pub peers: Vec<WireGuardPeerStatus>,
    pub connected: bool,
    pub last_handshake: Option<std::time::SystemTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireGuardPeerStatus {
    pub public_key: String,
    pub endpoint: Option<String>,
    pub allowed_ips: Vec<String>,
    pub latest_handshake: Option<std::time::SystemTime>,
    pub transfer_rx: u64,
    pub transfer_tx: u64,
    pub persistent_keepalive: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireGuardKeyPair {
    pub private_key: String,
    pub public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotspotConfig {
    pub ssid: String,
    pub password: String,
    pub interface: String,
    pub channel: u32,
    pub ip_range: String, // e.g., "192.168.4.0/24"
    pub gateway: String,  // e.g., "192.168.4.1"
}

#[derive(Clone)]
pub struct NetworkManager {
    iwd_manager: IwdManager,
}

impl NetworkManager {
    pub fn new() -> Self {
        Self {
            iwd_manager: IwdManager::new(),
        }
    }

    pub async fn init_iwd(&mut self) -> Result<()> {
        self.iwd_manager.connect().await
    }

    pub async fn get_interfaces(&self) -> Result<Vec<Interface>> {
        let output = Command::new("/usr/bin/ip")
            .args(&["-j", "addr", "show"])
            .output()
            .context("Failed to execute 'ip addr show' command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NetworkError::CommandFailed {
                command: "ip addr show".to_string(),
                details: stderr.to_string(),
            }.into());
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let interfaces_data: Vec<serde_json::Value> = serde_json::from_str(&json_str)
            .context("Failed to parse network interface JSON data")?;

        let mut interfaces = Vec::new();

        for iface_data in interfaces_data {
            // Skip loopback
            if iface_data["ifname"] == "lo" {
                continue;
            }

            let name = iface_data["ifname"].as_str().unwrap_or("").to_string();
            let mac = iface_data["address"].as_str().unwrap_or("N/A").to_string();
            let state = iface_data["operstate"].as_str().unwrap_or("UNKNOWN").to_string();
            let mtu = iface_data["mtu"].as_u64().unwrap_or(1500) as u32;

            let mut ipv4_addresses = Vec::new();
            let mut ipv6_addresses = Vec::new();

            if let Some(addr_info) = iface_data["addr_info"].as_array() {
                for addr in addr_info {
                    let family = addr["family"].as_str().unwrap_or("");
                    let local = addr["local"].as_str().unwrap_or("");
                    let prefixlen = addr["prefixlen"].as_u64().unwrap_or(0);

                    let addr_str = format!("{}/{}", local, prefixlen);

                    match family {
                        "inet" => ipv4_addresses.push(addr_str),
                        "inet6" if !local.starts_with("fe80") => ipv6_addresses.push(addr_str),
                        _ => {}
                    }
                }
            }

            let gateway = self.get_gateway(&name).await?;
            let ipv6_gateway = self.get_ipv6_gateway(&name).await?;
            let dns_servers = self.get_dns_servers().await?;
            let stats = self.get_interface_stats(&name).await?;
            // Skip slow WiFi info gathering at startup - do it lazily when needed
            let wifi_info = if self.is_wireless_interface(&name).await? {
                Some(WifiInfo {
                    current_network: None,
                    signal_strength: None,
                    frequency: None,
                    channel: None,
                })
            } else {
                None
            };
            let ipv6_info = self.get_ipv6_info(&name).await?;

            interfaces.push(Interface {
                name,
                mac_address: mac,
                state,
                mtu,
                ipv4_addresses,
                ipv6_addresses,
                ipv6_info,
                gateway,
                ipv6_gateway,
                dns_servers,
                stats,
                wifi_info,
            });
        }

        Ok(interfaces)
    }

    async fn get_gateway(&self, interface: &str) -> Result<Option<String>> {
        let output = Command::new("/usr/bin/ip")
            .args(&["-j", "route", "show", "default", "dev", interface])
            .output()?;

        let json_str = String::from_utf8_lossy(&output.stdout);
        if json_str.trim().is_empty() {
            return Ok(None);
        }

        let routes: Vec<serde_json::Value> = serde_json::from_str(&json_str)?;
        
        if let Some(route) = routes.first() {
            if let Some(gateway) = route["gateway"].as_str() {
                return Ok(Some(gateway.to_string()));
            }
        }

        Ok(None)
    }

    async fn get_dns_servers(&self) -> Result<Vec<String>> {
        let output = Command::new("/usr/bin/resolvectl")
            .arg("status")
            .output()?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut dns_servers = Vec::new();
        let mut in_global = false;

        for line in output_str.lines() {
            if line.contains("Global") {
                in_global = true;
            } else if line.contains("Link") && line.contains("(") {
                in_global = false;
            } else if in_global && line.contains("DNS Servers:") {
                if let Some(server) = line.split(':').nth(1) {
                    dns_servers.push(server.trim().to_string());
                }
            } else if in_global && !line.contains(':') && !line.trim().is_empty() {
                let trimmed = line.trim();
                if trimmed.parse::<std::net::IpAddr>().is_ok() {
                    dns_servers.push(trimmed.to_string());
                }
            }
        }

        Ok(dns_servers)
    }

    async fn get_interface_stats(&self, interface: &str) -> Result<InterfaceStats> {
        let stats_path = format!("/sys/class/net/{}/statistics", interface);
        
        let mut stats = InterfaceStats::default();

        if Path::new(&stats_path).exists() {
            stats.rx_bytes = fs::read_to_string(format!("{}/rx_bytes", stats_path))
                .unwrap_or_default()
                .trim()
                .parse()
                .unwrap_or(0);
            
            stats.tx_bytes = fs::read_to_string(format!("{}/tx_bytes", stats_path))
                .unwrap_or_default()
                .trim()
                .parse()
                .unwrap_or(0);
            
            stats.rx_packets = fs::read_to_string(format!("{}/rx_packets", stats_path))
                .unwrap_or_default()
                .trim()
                .parse()
                .unwrap_or(0);
            
            stats.tx_packets = fs::read_to_string(format!("{}/tx_packets", stats_path))
                .unwrap_or_default()
                .trim()
                .parse()
                .unwrap_or(0);
            
            stats.rx_errors = fs::read_to_string(format!("{}/rx_errors", stats_path))
                .unwrap_or_default()
                .trim()
                .parse()
                .unwrap_or(0);
            
            stats.tx_errors = fs::read_to_string(format!("{}/tx_errors", stats_path))
                .unwrap_or_default()
                .trim()
                .parse()
                .unwrap_or(0);
        }

        Ok(stats)
    }

    pub async fn set_interface_state(&self, interface: &str, state: &str) -> Result<()> {
        Command::new("/usr/bin/ip")
            .args(&["link", "set", interface, state])
            .output()?;
        Ok(())
    }

    pub async fn add_ip_address(&self, interface: &str, ip_with_prefix: &str) -> Result<()> {
        Command::new("/usr/bin/ip")
            .args(&["addr", "add", ip_with_prefix, "dev", interface])
            .output()?;
        Ok(())
    }

    pub async fn remove_ip_address(&self, interface: &str, ip_with_prefix: &str) -> Result<()> {
        Command::new("/usr/bin/ip")
            .args(&["addr", "del", ip_with_prefix, "dev", interface])
            .output()?;
        Ok(())
    }

    // WiFi-specific methods using systemd-networkd approach
    pub async fn get_wifi_info(&self, interface: &str) -> Result<Option<WifiInfo>> {
        // Check if this is a wireless interface
        if !self.is_wireless_interface(interface).await? {
            return Ok(None);
        }

        let current_network = self.get_current_wifi_network(interface).await?;
        let signal_strength = self.get_signal_strength(interface).await?;
        let (frequency, channel) = self.get_frequency_info(interface).await?;

        Ok(Some(WifiInfo {
            current_network,
            signal_strength,
            frequency,
            channel,
        }))
    }

    pub async fn is_wireless_interface(&self, interface: &str) -> Result<bool> {
        let wireless_path = format!("/sys/class/net/{}/wireless", interface);
        Ok(Path::new(&wireless_path).exists())
    }

    async fn get_current_wifi_network(&self, interface: &str) -> Result<Option<WifiNetwork>> {
        // Try iwd first (modern approach)
        if let Ok(Some(iwd_network)) = self.iwd_manager.get_connected_network(interface).await {
            return Ok(Some(WifiNetwork {
                ssid: iwd_network.name,
                bssid: "Unknown".to_string(), // iwd doesn't expose BSSID easily
                signal_strength: iwd_network.signal_strength as i32,
                frequency: 0, // We'll need to get this separately if needed
                channel: 0,
                connected: iwd_network.connected,
                security: self.parse_iwd_security_type(&iwd_network.security_type),
                encryption: vec![iwd_network.security_type],
                in_history: false, // Will be set later by caller
            }));
        }

        // Fallback to legacy iw method
        let output = match Command::new("/usr/bin/iw")
            .args(&["dev", interface, "link"])
            .output() {
                Ok(output) => output,
                Err(_) => {
                    // Neither iwd nor iw available
                    return Ok(None);
                }
            };

        if !output.status.success() {
            return Ok(None);
        }

        let link_info = String::from_utf8_lossy(&output.stdout);
        
        if link_info.contains("Not connected") {
            return Ok(None);
        }

        self.parse_iw_link_info(&link_info)
    }

    fn parse_iwd_security_type(&self, security_type: &str) -> WifiSecurity {
        match security_type.to_lowercase().as_str() {
            "open" => WifiSecurity::Open,
            "wep" => WifiSecurity::WEP,
            "psk" => WifiSecurity::WPA2,
            "8021x" => WifiSecurity::Enterprise,
            "sae" => WifiSecurity::WPA3,
            _ => WifiSecurity::WPA2, // Default fallback
        }
    }

    fn parse_iw_link_info(&self, link_info: &str) -> Result<Option<WifiNetwork>> {
        let mut ssid = String::new();
        let mut bssid = String::new();
        let mut frequency = 0u32;
        let mut signal = 0i32;

        for line in link_info.lines() {
            let line = line.trim();
            if line.starts_with("SSID:") {
                ssid = line.strip_prefix("SSID:").unwrap_or("").trim().to_string();
            } else if line.starts_with("Connected to") {
                bssid = line.split_whitespace().nth(2).unwrap_or("").to_string();
            } else if line.contains("freq:") {
                if let Some(freq_str) = line.split("freq:").nth(1) {
                    if let Some(freq_part) = freq_str.split_whitespace().next() {
                        frequency = freq_part.parse().unwrap_or(0);
                    }
                }
            } else if line.contains("signal:") {
                if let Some(signal_str) = line.split("signal:").nth(1) {
                    if let Some(signal_part) = signal_str.split_whitespace().next() {
                        signal = signal_part.parse().unwrap_or(0);
                    }
                }
            }
        }

        if !ssid.is_empty() {
            let channel = self.frequency_to_channel(frequency);
            Ok(Some(WifiNetwork {
                ssid,
                bssid,
                signal_strength: signal,
                frequency,
                channel,
                security: WifiSecurity::WPA2, // Will be enhanced with proper detection
                encryption: vec!["WPA2".to_string()],
                connected: false, // This would need to be determined separately
                in_history: false, // Will be set later by caller
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_signal_strength(&self, interface: &str) -> Result<Option<i32>> {
        let output = match Command::new("/usr/bin/iw")
            .args(&["dev", interface, "link"])
            .output() {
                Ok(output) => output,
                Err(_) => return Ok(None),
            };

        if !output.status.success() {
            return Ok(None);
        }

        let link_info = String::from_utf8_lossy(&output.stdout);
        
        for line in link_info.lines() {
            if line.contains("signal:") {
                if let Some(signal_str) = line.split("signal:").nth(1) {
                    if let Some(signal_part) = signal_str.split_whitespace().next() {
                        return Ok(Some(signal_part.parse().unwrap_or(0)));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn get_frequency_info(&self, interface: &str) -> Result<(Option<u32>, Option<u32>)> {
        let output = match Command::new("/usr/bin/iw")
            .args(&["dev", interface, "link"])
            .output() {
                Ok(output) => output,
                Err(_) => return Ok((None, None)),
            };

        if !output.status.success() {
            return Ok((None, None));
        }

        let link_info = String::from_utf8_lossy(&output.stdout);
        
        for line in link_info.lines() {
            if line.contains("freq:") {
                if let Some(freq_str) = line.split("freq:").nth(1) {
                    if let Some(freq_part) = freq_str.split_whitespace().next() {
                        if let Ok(frequency) = freq_part.parse::<u32>() {
                            let channel = self.frequency_to_channel(frequency);
                            return Ok((Some(frequency), Some(channel)));
                        }
                    }
                }
            }
        }

        Ok((None, None))
    }

    pub async fn scan_wifi_networks(&self, interface: &str) -> Result<Vec<WifiNetwork>> {
        // Check if interface exists and is wireless
        if !self.is_wireless_interface(interface).await? {
            return Err(NetworkError::WiFiError {
                details: format!("Interface '{}' is not a wireless interface", interface),
            }.into());
        }

        // Try iwd first (modern approach)
        if let Ok(iwd_networks) = self.iwd_manager.scan_networks(interface).await {
            let mut wifi_networks = Vec::new();
            for iwd_net in iwd_networks {
                wifi_networks.push(WifiNetwork {
                    ssid: iwd_net.name,
                    bssid: "Unknown".to_string(),
                    signal_strength: iwd_net.signal_strength as i32,
                    frequency: 0, // iwd doesn't expose this easily
                    channel: 0,   // Will be calculated from frequency if available
                    connected: iwd_net.connected,
                    security: self.parse_iwd_security_type(&iwd_net.security_type),
                    encryption: vec![iwd_net.security_type],
                    in_history: false, // Will be set later by caller
                });
            }
            return Ok(wifi_networks);
        }

        // Fallback to legacy iw method
        let iw_check = Command::new("/usr/bin/which").args(&["iw"]).output();
        if iw_check.is_err() || !iw_check.unwrap().status.success() {
            return Err(NetworkError::ResourceUnavailable {
                resource: "Neither iwd nor iw wireless tools available".to_string(),
            }.into());
        }

        // Perform WiFi scan with iw
        let output = match Command::new("/usr/bin/iw")
            .args(&["dev", interface, "scan"])
            .output() {
                Ok(output) => output,
                Err(_) => return Ok(Vec::new()),
            };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NetworkError::WiFiError {
                details: format!("WiFi scan failed on '{}': {}", interface, stderr),
            }.into());
        }

        let scan_results = String::from_utf8_lossy(&output.stdout);
        self.parse_wifi_scan_results(&scan_results)
    }

    fn parse_wifi_scan_results(&self, scan_output: &str) -> Result<Vec<WifiNetwork>> {
        let mut networks = Vec::new();
        let mut current_bssid = String::new();
        let mut current_frequency = 0u32;
        let mut current_signal = 0i32;
        let mut current_ssid = String::new();
        let mut current_security = WifiSecurity::Open;
        let mut current_encryption = Vec::new();

        for line in scan_output.lines() {
            let line = line.trim();
            
            if line.starts_with("BSS ") {
                // Save previous network if exists
                if !current_bssid.is_empty() && !current_ssid.is_empty() {
                    let channel = self.frequency_to_channel(current_frequency);
                    networks.push(WifiNetwork {
                        ssid: current_ssid.clone(),
                        bssid: current_bssid.clone(),
                        signal_strength: current_signal,
                        frequency: current_frequency,
                        channel,
                        security: current_security.clone(),
                        encryption: current_encryption.clone(),
                        connected: false, // Legacy scan doesn't provide connection status
                        in_history: false, // Will be set later by caller
                    });
                }

                // Start new network
                current_bssid = line.split_whitespace().nth(1).unwrap_or("").to_string();
                if current_bssid.ends_with('(') {
                    current_bssid.pop();
                }
                current_ssid.clear();
                current_security = WifiSecurity::Open;
                current_encryption.clear();
                current_frequency = 0;
                current_signal = 0;
            } else if line.starts_with("freq:") {
                current_frequency = line.strip_prefix("freq:").unwrap_or("0").trim().parse().unwrap_or(0);
            } else if line.starts_with("signal:") {
                let signal_str = line.strip_prefix("signal:").unwrap_or("0").trim();
                // Parse signal like "-45.00 dBm"
                current_signal = signal_str.split('.').next().unwrap_or("0").parse().unwrap_or(0);
            } else if line.starts_with("SSID:") {
                current_ssid = line.strip_prefix("SSID:").unwrap_or("").trim().to_string();
            } else if line.contains("Privacy") {
                current_security = WifiSecurity::WEP;
                current_encryption.push("WEP".to_string());
            } else if line.contains("WPA2") {
                current_security = WifiSecurity::WPA2;
                current_encryption.push("WPA2".to_string());
            } else if line.contains("WPA3") {
                current_security = WifiSecurity::WPA3;
                current_encryption.push("WPA3".to_string());
            } else if line.contains("WPA:") && !line.contains("WPA2") && !line.contains("WPA3") {
                current_security = WifiSecurity::WPA;
                current_encryption.push("WPA".to_string());
            } else if line.contains("IEEE 802.1X") || line.contains("EAP") || line.contains("Enterprise") {
                current_security = WifiSecurity::Enterprise;
                current_encryption.push("Enterprise".to_string());
            }
        }

        // Don't forget the last network
        if !current_bssid.is_empty() && !current_ssid.is_empty() {
            let channel = self.frequency_to_channel(current_frequency);
            networks.push(WifiNetwork {
                ssid: current_ssid,
                bssid: current_bssid,
                signal_strength: current_signal,
                frequency: current_frequency,
                channel,
                security: current_security,
                encryption: current_encryption,
                connected: false, // Legacy scan doesn't provide connection status
                in_history: false, // Will be set later by caller
            });
        }

        // Remove duplicates and sort by signal strength
        networks.sort_by(|a, b| b.signal_strength.cmp(&a.signal_strength));
        networks.dedup_by(|a, b| a.ssid == b.ssid);

        Ok(networks)
    }

    pub async fn connect_to_wifi(
        &self,
        interface: &str,
        credentials: &WifiCredentials,
        dhcp: bool,
        ip: Option<String>,
        gateway: Option<String>,
        dns: Option<Vec<String>>,
    ) -> Result<()> {
        // Try iwd first (modern approach)
        if let Ok(_) = self.iwd_manager.connect_to_network(
            interface,
            &credentials.ssid,
            credentials.password.as_deref()
        ).await {
            // Connection successful with iwd
            return Ok(());
        }

        // Fallback to legacy wpa_supplicant approach
        // Use systemd-networkd configuration
        let systemd_config = crate::systemd::SystemdNetworkConfig::new();
        systemd_config.create_wifi_config(interface, credentials, dhcp, ip, gateway, dns).await?;
        
        // Restart the interface to apply configuration
        self.set_interface_state(interface, "down").await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        self.set_interface_state(interface, "up").await?;
        
        Ok(())
    }

    pub async fn disconnect_wifi(&self, interface: &str) -> Result<()> {
        // Try iwd first (modern approach)
        if let Ok(_) = self.iwd_manager.disconnect_device(interface).await {
            return Ok(());
        }

        // Fallback to legacy wpa_supplicant approach
        let systemd_config = crate::systemd::SystemdNetworkConfig::new();
        systemd_config.disconnect_wifi(interface).await?;
        
        // Bring interface down
        self.set_interface_state(interface, "down").await?;
        
        Ok(())
    }

    // IPv6-specific methods
    async fn get_ipv6_info(&self, interface: &str) -> Result<Option<Ipv6Info>> {
        let addresses = self.get_detailed_ipv6_addresses(interface).await?;
        let default_route = self.get_ipv6_default_route(interface).await?;
        let dns_servers = self.get_ipv6_dns_servers().await?;
        let (accept_ra, privacy_extensions, dhcpv6_enabled) = self.get_ipv6_settings(interface).await?;

        if addresses.is_empty() {
            return Ok(None);
        }

        Ok(Some(Ipv6Info {
            addresses,
            default_route,
            dns_servers,
            accept_ra,
            privacy_extensions,
            dhcpv6_enabled,
        }))
    }

    async fn get_detailed_ipv6_addresses(&self, interface: &str) -> Result<Vec<Ipv6Address>> {
        let output = Command::new("/usr/bin/ip")
            .args(&["-6", "-j", "addr", "show", interface])
            .output()?;

        if !output.status.success() {
            return Ok(vec![]);
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let interfaces_data: Vec<serde_json::Value> = serde_json::from_str(&json_str)?;

        let mut addresses = Vec::new();
        
        for iface_data in interfaces_data {
            if let Some(addr_info) = iface_data["addr_info"].as_array() {
                for addr in addr_info {
                    if addr["family"] == "inet6" {
                        let address_str = addr["local"].as_str().unwrap_or("").to_string();
                        let prefix_len = addr["prefixlen"].as_u64().unwrap_or(0) as u8;
                        let scope_str = addr["scope"].as_str().unwrap_or("unknown");
                        
                        let scope = match scope_str {
                            "global" => Ipv6Scope::Global,
                            "link" => Ipv6Scope::LinkLocal,
                            "site" => Ipv6Scope::SiteLocal,
                            "host" => Ipv6Scope::Loopback,
                            _ => {
                                if address_str.starts_with("fc") || address_str.starts_with("fd") {
                                    Ipv6Scope::UniqueLocal
                                } else {
                                    Ipv6Scope::Unknown
                                }
                            }
                        };

                        let mut flags = Vec::new();
                        if let Some(flags_array) = addr["flags"].as_array() {
                            for flag in flags_array {
                                if let Some(flag_str) = flag.as_str() {
                                    flags.push(flag_str.to_string());
                                }
                            }
                        }

                        addresses.push(Ipv6Address {
                            address: address_str,
                            prefix_length: prefix_len,
                            scope,
                            flags,
                            preferred_lifetime: addr["preferred_life_time"].as_u64().map(|x| x as u32),
                            valid_lifetime: addr["valid_life_time"].as_u64().map(|x| x as u32),
                        });
                    }
                }
            }
        }

        Ok(addresses)
    }

    async fn get_ipv6_gateway(&self, interface: &str) -> Result<Option<String>> {
        let output = Command::new("/usr/bin/ip")
            .args(&["-6", "route", "show", "default", "dev", interface])
            .output()?;

        if !output.status.success() {
            return Ok(None);
        }

        let route_info = String::from_utf8_lossy(&output.stdout);
        
        for line in route_info.lines() {
            if line.contains("default via") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(pos) = parts.iter().position(|&x| x == "via") {
                    if let Some(gateway) = parts.get(pos + 1) {
                        return Ok(Some(gateway.to_string()));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn get_ipv6_default_route(&self, interface: &str) -> Result<Option<String>> {
        self.get_ipv6_gateway(interface).await
    }

    async fn get_ipv6_dns_servers(&self) -> Result<Vec<String>> {
        // Check systemd-resolved for IPv6 DNS servers
        let output = Command::new("/usr/bin/resolvectl")
            .args(&["status"])
            .output()?;

        if !output.status.success() {
            return Ok(vec![]);
        }

        let resolve_info = String::from_utf8_lossy(&output.stdout);
        let mut dns_servers = Vec::new();

        for line in resolve_info.lines() {
            let line = line.trim();
            if line.starts_with("DNS Servers:") {
                let servers = line.strip_prefix("DNS Servers:").unwrap_or("").trim();
                for server in servers.split_whitespace() {
                    // Check if it's an IPv6 address (contains colons)
                    if server.contains(':') && server.parse::<std::net::Ipv6Addr>().is_ok() {
                        dns_servers.push(server.to_string());
                    }
                }
            }
        }

        Ok(dns_servers)
    }

    async fn get_ipv6_settings(&self, interface: &str) -> Result<(bool, bool, bool)> {
        let mut accept_ra = false;
        let mut privacy_extensions = false;
        let mut dhcpv6_enabled = false;

        // Check accept_ra setting
        if let Ok(content) = fs::read_to_string(format!("/proc/sys/net/ipv6/conf/{}/accept_ra", interface)) {
            accept_ra = content.trim() != "0";
        }

        // Check privacy extensions
        if let Ok(content) = fs::read_to_string(format!("/proc/sys/net/ipv6/conf/{}/use_tempaddr", interface)) {
            privacy_extensions = content.trim() != "0";
        }

        // Check if DHCPv6 is running (simplified check)
        let output = Command::new("/usr/bin/systemctl")
            .args(&["is-active", "dhcpcd"])
            .output();
        
        if let Ok(output) = output {
            dhcpv6_enabled = output.status.success();
        }

        Ok((accept_ra, privacy_extensions, dhcpv6_enabled))
    }

    pub async fn configure_ipv6(
        &self,
        interface: &str,
        config: &Ipv6Config,
    ) -> Result<()> {
        let systemd_config = crate::systemd::SystemdNetworkConfig::new();
        
        // Apply immediate sysctl changes
        systemd_config.configure_ipv6_sysctl(interface, config).await?;
        
        // Create persistent systemd-networkd configuration
        systemd_config.create_ipv6_config(interface, config, false, None, None, None).await?;
        
        // Apply IPv6 addresses immediately
        for address in &config.addresses {
            if let Err(_) = systemd_config.add_ipv6_address(interface, address).await {
                // Address might already exist, continue
            }
        }
        
        Ok(())
    }

    pub async fn add_ipv6_address(&self, interface: &str, address: &str) -> Result<()> {
        let systemd_config = crate::systemd::SystemdNetworkConfig::new();
        systemd_config.add_ipv6_address(interface, address).await
    }

    pub async fn remove_ipv6_address(&self, interface: &str, address: &str) -> Result<()> {
        let systemd_config = crate::systemd::SystemdNetworkConfig::new();
        systemd_config.remove_ipv6_address(interface, address).await
    }

    pub async fn add_ipv6_route(&self, interface: &str, destination: &str, gateway: Option<&str>) -> Result<()> {
        let systemd_config = crate::systemd::SystemdNetworkConfig::new();
        systemd_config.add_ipv6_route(interface, destination, gateway).await
    }

    // WireGuard methods
    pub async fn generate_wireguard_keys(&self) -> Result<WireGuardKeyPair> {
        // Check if WireGuard tools are available
        let wg_check = Command::new("/usr/bin/which").args(&["wg"]).output();
        if wg_check.is_err() || !wg_check.unwrap().status.success() {
            return Err(NetworkError::ResourceUnavailable {
                resource: "WireGuard tools (wg command not found)".to_string(),
            }.into());
        }

        // Generate private key
        let private_output = Command::new("/usr/bin/wg")
            .args(&["genkey"])
            .output()
            .context("Failed to execute 'wg genkey' command")?;

        if !private_output.status.success() {
            let stderr = String::from_utf8_lossy(&private_output.stderr);
            return Err(NetworkError::WireGuardError {
                details: format!("Key generation failed: {}", stderr),
            }.into());
        }

        let private_key = String::from_utf8_lossy(&private_output.stdout).trim().to_string();
        
        if private_key.is_empty() {
            return Err(NetworkError::WireGuardError {
                details: "Generated private key is empty".to_string(),
            }.into());
        }

        // Generate public key from private key using shell pipe
        let public_output = Command::new("/bin/sh")
            .args(&["-c", &format!("echo '{}' | wg pubkey", private_key)])
            .output()
            .context("Failed to generate WireGuard public key")?;

        if !public_output.status.success() {
            let stderr = String::from_utf8_lossy(&public_output.stderr);
            return Err(NetworkError::WireGuardError {
                details: format!("Public key generation failed: {}", stderr),
            }.into());
        }

        let public_key = String::from_utf8_lossy(&public_output.stdout).trim().to_string();
        
        if public_key.is_empty() {
            return Err(NetworkError::WireGuardError {
                details: "Generated public key is empty".to_string(),
            }.into());
        }

        Ok(WireGuardKeyPair {
            private_key,
            public_key,
        })
    }

    pub async fn create_wireguard_interface(&self, config: &WireGuardConfig) -> Result<()> {
        let systemd_config = crate::systemd::SystemdNetworkConfig::new();
        systemd_config.create_wireguard_config(config).await?;
        
        // Bring up the interface
        self.set_interface_state(&config.interface_name, "up").await?;
        
        Ok(())
    }

    pub async fn destroy_wireguard_interface(&self, interface_name: &str) -> Result<()> {
        // Bring down the interface first
        self.set_interface_state(interface_name, "down").await?;
        
        // Remove the interface
        Command::new("/usr/bin/ip")
            .args(&["link", "delete", interface_name])
            .output()?;

        // Remove systemd configuration
        let systemd_config = crate::systemd::SystemdNetworkConfig::new();
        systemd_config.remove_wireguard_config(interface_name).await?;
        
        Ok(())
    }

    pub async fn get_wireguard_status(&self, interface_name: &str) -> Result<Option<WireGuardStatus>> {
        let output = Command::new("/usr/bin/wg")
            .args(&["show", interface_name, "dump"])
            .output()?;

        if !output.status.success() {
            return Ok(None);
        }

        let dump_output = String::from_utf8_lossy(&output.stdout);
        self.parse_wireguard_dump(&dump_output, interface_name)
    }

    fn parse_wireguard_dump(&self, dump_output: &str, interface_name: &str) -> Result<Option<WireGuardStatus>> {
        let lines: Vec<&str> = dump_output.lines().collect();
        
        if lines.is_empty() {
            return Ok(None);
        }

        // First line is the interface
        let interface_line = lines[0];
        let parts: Vec<&str> = interface_line.split('\t').collect();
        
        if parts.len() < 3 {
            return Ok(None);
        }

        let public_key = parts[1].to_string();
        let listen_port = if !parts[2].is_empty() {
            Some(parts[2].parse().unwrap_or(0))
        } else {
            None
        };

        let mut peers = Vec::new();
        
        // Remaining lines are peers
        for line in &lines[1..] {
            let peer_parts: Vec<&str> = line.split('\t').collect();
            if peer_parts.len() >= 4 {
                let peer_public_key = peer_parts[1].to_string();
                let endpoint = if !peer_parts[3].is_empty() {
                    Some(peer_parts[3].to_string())
                } else {
                    None
                };
                
                let allowed_ips: Vec<String> = if !peer_parts[4].is_empty() {
                    peer_parts[4].split(',').map(|s| s.trim().to_string()).collect()
                } else {
                    vec![]
                };

                let latest_handshake = if peer_parts.len() > 5 && !peer_parts[5].is_empty() {
                    peer_parts[5].parse::<u64>().ok()
                        .and_then(|ts| std::time::UNIX_EPOCH.checked_add(std::time::Duration::from_secs(ts)))
                } else {
                    None
                };

                let transfer_rx = if peer_parts.len() > 6 {
                    peer_parts[6].parse().unwrap_or(0)
                } else {
                    0
                };

                let transfer_tx = if peer_parts.len() > 7 {
                    peer_parts[7].parse().unwrap_or(0)
                } else {
                    0
                };

                let persistent_keepalive = if peer_parts.len() > 8 && !peer_parts[8].is_empty() {
                    Some(peer_parts[8].parse().unwrap_or(0))
                } else {
                    None
                };

                peers.push(WireGuardPeerStatus {
                    public_key: peer_public_key,
                    endpoint,
                    allowed_ips,
                    latest_handshake,
                    transfer_rx,
                    transfer_tx,
                    persistent_keepalive,
                });
            }
        }

        let connected = peers.iter().any(|p| p.latest_handshake.is_some());
        let last_handshake = peers.iter()
            .filter_map(|p| p.latest_handshake)
            .max();

        Ok(Some(WireGuardStatus {
            interface: interface_name.to_string(),
            public_key,
            listen_port,
            peers,
            connected,
            last_handshake,
        }))
    }

    pub async fn list_wireguard_interfaces(&self) -> Result<Vec<String>> {
        let output = Command::new("/usr/bin/wg")
            .args(&["show", "interfaces"])
            .output()?;

        if !output.status.success() {
            return Ok(vec![]);
        }

        let interfaces_str = String::from_utf8_lossy(&output.stdout);
        let interfaces: Vec<String> = interfaces_str
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        Ok(interfaces)
    }

    pub async fn connect_wireguard(&self, interface_name: &str) -> Result<()> {
        // WireGuard interfaces auto-connect when brought up if properly configured
        self.set_interface_state(interface_name, "up").await?;
        
        // Give it a moment to establish connection
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        
        Ok(())
    }

    pub async fn disconnect_wireguard(&self, interface_name: &str) -> Result<()> {
        self.set_interface_state(interface_name, "down").await?;
        Ok(())
    }

    // WiFi Hotspot methods
    pub async fn check_internet_connectivity(&self) -> Result<bool> {
        // Check if we can reach a public DNS server
        let result = Command::new("/usr/bin/ping")
            .args(&["-c", "1", "-W", "3", "8.8.8.8"])
            .output()
            .context("Failed to check internet connectivity")?;
        
        Ok(result.status.success())
    }

    pub async fn get_internet_interface(&self) -> Result<Option<String>> {
        // Find interface with default route (internet connection)
        let output = Command::new("/usr/bin/ip")
            .args(&["route", "show", "default"])
            .output()
            .context("Failed to get default route")?;

        let route_output = String::from_utf8_lossy(&output.stdout);
        
        // Parse output like: "default via 192.168.1.1 dev wlan0 proto dhcp src 192.168.1.100 metric 600"
        for line in route_output.lines() {
            if line.contains("default") {
                if let Some(dev_pos) = line.find(" dev ") {
                    let after_dev = &line[dev_pos + 5..];
                    if let Some(space_pos) = after_dev.find(' ') {
                        return Ok(Some(after_dev[..space_pos].to_string()));
                    } else {
                        return Ok(Some(after_dev.to_string()));
                    }
                }
            }
        }
        
        Ok(None)
    }

    pub async fn create_hotspot(&self, config: &HotspotConfig) -> Result<()> {
        // Check prerequisites
        if !self.check_internet_connectivity().await? {
            return Err(NetworkError::HotspotError {
                details: "No internet connection available for hotspot".to_string(),
            }.into());
        }

        let internet_interface = self.get_internet_interface().await?
            .ok_or_else(|| NetworkError::HotspotError {
                details: "No internet interface found".to_string(),
            })?;

        // Check if the WiFi interface is available and not connected
        if let Ok(wifi_info) = self.get_wifi_info(&config.interface).await {
            if wifi_info.is_some() && wifi_info.unwrap().current_network.is_some() {
                return Err(NetworkError::HotspotError {
                    details: "WiFi interface is currently connected to a network".to_string(),
                }.into());
            }
        }

        // Create hostapd configuration
        self.create_hostapd_config(config).await?;
        
        // Configure interface IP
        self.configure_hotspot_interface(config).await?;
        
        // Setup DHCP server
        self.setup_dhcp_server(config).await?;
        
        // Setup NAT/iptables rules
        self.setup_nat_rules(config, &internet_interface).await?;
        
        // Start hostapd
        self.start_hostapd(config).await?;
        
        Ok(())
    }

    async fn create_hostapd_config(&self, config: &HotspotConfig) -> Result<()> {
        let hostapd_config = format!(
            "interface={}\n\
             driver=nl80211\n\
             ssid={}\n\
             hw_mode=g\n\
             channel={}\n\
             wmm_enabled=1\n\
             macaddr_acl=0\n\
             auth_algs=1\n\
             ignore_broadcast_ssid=0\n\
             wpa=2\n\
             wpa_passphrase={}\n\
             wpa_key_mgmt=WPA-PSK\n\
             wpa_pairwise=TKIP\n\
             rsn_pairwise=CCMP\n",
            config.interface,
            config.ssid,
            config.channel,
            config.password
        );

        fs::write("/tmp/hostapd.conf", hostapd_config)
            .context("Failed to write hostapd configuration")?;
        
        Ok(())
    }

    async fn configure_hotspot_interface(&self, config: &HotspotConfig) -> Result<()> {
        // Bring interface down first
        Command::new("/usr/bin/ip")
            .args(&["link", "set", &config.interface, "down"])
            .output()
            .context("Failed to bring interface down")?;

        // Set interface IP address
        Command::new("/usr/bin/ip")
            .args(&["addr", "add", &format!("{}/24", config.gateway), "dev", &config.interface])
            .output()
            .context("Failed to set interface IP")?;

        // Bring interface up
        Command::new("/usr/bin/ip")
            .args(&["link", "set", &config.interface, "up"])
            .output()
            .context("Failed to bring interface up")?;

        Ok(())
    }

    async fn setup_dhcp_server(&self, config: &HotspotConfig) -> Result<()> {
        // Create dnsmasq configuration for DHCP
        let dnsmasq_config = format!(
            "interface={}\n\
             dhcp-range={}.10,{}.50,255.255.255.0,24h\n\
             dhcp-option=3,{}\n\
             dhcp-option=6,8.8.8.8,8.8.4.4\n\
             server=8.8.8.8\n\
             log-queries\n\
             log-dhcp\n\
             listen-address={}\n",
            config.interface,
            &config.gateway[..config.gateway.rfind('.').unwrap()], // Get network part
            &config.gateway[..config.gateway.rfind('.').unwrap()],
            config.gateway,
            config.gateway
        );

        fs::write("/tmp/dnsmasq.conf", dnsmasq_config)
            .context("Failed to write dnsmasq configuration")?;

        // Start dnsmasq
        Command::new("/usr/bin/dnsmasq")
            .args(&["-C", "/tmp/dnsmasq.conf", "-d"])
            .spawn()
            .context("Failed to start dnsmasq")?;

        Ok(())
    }

    async fn setup_nat_rules(&self, config: &HotspotConfig, internet_interface: &str) -> Result<()> {
        // Enable IP forwarding
        Command::new("/usr/bin/sysctl")
            .args(&["-w", "net.ipv4.ip_forward=1"])
            .output()
            .context("Failed to enable IP forwarding")?;

        // Setup NAT rules
        Command::new("/usr/bin/iptables")
            .args(&["-t", "nat", "-A", "POSTROUTING", "-o", internet_interface, "-j", "MASQUERADE"])
            .output()
            .context("Failed to setup NAT rule")?;

        Command::new("/usr/bin/iptables")
            .args(&["-A", "FORWARD", "-i", internet_interface, "-o", &config.interface, "-m", "state", "--state", "RELATED,ESTABLISHED", "-j", "ACCEPT"])
            .output()
            .context("Failed to setup forward rule 1")?;

        Command::new("/usr/bin/iptables")
            .args(&["-A", "FORWARD", "-i", &config.interface, "-o", internet_interface, "-j", "ACCEPT"])
            .output()
            .context("Failed to setup forward rule 2")?;

        Ok(())
    }

    async fn start_hostapd(&self, _config: &HotspotConfig) -> Result<()> {
        Command::new("/usr/bin/hostapd")
            .args(&["/tmp/hostapd.conf", "-B"]) // -B for background mode
            .output()
            .context("Failed to start hostapd")?;

        Ok(())
    }

    pub async fn stop_hotspot(&self, config: &HotspotConfig) -> Result<()> {
        // Stop hostapd
        Command::new("/usr/bin/pkill")
            .args(&["hostapd"])
            .output()
            .ok(); // Don't fail if not running

        // Stop dnsmasq
        Command::new("/usr/bin/pkill")
            .args(&["dnsmasq"])
            .output()
            .ok(); // Don't fail if not running

        // Remove iptables rules
        Command::new("/usr/bin/iptables")
            .args(&["-F"])
            .output()
            .ok();

        Command::new("/usr/bin/iptables")
            .args(&["-t", "nat", "-F"])
            .output()
            .ok();

        // Reset interface
        Command::new("/usr/bin/ip")
            .args(&["addr", "flush", "dev", &config.interface])
            .output()
            .context("Failed to flush interface addresses")?;

        Command::new("/usr/bin/ip")
            .args(&["link", "set", &config.interface, "down"])
            .output()
            .context("Failed to bring interface down")?;

        Ok(())
    }

    fn frequency_to_channel(&self, frequency: u32) -> u32 {
        // Convert frequency to WiFi channel
        match frequency {
            2412..=2484 => (frequency - 2412) / 5 + 1,  // 2.4 GHz band
            5000..=6000 => (frequency - 5000) / 5,       // 5 GHz band  
            _ => 0,
        }
    }

    /// Update only statistics for existing interfaces (optimized for frequent polling)
    pub async fn update_interface_stats(&self, interfaces: &mut [Interface]) -> Result<()> {
        for interface in interfaces {
            interface.stats = self.get_interface_stats(&interface.name).await?;
            
            // Note: WiFi info updates are too slow for stats refresh
            // WiFi info should be updated separately and less frequently
        }
        Ok(())
    }
    
    pub async fn get_detailed_wifi_info(&self, interface: &str) -> Result<Option<DetailedWifiInfo>> {
        // Get basic WiFi info first
        let wifi_info = self.get_wifi_info(interface).await?;
        if wifi_info.is_none() {
            return Ok(None);
        }
        
        let wifi_info = wifi_info.unwrap();
        if let Some(current_network) = wifi_info.current_network {
            // Get detailed interface statistics
            let stats = self.get_interface_stats(interface).await?;
            
            // Get additional WiFi-specific information using iwconfig or iw
            let (link_speed, tx_power, signal_quality) = self.get_wifi_link_details(interface).await?;
            
            // Get connection time by checking when the interface came up
            let connected_time = self.get_connection_uptime(interface).await?;
            
            Ok(Some(DetailedWifiInfo {
                ssid: current_network.ssid,
                bssid: current_network.bssid,
                signal_strength: current_network.signal_strength,
                signal_quality,
                frequency: current_network.frequency,
                channel: current_network.channel,
                tx_power,
                link_speed,
                security: current_network.security,
                encryption: current_network.encryption,
                connected_time,
                tx_packets: stats.tx_packets,
                rx_packets: stats.rx_packets,
                tx_bytes: stats.tx_bytes,
                rx_bytes: stats.rx_bytes,
                tx_errors: stats.tx_errors,
                rx_errors: stats.rx_errors,
                tx_dropped: 0, // Will be populated by get_wifi_link_details
                rx_dropped: 0, // Will be populated by get_wifi_link_details
                tx_retries: 0, // Will be populated by get_wifi_link_details
            }))
        } else {
            Ok(None)
        }
    }
    
    async fn get_wifi_link_details(&self, interface: &str) -> Result<(Option<u32>, Option<i32>, Option<u32>)> {
        // Try to get link details using iw command
        let output = Command::new("/usr/bin/iw")
            .args(&["dev", interface, "link"])
            .output();
            
        if let Ok(output) = output {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let mut link_speed = None;
                let mut tx_power = None;
                let mut signal_quality = None;
                
                for line in output_str.lines() {
                    let line = line.trim();
                    
                    // Parse tx bitrate: "tx bitrate: 144.4 MBit/s"
                    if line.starts_with("tx bitrate:") {
                        if let Some(speed_str) = line.split_whitespace().nth(2) {
                            if let Ok(speed) = speed_str.parse::<f32>() {
                                link_speed = Some(speed as u32);
                            }
                        }
                    }
                    
                    // Parse signal strength to quality percentage
                    if line.starts_with("signal:") {
                        if let Some(signal_str) = line.split_whitespace().nth(1) {
                            if let Ok(signal) = signal_str.parse::<i32>() {
                                // Convert signal strength to quality percentage
                                // -30 dBm = 100%, -90 dBm = 0%
                                let quality = ((signal + 90) * 100 / 60).max(0).min(100) as u32;
                                signal_quality = Some(quality);
                            }
                        }
                    }
                }
                
                return Ok((link_speed, tx_power, signal_quality));
            }
        }
        
        // Fallback: try iwconfig
        let output = Command::new("/usr/bin/iwconfig")
            .arg(interface)
            .output();
            
        if let Ok(output) = output {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let mut link_speed = None;
                let mut tx_power = None;
                let mut signal_quality = None;
                
                for line in output_str.lines() {
                    let line = line.trim();
                    
                    // Parse bit rate: "Bit Rate=54 Mb/s"
                    if line.contains("Bit Rate=") {
                        if let Some(rate_part) = line.split("Bit Rate=").nth(1) {
                            if let Some(speed_str) = rate_part.split_whitespace().next() {
                                if let Ok(speed) = speed_str.parse::<u32>() {
                                    link_speed = Some(speed);
                                }
                            }
                        }
                    }
                    
                    // Parse Tx-Power: "Tx-Power=20 dBm"
                    if line.contains("Tx-Power=") {
                        if let Some(power_part) = line.split("Tx-Power=").nth(1) {
                            if let Some(power_str) = power_part.split_whitespace().next() {
                                if let Ok(power) = power_str.parse::<i32>() {
                                    tx_power = Some(power);
                                }
                            }
                        }
                    }
                    
                    // Parse Link Quality: "Link Quality=60/70"
                    if line.contains("Link Quality=") {
                        if let Some(quality_part) = line.split("Link Quality=").nth(1) {
                            if let Some(quality_str) = quality_part.split_whitespace().next() {
                                if let Some(numerator_str) = quality_str.split('/').next() {
                                    if let Some(denominator_str) = quality_str.split('/').nth(1) {
                                        if let (Ok(num), Ok(den)) = (numerator_str.parse::<u32>(), denominator_str.parse::<u32>()) {
                                            if den > 0 {
                                                signal_quality = Some((num * 100) / den);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                return Ok((link_speed, tx_power, signal_quality));
            }
        }
        
        Ok((None, None, None))
    }
    
    async fn get_connection_uptime(&self, interface: &str) -> Result<Option<std::time::Duration>> {
        // Check /proc/net/wireless or /sys/class/net/{interface}/operstate timestamp
        let operstate_path = format!("/sys/class/net/{}/operstate", interface);
        
        if let Ok(metadata) = std::fs::metadata(&operstate_path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = modified.elapsed() {
                    return Ok(Some(duration));
                }
            }
        }
        
        Ok(None)
    }
}

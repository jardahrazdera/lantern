// src/iwd.rs - iwd command-line integration for modern WiFi management
#![allow(dead_code)] // Many methods are for future features or CLI mode
#![allow(clippy::needless_borrows_for_generic_args)] // Command args are clearer with explicit borrows
#![allow(clippy::collapsible_if)] // Code clarity over micro-optimizations
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;

// No more fake signal generation - using real iw data only!

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IwdNetwork {
    pub name: String,
    pub signal_strength: i16,
    pub security_type: String,
    pub path: String,
    pub connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IwdDevice {
    pub name: String,
    pub powered: bool,
    pub adapter: String,
    pub mode: String,
    pub scanning: bool,
    pub path: String,
}

#[derive(Clone)]
pub struct IwdManager;

impl IwdManager {
    pub fn new() -> Self {
        Self
    }

    // Parse iw scan output to extract real WiFi network data
    fn parse_iw_scan_output(&self, output: &str) -> Result<Vec<IwdNetwork>> {
        let mut networks = Vec::new();
        let mut _current_bss: Option<String> = None;
        let mut current_ssid: Option<String> = None;
        let mut current_signal: Option<i16> = None;
        let mut current_security = "open".to_string();

        for line in output.lines() {
            let line = line.trim();

            // New BSS entry starts
            if line.starts_with("BSS ") {
                // Save previous network if complete
                if let (Some(ssid), Some(signal)) = (&current_ssid, current_signal) {
                    if !ssid.is_empty() {
                        networks.push(IwdNetwork {
                            name: ssid.clone(),
                            signal_strength: signal,
                            security_type: current_security.clone(),
                            path: format!("/net/connman/iwd/network/{}", ssid),
                            connected: false, // We'll detect this separately
                        });
                    }
                }

                // Reset for new BSS
                _current_bss = Some(line.to_string());
                current_ssid = None;
                current_signal = None;
                current_security = "open".to_string();
            }

            // Signal strength
            if line.starts_with("signal: ") && line.contains("dBm") {
                if let Some(dbm_str) = line.split_whitespace().nth(1) {
                    if let Ok(dbm_float) = dbm_str.parse::<f32>() {
                        current_signal = Some(dbm_float as i16);
                    }
                }
            }

            // SSID
            if line.starts_with("SSID: ") {
                let ssid = line.strip_prefix("SSID: ").unwrap_or("").trim();
                if !ssid.is_empty() {
                    current_ssid = Some(ssid.to_string());
                }
            }

            // Security (detect WPA/WPA2/WPA3)
            if line.contains("RSN:") || line.contains("WPA:") {
                current_security = "psk".to_string();
            }
            if line.contains("Privacy") {
                if current_security == "open" {
                    current_security = "wep".to_string();
                }
            }
        }

        // Save last network
        if let (Some(ssid), Some(signal)) = (&current_ssid, current_signal) {
            if !ssid.is_empty() {
                networks.push(IwdNetwork {
                    name: ssid.clone(),
                    signal_strength: signal,
                    security_type: current_security,
                    path: format!("/net/connman/iwd/network/{}", ssid),
                    connected: false,
                });
            }
        }

        // Sort by signal strength (strongest first)
        networks.sort_by(|a, b| b.signal_strength.cmp(&a.signal_strength));

        Ok(networks)
    }

    // Get real signal strength for a connected network
    pub async fn get_connection_signal(&self, device_name: &str) -> Result<Option<i16>> {
        let output = Command::new("/usr/bin/iwctl")
            .args(&["station", device_name, "show"])
            .output()
            .context("Failed to get station info")?;

        if !output.status.success() {
            return Ok(None);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Look for RSSI line in the output
        for line in stdout.lines() {
            if line.contains("RSSI") && line.contains("dBm") {
                // Parse line like "RSSI                  -54 dBm"
                if let Some(dbm_part) = line.split_whitespace().find(|part| part.ends_with("dBm")) {
                    let dbm_str = dbm_part.trim_end_matches("dBm");
                    if let Ok(dbm) = dbm_str.parse::<i16>() {
                        return Ok(Some(dbm));
                    }
                }
            }
        }

        Ok(None)
    }

    pub async fn connect(&mut self) -> Result<()> {
        // Check if iwctl is available by listing devices
        let output = Command::new("/usr/bin/iwctl")
            .args(&["device", "list"])
            .output()
            .context("Failed to check iwctl availability")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("iwctl command not available"));
        }

        // Check if iwd service is running
        let status = Command::new("/usr/bin/systemctl")
            .args(&["is-active", "iwd"])
            .output()
            .context("Failed to check iwd service status")?;

        if !status.status.success() {
            return Err(anyhow::anyhow!("iwd service is not running"));
        }

        Ok(())
    }

    pub async fn get_devices(&self) -> Result<Vec<IwdDevice>> {
        let output = Command::new("/usr/bin/iwctl")
            .args(&["device", "list"])
            .output()
            .context("Failed to list wireless devices")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to get wireless devices"));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut devices = Vec::new();

        // Parse iwctl device list output
        for line in stdout.lines().skip(4) {
            // Skip header lines
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let name = parts[0].to_string();
                let powered = parts[3] == "on";

                devices.push(IwdDevice {
                    name: name.clone(),
                    powered,
                    adapter: "unknown".to_string(),
                    mode: "station".to_string(),
                    scanning: false,
                    path: format!("/net/connman/iwd/{}", name),
                });
            }
        }

        Ok(devices)
    }

    pub async fn scan_networks(&self, device_name: &str) -> Result<Vec<IwdNetwork>> {
        // Use iw to trigger scan and get results directly
        let scan_output = Command::new("/usr/bin/iw")
            .args(&["dev", device_name, "scan"])
            .output()
            .context("Failed to scan with iw")?;

        if !scan_output.status.success() {
            return Err(anyhow::anyhow!("iw scan failed"));
        }

        // Parse iw scan output directly - no need to wait
        self.parse_iw_scan_output(&String::from_utf8_lossy(&scan_output.stdout))
    }

    pub async fn get_networks(&self, device_name: &str) -> Result<Vec<IwdNetwork>> {
        // Just use scan_networks - same thing but with real data
        self.scan_networks(device_name).await
    }

    pub async fn connect_to_network(
        &self,
        device_name: &str,
        network_name: &str,
        passphrase: Option<&str>,
    ) -> Result<()> {
        let mut cmd = Command::new("/usr/bin/iwctl");
        cmd.args(&["station", device_name, "connect", network_name]);

        if let Some(pass) = passphrase {
            cmd.args(&["--passphrase", pass]);
        }

        let output = cmd.output().context("Failed to connect to WiFi network")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to connect to WiFi: {}", stderr));
        }

        Ok(())
    }

    pub async fn disconnect_device(&self, device_name: &str) -> Result<()> {
        let output = Command::new("/usr/bin/iwctl")
            .args(&["station", device_name, "disconnect"])
            .output()
            .context("Failed to disconnect from WiFi")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "Failed to disconnect from WiFi: {}",
                stderr
            ));
        }

        Ok(())
    }

    pub async fn get_connected_network(&self, device_name: &str) -> Result<Option<IwdNetwork>> {
        let networks = self.get_networks(device_name).await?;
        Ok(networks.into_iter().find(|n| n.connected))
    }

    pub async fn power_device(&self, device_name: &str, powered: bool) -> Result<()> {
        let power_state = if powered { "on" } else { "off" };

        let output = Command::new("/usr/bin/iwctl")
            .args(&[
                "device",
                device_name,
                "set-property",
                "Powered",
                power_state,
            ])
            .output()
            .context("Failed to set device power state")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to set device power: {}", stderr));
        }

        Ok(())
    }
}

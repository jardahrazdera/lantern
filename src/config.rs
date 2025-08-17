// src/config.rs
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use crate::network::{EnterpriseCredentials};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub interface: String,
    pub dhcp: bool,
    pub ip: Option<String>,
    pub gateway: Option<String>,
    pub dns: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiProfile {
    pub ssid: String,
    pub security_type: String,
    pub password: Option<String>,
    pub interface: String,
    pub dhcp: bool,
    pub ip: Option<String>,
    pub gateway: Option<String>,
    pub dns: Option<Vec<String>>,
    pub last_connected: Option<SystemTime>,
    pub auto_connect: bool,
    pub priority: i32, // Higher number = higher priority
    pub enterprise: Option<EnterpriseCredentials>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub profiles: Vec<Profile>,
    pub wifi_profiles: Vec<WifiProfile>,
}

impl Config {
    #[allow(dead_code)]
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if config_path.exists() {
            let content = fs::read_to_string(config_path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self {
                profiles: Vec::new(),
                wifi_profiles: Vec::new(),
            })
        }
    }

    #[allow(dead_code)]
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = toml::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        
        Ok(())
    }

    #[allow(dead_code)]
    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        Ok(config_dir.join("lantern").join("config.toml"))
    }

    #[allow(dead_code)]
    pub fn add_profile(&mut self, profile: Profile) {
        self.profiles.retain(|p| p.name != profile.name);
        self.profiles.push(profile);
    }

    #[allow(dead_code)]
    pub fn get_profile(&self, name: &str) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.name == name)
    }

    pub fn add_wifi_profile(&mut self, profile: WifiProfile) {
        // Remove existing profile for same SSID+interface
        self.wifi_profiles.retain(|p| !(p.ssid == profile.ssid && p.interface == profile.interface));
        self.wifi_profiles.push(profile);
    }

    pub fn get_wifi_profile(&self, ssid: &str, interface: &str) -> Option<&WifiProfile> {
        self.wifi_profiles.iter().find(|p| p.ssid == ssid && p.interface == interface)
    }

    pub fn get_wifi_profiles_by_priority(&self) -> Vec<&WifiProfile> {
        let mut profiles = self.wifi_profiles.iter().collect::<Vec<_>>();
        profiles.sort_by(|a, b| {
            // Sort by auto_connect first, then priority, then last_connected
            match (a.auto_connect, b.auto_connect) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => match b.priority.cmp(&a.priority) {
                    std::cmp::Ordering::Equal => {
                        match (&b.last_connected, &a.last_connected) {
                            (Some(b_time), Some(a_time)) => b_time.cmp(a_time),
                            (Some(_), None) => std::cmp::Ordering::Less,
                            (None, Some(_)) => std::cmp::Ordering::Greater,
                            (None, None) => std::cmp::Ordering::Equal,
                        }
                    }
                    other => other,
                }
            }
        });
        profiles
    }

    pub fn update_wifi_connection(&mut self, ssid: &str, interface: &str) {
        if let Some(profile) = self.wifi_profiles.iter_mut().find(|p| p.ssid == ssid && p.interface == interface) {
            profile.last_connected = Some(SystemTime::now());
        }
    }
}

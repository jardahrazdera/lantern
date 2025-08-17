// src/app.rs
#![allow(dead_code)] // Many methods are for future features or CLI mode
#![allow(clippy::useless_format)] // Format strings may contain dynamic content in future
#![allow(clippy::unnecessary_map_or)] // Code clarity over micro-optimizations
use crate::config::{Config, WifiProfile};
use crate::network::{
    DetailedWifiInfo, EnterpriseAuthMethod, EnterpriseCredentials, Interface, NetworkManager,
    Phase2AuthMethod, WifiCredentials, WifiNetwork, WifiSecurity,
};
use crate::systemd::SystemdNetworkConfig;
use anyhow::Result;
use std::time::{Duration, Instant, SystemTime};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

#[derive(Clone)]
pub struct App {
    pub interfaces: Vec<Interface>,
    pub selected_index: usize,
    pub show_details: bool,
    pub show_edit_dialog: bool,
    pub network_manager: NetworkManager,
    pub systemd_config: SystemdNetworkConfig,
    pub config: Config,
    pub last_refresh: Instant,
    pub last_interface_refresh: Instant,
    pub last_wifi_update: Instant,
    pub last_auto_connect_check: Instant,
    pub status_message: Option<(String, Instant)>,
    pub needs_redraw: bool,

    // Edit dialog state
    pub edit_interface: Option<Interface>,
    pub use_dhcp: bool,
    pub ip_input: Input,
    pub gateway_input: Input,
    pub dns_input: Input,
    pub active_input: usize,

    // WiFi state
    pub show_wifi_dialog: bool,
    pub show_wifi_loading_dialog: bool,
    pub wifi_scan_pending: bool,
    pub wifi_networks: Vec<WifiNetwork>,
    pub selected_wifi_index: usize,
    pub wifi_scanning: bool,
    pub last_wifi_scan: Instant,

    // WiFi connection dialog state
    pub show_wifi_connect_dialog: bool,
    pub selected_wifi_network: Option<WifiNetwork>,
    pub wifi_password_input: Input,
    pub wifi_use_dhcp: bool,
    pub wifi_ip_input: Input,
    pub wifi_gateway_input: Input,
    pub wifi_dns_input: Input,
    pub wifi_active_input: usize,
    pub wifi_hidden_ssid: bool,

    // Enterprise WiFi dialog state
    pub show_wifi_enterprise_dialog: bool,
    pub enterprise_auth_method: EnterpriseAuthMethod,
    pub enterprise_phase2_auth: Option<Phase2AuthMethod>,
    pub enterprise_username_input: Input,
    pub enterprise_password_input: Input,
    pub enterprise_identity_input: Input,
    pub enterprise_ca_cert_input: Input,
    pub enterprise_client_cert_input: Input,
    pub enterprise_private_key_input: Input,
    pub enterprise_key_password_input: Input,
    pub enterprise_active_input: usize,

    // Hotspot dialog state
    pub show_hotspot_dialog: bool,
    pub hotspot_ssid_input: Input,
    pub hotspot_password_input: Input,
    pub hotspot_channel: u32,
    pub hotspot_active_input: usize,

    // WiFi diagnostics dialog state
    pub show_wifi_diagnostics_dialog: bool,
    pub wifi_diagnostics_data: Option<DetailedWifiInfo>,
}

impl App {
    pub async fn new() -> Result<Self> {
        let network_manager = NetworkManager::new();
        let interfaces = network_manager.get_interfaces().await?;
        let config = Config::load().unwrap_or_else(|_| Config {
            profiles: Vec::new(),
            wifi_profiles: Vec::new(),
        });

        Ok(Self {
            interfaces,
            selected_index: 0,
            show_details: false,
            show_edit_dialog: false,
            network_manager,
            systemd_config: SystemdNetworkConfig::new(),
            config,
            last_refresh: Instant::now(),
            last_interface_refresh: Instant::now(),
            last_wifi_update: Instant::now(),
            last_auto_connect_check: Instant::now(),
            status_message: None,
            needs_redraw: true,
            edit_interface: None,
            use_dhcp: false,
            ip_input: Input::default(),
            gateway_input: Input::default(),
            dns_input: Input::default(),
            active_input: 0,

            // WiFi initialization
            show_wifi_dialog: false,
            show_wifi_loading_dialog: false,
            wifi_scan_pending: false,
            wifi_networks: Vec::new(),
            selected_wifi_index: 0,
            wifi_scanning: false,
            last_wifi_scan: Instant::now() - Duration::from_secs(60), // Force initial scan

            // WiFi connection dialog initialization
            show_wifi_connect_dialog: false,
            selected_wifi_network: None,
            wifi_password_input: Input::default(),
            wifi_use_dhcp: true,
            wifi_ip_input: Input::default(),
            wifi_gateway_input: Input::default(),
            wifi_dns_input: Input::default(),
            wifi_active_input: 0,
            wifi_hidden_ssid: false,

            // Enterprise WiFi initialization
            show_wifi_enterprise_dialog: false,
            enterprise_auth_method: EnterpriseAuthMethod::PEAP,
            enterprise_phase2_auth: Some(Phase2AuthMethod::MSCHAPV2),
            enterprise_username_input: Input::default(),
            enterprise_password_input: Input::default(),
            enterprise_identity_input: Input::default(),
            enterprise_ca_cert_input: Input::default(),
            enterprise_client_cert_input: Input::default(),
            enterprise_private_key_input: Input::default(),
            enterprise_key_password_input: Input::default(),
            enterprise_active_input: 0,

            // Hotspot initialization
            show_hotspot_dialog: false,
            hotspot_ssid_input: Input::default().with_value("Lantern-Hotspot".to_string()),
            hotspot_password_input: Input::default().with_value("password123".to_string()),
            hotspot_channel: 6,
            hotspot_active_input: 0,

            // WiFi diagnostics initialization
            show_wifi_diagnostics_dialog: false,
            wifi_diagnostics_data: None,
        })
    }

    pub async fn refresh_interfaces(&mut self) -> Result<()> {
        self.interfaces = self.network_manager.get_interfaces().await?;
        self.last_interface_refresh = Instant::now();
        // Silent refresh for automatic updates
        Ok(())
    }

    pub async fn manual_refresh_interfaces(&mut self) -> Result<()> {
        self.interfaces = self.network_manager.get_interfaces().await?;
        self.last_interface_refresh = Instant::now();
        self.status_message = Some(("Interfaces refreshed".to_string(), Instant::now()));
        Ok(())
    }

    pub fn next(&mut self) {
        if !self.show_edit_dialog && self.selected_index < self.interfaces.len() - 1 {
            self.selected_index += 1;
            self.needs_redraw = true;
        }
    }

    pub fn previous(&mut self) {
        if !self.show_edit_dialog && self.selected_index > 0 {
            self.selected_index -= 1;
            self.needs_redraw = true;
        }
    }

    pub fn toggle_details(&mut self) {
        if !self.show_edit_dialog {
            self.show_details = !self.show_details;
            self.needs_redraw = true;
        }
    }

    pub fn edit_interface(&mut self) {
        if let Some(interface) = self.interfaces.get(self.selected_index) {
            self.edit_interface = Some(interface.clone());
            self.show_edit_dialog = true;

            // Pre-fill current values
            if let Some(ip) = interface.ipv4_addresses.first() {
                self.ip_input = Input::default().with_value(ip.clone());
            }
            if let Some(gateway) = &interface.gateway {
                self.gateway_input = Input::default().with_value(gateway.clone());
            }
            if !interface.dns_servers.is_empty() {
                self.dns_input = Input::default().with_value(interface.dns_servers.join(", "));
            }
        }
    }

    pub fn close_dialog(&mut self) {
        self.show_edit_dialog = false;
        self.edit_interface = None;
        self.ip_input = Input::default();
        self.gateway_input = Input::default();
        self.dns_input = Input::default();
        self.active_input = 0;
    }

    pub fn toggle_dhcp(&mut self) {
        self.use_dhcp = !self.use_dhcp;
    }

    pub fn next_input(&mut self) {
        if !self.use_dhcp {
            self.active_input = (self.active_input + 1) % 3;
        }
    }

    pub fn input_char(&mut self, c: char) {
        match self.active_input {
            0 => {
                self.ip_input.handle_event(&crossterm::event::Event::Key(
                    crossterm::event::KeyEvent::new(
                        crossterm::event::KeyCode::Char(c),
                        crossterm::event::KeyModifiers::empty(),
                    ),
                ));
            }
            1 => {
                self.gateway_input
                    .handle_event(&crossterm::event::Event::Key(
                        crossterm::event::KeyEvent::new(
                            crossterm::event::KeyCode::Char(c),
                            crossterm::event::KeyModifiers::empty(),
                        ),
                    ));
            }
            2 => {
                self.dns_input.handle_event(&crossterm::event::Event::Key(
                    crossterm::event::KeyEvent::new(
                        crossterm::event::KeyCode::Char(c),
                        crossterm::event::KeyModifiers::empty(),
                    ),
                ));
            }
            _ => {}
        }
    }

    pub fn delete_char(&mut self) {
        let backspace_event = crossterm::event::Event::Key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Backspace,
            crossterm::event::KeyModifiers::empty(),
        ));

        match self.active_input {
            0 => {
                self.ip_input.handle_event(&backspace_event);
            }
            1 => {
                self.gateway_input.handle_event(&backspace_event);
            }
            2 => {
                self.dns_input.handle_event(&backspace_event);
            }
            _ => {}
        }
    }

    pub async fn save_configuration(&mut self) -> Result<()> {
        if let Some(interface) = &self.edit_interface {
            let dns_servers: Vec<String> = self
                .dns_input
                .value()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            self.systemd_config
                .create_config(
                    &interface.name,
                    self.use_dhcp,
                    if self.use_dhcp {
                        None
                    } else {
                        Some(self.ip_input.value().to_string())
                    },
                    if self.use_dhcp {
                        None
                    } else {
                        Some(self.gateway_input.value().to_string())
                    },
                    if self.use_dhcp {
                        None
                    } else {
                        Some(dns_servers)
                    },
                )
                .await?;

            self.status_message = Some(("Configuration saved".to_string(), Instant::now()));
            self.close_dialog();
            self.refresh_interfaces().await?;
        }
        Ok(())
    }

    pub async fn toggle_interface_state(&mut self) -> Result<()> {
        if let Some(interface) = self.interfaces.get(self.selected_index) {
            let interface_name = interface.name.clone();
            let new_state = if interface.state == "UP" {
                "down"
            } else {
                "up"
            };
            self.network_manager
                .set_interface_state(&interface_name, new_state)
                .await?;
            self.refresh_interfaces().await?;
            self.status_message = Some((
                format!("Interface {} set to {}", interface_name, new_state),
                Instant::now(),
            ));
        }
        Ok(())
    }

    pub fn should_refresh_stats(&self) -> bool {
        self.last_refresh.elapsed() > Duration::from_secs(1)
    }

    pub fn should_refresh_interfaces(&self) -> bool {
        self.last_interface_refresh.elapsed() > Duration::from_secs(5)
    }

    pub fn should_update_wifi_info(&self) -> bool {
        self.last_wifi_update.elapsed() > Duration::from_secs(10)
    }

    pub fn should_check_auto_connect(&self) -> bool {
        self.last_auto_connect_check.elapsed() > Duration::from_secs(30)
    }

    #[allow(dead_code)]
    pub async fn update_stats(&mut self) -> Result<()> {
        // Only update statistics, not full interface data (performance optimization)
        self.network_manager
            .update_interface_stats(&mut self.interfaces)
            .await?;
        self.last_refresh = Instant::now();
        Ok(())
    }

    pub async fn update_wifi_info(&mut self) -> Result<()> {
        // Update WiFi info for wireless interfaces (less frequent than stats)
        for interface in &mut self.interfaces {
            if interface.wifi_info.is_some() && interface.state == "UP" {
                if let Ok(wifi_info) = self.network_manager.get_wifi_info(&interface.name).await {
                    interface.wifi_info = wifi_info;
                }
            }
        }
        self.last_wifi_update = Instant::now();
        Ok(())
    }

    // Methods to mark when background refresh operations start
    pub fn mark_stats_refresh_started(&mut self) {
        self.last_refresh = Instant::now();
    }

    pub fn mark_interface_refresh_started(&mut self) {
        self.last_interface_refresh = Instant::now();
    }

    pub fn mark_wifi_update_started(&mut self) {
        self.last_wifi_update = Instant::now();
    }

    pub fn mark_auto_connect_check_started(&mut self) {
        self.last_auto_connect_check = Instant::now();
    }

    // Auto-connect functionality
    pub async fn check_auto_connect(&mut self) -> Result<()> {
        // Only auto-connect if no WiFi interface is currently connected
        let has_connected_wifi = self.interfaces.iter().any(|iface| {
            iface.wifi_info.is_some()
                && iface.state == "UP"
                && iface.wifi_info.as_ref().unwrap().current_network.is_some()
        });

        if has_connected_wifi {
            return Ok(()); // Already connected to WiFi
        }

        // Find the first available WiFi interface
        let wifi_interface = self
            .interfaces
            .iter()
            .find(|iface| iface.wifi_info.is_some())
            .map(|iface| iface.name.clone());

        if let Some(interface_name) = wifi_interface {
            // Get auto-connect profiles sorted by priority (clone to avoid borrowing issues)
            let auto_connect_profiles: Vec<_> = self
                .config
                .get_wifi_profiles_by_priority()
                .into_iter()
                .filter(|profile| profile.auto_connect && profile.interface == interface_name)
                .cloned()
                .collect();

            if !auto_connect_profiles.is_empty() {
                // Scan for available networks
                if let Ok(available_networks) = self
                    .network_manager
                    .scan_wifi_networks(&interface_name)
                    .await
                {
                    // Try to connect to the highest priority available network
                    for profile in auto_connect_profiles {
                        if let Some(_network) = available_networks
                            .iter()
                            .find(|net| net.ssid == profile.ssid)
                        {
                            // Attempt auto-connect
                            if let Err(e) = self
                                .auto_connect_to_profile(&profile, &interface_name)
                                .await
                            {
                                eprintln!("Auto-connect failed for {}: {}", profile.ssid, e);
                                continue; // Try next profile
                            } else {
                                self.status_message = Some((
                                    format!("Auto-connected to {}", profile.ssid),
                                    Instant::now(),
                                ));
                                break; // Successfully connected
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn auto_connect_to_profile(
        &mut self,
        profile: &crate::config::WifiProfile,
        interface_name: &str,
    ) -> Result<()> {
        let credentials = crate::network::WifiCredentials {
            ssid: profile.ssid.clone(),
            password: profile.password.clone(),
            security: self.parse_security_type(&profile.security_type),
            hidden: false, // Auto-connect typically for visible networks
            enterprise: profile.enterprise.clone(),
        };

        self.network_manager
            .connect_to_wifi(
                interface_name,
                &credentials,
                profile.dhcp,
                profile.ip.clone(),
                profile.gateway.clone(),
                profile.dns.clone(),
            )
            .await?;

        // Update connection time
        self.config
            .update_wifi_connection(&profile.ssid, interface_name);
        let _ = self.config.save(); // Save updated connection time

        Ok(())
    }

    fn parse_security_type(&self, security_str: &str) -> crate::network::WifiSecurity {
        match security_str {
            "Open" => crate::network::WifiSecurity::Open,
            "WEP" => crate::network::WifiSecurity::WEP,
            "WPA" => crate::network::WifiSecurity::WPA,
            "WPA2" => crate::network::WifiSecurity::WPA2,
            "WPA3" => crate::network::WifiSecurity::WPA3,
            "Enterprise" => crate::network::WifiSecurity::Enterprise,
            _ => crate::network::WifiSecurity::WPA2, // Default fallback
        }
    }

    // Toggle auto-connect for the selected WiFi network
    pub fn toggle_wifi_auto_connect(&mut self) -> Result<()> {
        let interface_name = self.get_selected_interface().map(|i| i.name.clone());
        let network_ssid = self.get_selected_wifi_network().map(|n| n.ssid.clone());

        if let (Some(interface_name), Some(network_ssid)) = (interface_name, network_ssid) {
            if let Some(profile) = self
                .config
                .wifi_profiles
                .iter_mut()
                .find(|p| p.ssid == network_ssid && p.interface == interface_name)
            {
                profile.auto_connect = !profile.auto_connect;
                let enabled = profile.auto_connect;

                if let Err(e) = self.config.save() {
                    eprintln!("Warning: Failed to save auto-connect setting: {}", e);
                }

                self.status_message = Some((
                    format!(
                        "Auto-connect {} for {}",
                        if enabled { "enabled" } else { "disabled" },
                        network_ssid
                    ),
                    Instant::now(),
                ));
            } else {
                self.status_message = Some((
                    "Network not saved - connect first to enable auto-connect".to_string(),
                    Instant::now(),
                ));
            }
        }
        Ok(())
    }

    pub fn get_selected_interface(&self) -> Option<&Interface> {
        self.interfaces.get(self.selected_index)
    }

    pub fn needs_redraw(&self) -> bool {
        self.needs_redraw
            || self
                .status_message
                .as_ref()
                .map_or(false, |(_, time)| time.elapsed().as_secs() < 3)
    }

    pub fn mark_redrawn(&mut self) {
        self.needs_redraw = false;
    }

    // WiFi-specific methods
    pub fn open_wifi_dialog(&mut self) {
        // Show loading dialog immediately for better UX
        self.show_wifi_loading_dialog = true;
        self.wifi_scan_pending = true;
    }

    // This method should be called from the main loop to actually perform the scan
    pub async fn process_wifi_scan_if_pending(&mut self) -> Result<()> {
        if !self.wifi_scan_pending {
            return Ok(());
        }

        self.wifi_scan_pending = false;

        // Try to find and use a WiFi interface automatically
        let wifi_interface = if let Some(interface) = self.get_selected_interface() {
            // First try the selected interface if it has WiFi capability
            if interface.wifi_info.is_some() || self.is_likely_wifi_interface(&interface.name) {
                Some(interface.name.clone())
            } else {
                // Find the first WiFi-capable interface
                self.find_wifi_interface().await
            }
        } else {
            // No interface selected, find any WiFi interface
            self.find_wifi_interface().await
        };

        if let Some(wifi_interface_name) = wifi_interface {
            match self
                .scan_wifi_networks_for_interface(&wifi_interface_name)
                .await
            {
                Ok(_) => {
                    // Hide loading dialog and show results
                    self.show_wifi_loading_dialog = false;
                    self.show_wifi_dialog = true;
                }
                Err(_) => {
                    // Scan failed, hide loading dialog
                    self.show_wifi_loading_dialog = false;
                }
            }
        } else {
            // No WiFi interface found, hide loading dialog
            self.show_wifi_loading_dialog = false;
        }
        Ok(())
    }

    pub fn close_wifi_dialog(&mut self) {
        self.show_wifi_dialog = false;
        self.show_wifi_loading_dialog = false;
        self.wifi_scan_pending = false;
        self.wifi_networks.clear();
        self.selected_wifi_index = 0;
        self.wifi_scanning = false;
    }

    pub async fn scan_wifi_networks(&mut self) -> Result<()> {
        if let Some(interface) = self.get_selected_interface() {
            if interface.wifi_info.is_some() {
                let interface_name = interface.name.clone();
                self.scan_wifi_networks_for_interface(&interface_name)
                    .await?;
            }
        }
        Ok(())
    }

    // Helper method to scan WiFi networks for a specific interface
    pub async fn scan_wifi_networks_for_interface(&mut self, interface_name: &str) -> Result<()> {
        self.wifi_scanning = true;
        self.wifi_networks = self
            .network_manager
            .scan_wifi_networks(interface_name)
            .await?;

        // Populate in_history field for performance optimization
        for network in &mut self.wifi_networks {
            network.in_history = self
                .config
                .get_wifi_profile(&network.ssid, interface_name)
                .is_some();
        }

        self.wifi_scanning = false;
        self.last_wifi_scan = Instant::now();
        self.selected_wifi_index = 0;
        Ok(())
    }

    // Helper method to detect if an interface is likely a WiFi interface based on naming patterns
    fn is_likely_wifi_interface(&self, interface_name: &str) -> bool {
        // Common WiFi interface naming patterns
        interface_name.starts_with("wlan")
            || interface_name.starts_with("wlp")
            || interface_name.starts_with("wifi")
            || interface_name.starts_with("wlx")
            || interface_name.starts_with("wlo")
    }

    // Helper method to find the first available WiFi interface
    async fn find_wifi_interface(&self) -> Option<String> {
        // First check interfaces with wifi_info
        for interface in &self.interfaces {
            if interface.wifi_info.is_some() {
                return Some(interface.name.clone());
            }
        }

        // Then check by naming patterns (this is fast and usually sufficient)
        for interface in &self.interfaces {
            if self.is_likely_wifi_interface(&interface.name) {
                return Some(interface.name.clone());
            }
        }

        // Skip the slow command execution check - naming patterns are enough
        None
    }

    pub fn should_refresh_wifi_scan(&self) -> bool {
        self.show_wifi_dialog && self.last_wifi_scan.elapsed() > Duration::from_secs(30)
    }

    pub fn wifi_navigate_up(&mut self) {
        if self.selected_wifi_index > 0 {
            self.selected_wifi_index -= 1;
        }
    }

    pub fn wifi_navigate_down(&mut self) {
        if self.selected_wifi_index < self.wifi_networks.len().saturating_sub(1) {
            self.selected_wifi_index += 1;
        }
    }

    pub fn get_selected_wifi_network(&self) -> Option<&WifiNetwork> {
        self.wifi_networks.get(self.selected_wifi_index)
    }

    pub fn open_wifi_connect_dialog(&mut self) {
        if let Some(network) = self.get_selected_wifi_network().cloned() {
            self.selected_wifi_network = Some(network.clone());
            self.show_wifi_connect_dialog = true;

            // Check if we have a saved profile for this network
            let saved_profile = if let Some(interface) = self.get_selected_interface() {
                self.config
                    .get_wifi_profile(&network.ssid, &interface.name)
                    .cloned()
            } else {
                None
            };

            if let Some(profile) = saved_profile {
                // Pre-fill with saved credentials and settings
                self.wifi_password_input =
                    Input::default().with_value(profile.password.unwrap_or_default());
                self.wifi_use_dhcp = profile.dhcp;
                self.wifi_ip_input = Input::default().with_value(profile.ip.unwrap_or_default());
                self.wifi_gateway_input =
                    Input::default().with_value(profile.gateway.unwrap_or_default());
                self.wifi_dns_input = Input::default()
                    .with_value(profile.dns.map(|dns| dns.join(", ")).unwrap_or_default());
            } else {
                // Use defaults for new networks
                self.wifi_password_input = Input::default();
                self.wifi_use_dhcp = true;
                self.wifi_ip_input = Input::default();
                self.wifi_gateway_input = Input::default();
                self.wifi_dns_input = Input::default();
            }

            self.wifi_active_input = 0;
        }
    }

    pub fn close_wifi_connect_dialog(&mut self) {
        self.show_wifi_connect_dialog = false;
        self.selected_wifi_network = None;
        self.wifi_password_input = Input::default();
        self.wifi_active_input = 0;
    }

    pub fn wifi_connect_next_input(&mut self) {
        if !self.wifi_use_dhcp {
            self.wifi_active_input = (self.wifi_active_input + 1) % 4; // password, ip, gateway, dns
        } else {
            self.wifi_active_input = 0; // Only password input when using DHCP
        }
    }

    pub fn wifi_connect_toggle_dhcp(&mut self) {
        self.wifi_use_dhcp = !self.wifi_use_dhcp;
        if self.wifi_use_dhcp {
            self.wifi_active_input = 0;
        }
    }

    pub fn wifi_connect_input_char(&mut self, c: char) {
        match self.wifi_active_input {
            0 => {
                // Password
                self.wifi_password_input
                    .handle_event(&crossterm::event::Event::Key(
                        crossterm::event::KeyEvent::new(
                            crossterm::event::KeyCode::Char(c),
                            crossterm::event::KeyModifiers::empty(),
                        ),
                    ));
            }
            1 => {
                // IP
                self.wifi_ip_input
                    .handle_event(&crossterm::event::Event::Key(
                        crossterm::event::KeyEvent::new(
                            crossterm::event::KeyCode::Char(c),
                            crossterm::event::KeyModifiers::empty(),
                        ),
                    ));
            }
            2 => {
                // Gateway
                self.wifi_gateway_input
                    .handle_event(&crossterm::event::Event::Key(
                        crossterm::event::KeyEvent::new(
                            crossterm::event::KeyCode::Char(c),
                            crossterm::event::KeyModifiers::empty(),
                        ),
                    ));
            }
            3 => {
                // DNS
                self.wifi_dns_input
                    .handle_event(&crossterm::event::Event::Key(
                        crossterm::event::KeyEvent::new(
                            crossterm::event::KeyCode::Char(c),
                            crossterm::event::KeyModifiers::empty(),
                        ),
                    ));
            }
            _ => {}
        }
    }

    pub fn wifi_connect_delete_char(&mut self) {
        let backspace_event = crossterm::event::Event::Key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Backspace,
            crossterm::event::KeyModifiers::empty(),
        ));

        match self.wifi_active_input {
            0 => {
                self.wifi_password_input.handle_event(&backspace_event);
            }
            1 => {
                self.wifi_ip_input.handle_event(&backspace_event);
            }
            2 => {
                self.wifi_gateway_input.handle_event(&backspace_event);
            }
            3 => {
                self.wifi_dns_input.handle_event(&backspace_event);
            }
            _ => {}
        }
    }

    pub async fn connect_to_selected_wifi(&mut self) -> Result<()> {
        if let (Some(interface), Some(network)) =
            (self.get_selected_interface(), &self.selected_wifi_network)
        {
            let credentials = WifiCredentials {
                ssid: network.ssid.clone(),
                password: if self.wifi_password_input.value().is_empty() {
                    None
                } else {
                    Some(self.wifi_password_input.value().to_string())
                },
                security: network.security.clone(),
                hidden: self.wifi_hidden_ssid,
                enterprise: None, // Regular WiFi connection doesn't use Enterprise
            };

            let dns_servers = if !self.wifi_use_dhcp && !self.wifi_dns_input.value().is_empty() {
                Some(
                    self.wifi_dns_input
                        .value()
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect(),
                )
            } else {
                None
            };

            // Try to connect to WiFi
            self.network_manager
                .connect_to_wifi(
                    &interface.name,
                    &credentials,
                    self.wifi_use_dhcp,
                    if self.wifi_use_dhcp {
                        None
                    } else {
                        Some(self.wifi_ip_input.value().to_string())
                    },
                    if self.wifi_use_dhcp {
                        None
                    } else {
                        Some(self.wifi_gateway_input.value().to_string())
                    },
                    dns_servers.clone(),
                )
                .await?;

            // Save WiFi profile to history
            let wifi_profile = WifiProfile {
                ssid: network.ssid.clone(),
                security_type: format!("{:?}", network.security),
                password: credentials.password.clone(),
                interface: interface.name.clone(),
                dhcp: self.wifi_use_dhcp,
                ip: if self.wifi_use_dhcp {
                    None
                } else {
                    Some(self.wifi_ip_input.value().to_string())
                },
                gateway: if self.wifi_use_dhcp {
                    None
                } else {
                    Some(self.wifi_gateway_input.value().to_string())
                },
                dns: dns_servers,
                last_connected: Some(SystemTime::now()),
                auto_connect: false, // User can enable this later
                priority: 0,         // Default priority
                enterprise: None,    // Regular WiFi doesn't use Enterprise credentials
            };

            self.config.add_wifi_profile(wifi_profile);

            // Save config to disk
            if let Err(e) = self.config.save() {
                eprintln!("Warning: Failed to save WiFi profile: {}", e);
            }

            self.status_message = Some((
                format!("Connecting to WiFi network: {}", network.ssid),
                Instant::now(),
            ));

            self.close_wifi_connect_dialog();
            self.close_wifi_dialog();
            self.refresh_interfaces().await?;
        }
        Ok(())
    }

    pub async fn disconnect_from_wifi(&mut self) -> Result<()> {
        if let Some(interface) = self.get_selected_interface() {
            if interface.wifi_info.is_some()
                && interface
                    .wifi_info
                    .as_ref()
                    .unwrap()
                    .current_network
                    .is_some()
            {
                self.network_manager
                    .disconnect_wifi(&interface.name)
                    .await?;

                self.status_message =
                    Some((format!("Disconnected from WiFi network"), Instant::now()));

                self.refresh_interfaces().await?;
            }
        }
        Ok(())
    }

    // Enterprise WiFi methods
    pub fn open_wifi_enterprise_dialog(&mut self) {
        if let Some(network) = self.get_selected_wifi_network().cloned() {
            self.selected_wifi_network = Some(network);
            self.show_wifi_enterprise_dialog = true;
            self.enterprise_active_input = 2; // Start with username field
        }
    }

    #[allow(dead_code)]
    pub fn open_wifi_enterprise_dialog_direct(&mut self) {
        self.show_wifi_enterprise_dialog = true;
        self.enterprise_active_input = 2; // Start with username field

        // Reset fields
        self.enterprise_username_input = Input::default();
        self.enterprise_password_input = Input::default();
        self.enterprise_identity_input = Input::default();
        self.enterprise_ca_cert_input = Input::default();
        self.enterprise_client_cert_input = Input::default();
        self.enterprise_private_key_input = Input::default();
        self.enterprise_key_password_input = Input::default();
    }

    pub fn close_wifi_enterprise_dialog(&mut self) {
        self.show_wifi_enterprise_dialog = false;
        self.enterprise_active_input = 2; // Reset to username field
    }

    pub fn enterprise_next_input(&mut self) {
        // Cycle through text inputs only: username(2), password(3), identity(4), ca_cert(5), client_cert(6), private_key(7), key_password(8)
        let max_field = match self.enterprise_auth_method {
            crate::network::EnterpriseAuthMethod::TLS => 8, // All fields available
            _ => 5,                                         // Only up to ca_cert
        };

        self.enterprise_active_input += 1;
        if self.enterprise_active_input > max_field {
            self.enterprise_active_input = 2; // Reset to username
        }
    }

    pub fn enterprise_cycle_auth_method(&mut self) {
        self.enterprise_auth_method = match self.enterprise_auth_method {
            EnterpriseAuthMethod::PEAP => EnterpriseAuthMethod::TTLS,
            EnterpriseAuthMethod::TTLS => EnterpriseAuthMethod::TLS,
            EnterpriseAuthMethod::TLS => EnterpriseAuthMethod::PWD,
            EnterpriseAuthMethod::PWD => EnterpriseAuthMethod::LEAP,
            EnterpriseAuthMethod::LEAP => EnterpriseAuthMethod::PEAP,
        };
    }

    pub fn enterprise_cycle_phase2_auth(&mut self) {
        self.enterprise_phase2_auth = match self.enterprise_phase2_auth {
            None => Some(Phase2AuthMethod::MSCHAPV2),
            Some(Phase2AuthMethod::MSCHAPV2) => Some(Phase2AuthMethod::PAP),
            Some(Phase2AuthMethod::PAP) => Some(Phase2AuthMethod::CHAP),
            Some(Phase2AuthMethod::CHAP) => Some(Phase2AuthMethod::GTC),
            Some(Phase2AuthMethod::GTC) => Some(Phase2AuthMethod::MD5),
            Some(Phase2AuthMethod::MD5) => None,
        };
    }

    pub fn enterprise_input_char(&mut self, c: char) {
        let event = crossterm::event::Event::Key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char(c),
            crossterm::event::KeyModifiers::empty(),
        ));

        match self.enterprise_active_input {
            0 | 1 => {} // Auth method and phase2 are handled by F1/F2 keys
            2 => {
                self.enterprise_username_input.handle_event(&event);
            }
            3 => {
                self.enterprise_password_input.handle_event(&event);
            }
            4 => {
                self.enterprise_identity_input.handle_event(&event);
            }
            5 => {
                self.enterprise_ca_cert_input.handle_event(&event);
            }
            6 => {
                self.enterprise_client_cert_input.handle_event(&event);
            }
            7 => {
                self.enterprise_private_key_input.handle_event(&event);
            }
            8 => {
                self.enterprise_key_password_input.handle_event(&event);
            }
            _ => {}
        }
    }

    pub fn enterprise_delete_char(&mut self) {
        let event = crossterm::event::Event::Key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Backspace,
            crossterm::event::KeyModifiers::empty(),
        ));

        match self.enterprise_active_input {
            0 | 1 => {} // Auth method and phase2 are handled by F1/F2 keys
            2 => {
                self.enterprise_username_input.handle_event(&event);
            }
            3 => {
                self.enterprise_password_input.handle_event(&event);
            }
            4 => {
                self.enterprise_identity_input.handle_event(&event);
            }
            5 => {
                self.enterprise_ca_cert_input.handle_event(&event);
            }
            6 => {
                self.enterprise_client_cert_input.handle_event(&event);
            }
            7 => {
                self.enterprise_private_key_input.handle_event(&event);
            }
            8 => {
                self.enterprise_key_password_input.handle_event(&event);
            }
            _ => {}
        }
    }

    pub async fn connect_to_enterprise_wifi(&mut self) -> Result<()> {
        if let (Some(interface), Some(network)) =
            (self.get_selected_interface(), &self.selected_wifi_network)
        {
            let enterprise_creds = EnterpriseCredentials {
                auth_method: self.enterprise_auth_method.clone(),
                username: self.enterprise_username_input.value().to_string(),
                password: self.enterprise_password_input.value().to_string(),
                identity: if self.enterprise_identity_input.value().is_empty() {
                    None
                } else {
                    Some(self.enterprise_identity_input.value().to_string())
                },
                ca_cert: if self.enterprise_ca_cert_input.value().is_empty() {
                    None
                } else {
                    Some(self.enterprise_ca_cert_input.value().to_string())
                },
                client_cert: if self.enterprise_client_cert_input.value().is_empty() {
                    None
                } else {
                    Some(self.enterprise_client_cert_input.value().to_string())
                },
                private_key: if self.enterprise_private_key_input.value().is_empty() {
                    None
                } else {
                    Some(self.enterprise_private_key_input.value().to_string())
                },
                private_key_password: if self.enterprise_key_password_input.value().is_empty() {
                    None
                } else {
                    Some(self.enterprise_key_password_input.value().to_string())
                },
                phase2_auth: self.enterprise_phase2_auth.clone(),
            };

            let credentials = WifiCredentials {
                ssid: network.ssid.clone(),
                password: None, // Not used for Enterprise
                security: WifiSecurity::Enterprise,
                hidden: self.wifi_hidden_ssid,
                enterprise: Some(enterprise_creds.clone()),
            };

            let dns_servers = if !self.wifi_use_dhcp && !self.wifi_dns_input.value().is_empty() {
                Some(
                    self.wifi_dns_input
                        .value()
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect(),
                )
            } else {
                None
            };

            // Connect to Enterprise WiFi
            self.network_manager
                .connect_to_wifi(
                    &interface.name,
                    &credentials,
                    self.wifi_use_dhcp,
                    if self.wifi_use_dhcp {
                        None
                    } else {
                        Some(self.wifi_ip_input.value().to_string())
                    },
                    if self.wifi_use_dhcp {
                        None
                    } else {
                        Some(self.wifi_gateway_input.value().to_string())
                    },
                    dns_servers.clone(),
                )
                .await?;

            // Save Enterprise WiFi profile to history
            let wifi_profile = crate::config::WifiProfile {
                ssid: network.ssid.clone(),
                security_type: "Enterprise".to_string(),
                password: None, // Not used for Enterprise
                interface: interface.name.clone(),
                dhcp: self.wifi_use_dhcp,
                ip: if self.wifi_use_dhcp {
                    None
                } else {
                    Some(self.wifi_ip_input.value().to_string())
                },
                gateway: if self.wifi_use_dhcp {
                    None
                } else {
                    Some(self.wifi_gateway_input.value().to_string())
                },
                dns: dns_servers,
                last_connected: Some(std::time::SystemTime::now()),
                auto_connect: false, // User can enable this later
                priority: 0,         // Default priority
                enterprise: Some(enterprise_creds.clone()),
            };

            self.config.add_wifi_profile(wifi_profile);

            // Save config to disk
            if let Err(e) = self.config.save() {
                eprintln!("Warning: Failed to save Enterprise WiFi profile: {}", e);
            }

            self.status_message = Some((
                format!("Connecting to Enterprise WiFi: {}", network.ssid),
                Instant::now(),
            ));

            self.close_wifi_enterprise_dialog();
            self.close_wifi_connect_dialog();
            self.close_wifi_dialog();
            self.refresh_interfaces().await?;
        }
        Ok(())
    }

    // Hotspot methods
    pub fn open_hotspot_dialog(&mut self) {
        self.show_hotspot_dialog = true;
        self.hotspot_active_input = 0;
    }

    pub fn close_hotspot_dialog(&mut self) {
        self.show_hotspot_dialog = false;
        self.hotspot_active_input = 0;
    }

    pub fn hotspot_next_input(&mut self) {
        self.hotspot_active_input = (self.hotspot_active_input + 1) % 3; // ssid, password, channel
    }

    pub fn hotspot_cycle_channel(&mut self) {
        // Cycle through common WiFi channels
        self.hotspot_channel = match self.hotspot_channel {
            1 => 6,
            6 => 11,
            11 => 36,
            36 => 44,
            44 => 1,
            _ => 6, // Default
        };
    }

    pub fn hotspot_input_char(&mut self, c: char) {
        let event = crossterm::event::Event::Key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char(c),
            crossterm::event::KeyModifiers::empty(),
        ));

        match self.hotspot_active_input {
            0 => {
                self.hotspot_ssid_input.handle_event(&event);
            }
            1 => {
                self.hotspot_password_input.handle_event(&event);
            }
            2 => {} // Channel is handled by hotspot_cycle_channel
            _ => {}
        }
    }

    pub fn hotspot_delete_char(&mut self) {
        let event = crossterm::event::Event::Key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Backspace,
            crossterm::event::KeyModifiers::empty(),
        ));

        match self.hotspot_active_input {
            0 => {
                self.hotspot_ssid_input.handle_event(&event);
            }
            1 => {
                self.hotspot_password_input.handle_event(&event);
            }
            2 => {} // Channel is handled by hotspot_cycle_channel
            _ => {}
        }
    }

    pub async fn create_hotspot(&mut self) -> Result<()> {
        if let Some(interface) = self.get_selected_interface() {
            // Check if it's a WiFi interface
            if interface.wifi_info.is_none() {
                self.status_message = Some((
                    "Selected interface is not a WiFi interface".to_string(),
                    Instant::now(),
                ));
                return Ok(());
            }

            let hotspot_config = crate::network::HotspotConfig {
                ssid: self.hotspot_ssid_input.value().to_string(),
                password: self.hotspot_password_input.value().to_string(),
                interface: interface.name.clone(),
                channel: self.hotspot_channel,
                ip_range: "192.168.4.0/24".to_string(),
                gateway: "192.168.4.1".to_string(),
            };

            match self.network_manager.create_hotspot(&hotspot_config).await {
                Ok(()) => {
                    self.status_message = Some((
                        format!("Hotspot '{}' created successfully", hotspot_config.ssid),
                        Instant::now(),
                    ));
                }
                Err(e) => {
                    self.status_message =
                        Some((format!("Failed to create hotspot: {}", e), Instant::now()));
                }
            }

            self.close_hotspot_dialog();
            self.refresh_interfaces().await?;
        }
        Ok(())
    }

    // WiFi Diagnostics methods
    pub async fn open_wifi_diagnostics_dialog(&mut self) {
        // Fetch diagnostics data when opening the dialog
        self.wifi_diagnostics_data = self.get_detailed_wifi_info().await.unwrap_or(None);
        self.show_wifi_diagnostics_dialog = true;
    }

    pub fn close_wifi_diagnostics_dialog(&mut self) {
        self.show_wifi_diagnostics_dialog = false;
        self.wifi_diagnostics_data = None;
    }

    pub async fn get_detailed_wifi_info(&self) -> Result<Option<DetailedWifiInfo>> {
        if let Some(interface) = self.get_selected_interface() {
            if interface.wifi_info.is_some() {
                return self
                    .network_manager
                    .get_detailed_wifi_info(&interface.name)
                    .await;
            }
        }
        Ok(None)
    }

    pub async fn refresh_wifi_diagnostics(&mut self) {
        if self.show_wifi_diagnostics_dialog {
            self.wifi_diagnostics_data = self.get_detailed_wifi_info().await.unwrap_or(None);
        }
    }
}

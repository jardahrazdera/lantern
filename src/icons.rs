// src/icons.rs - Nerd Font icons for consistent UI
// This module provides Nerd Font icons for better terminal compatibility
#![allow(dead_code)] // Icons are for future UI enhancements

// Network and connectivity icons
pub const WIFI: &str = ""; // nf-fa-wifi
pub const WIFI_LOCK: &str = ""; // nf-fa-lock
pub const WIFI_OPEN: &str = ""; // nf-fa-unlock
pub const ETHERNET: &str = ""; // nf-oct-link
pub const CONNECTED: &str = ""; // nf-fa-check
pub const DISCONNECTED: &str = ""; // nf-fa-times
pub const SELECTED: &str = ""; // nf-fa-chevron_right

// Signal strength bars (using block characters)
pub const SIGNAL_0: &str = "▁▁▁▁"; // Very weak
pub const SIGNAL_1: &str = "▂▁▁▁"; // Poor
pub const SIGNAL_2: &str = "▂▃▁▁"; // Fair
pub const SIGNAL_3: &str = "▂▃▄▁"; // Good
pub const SIGNAL_4: &str = "▂▃▄▅"; // Excellent

// Status and action icons
pub const SCANNING: &str = ""; // nf-fa-search
pub const REFRESH: &str = ""; // nf-fa-refresh
pub const SETTINGS: &str = ""; // nf-fa-cog
pub const UP_ARROW: &str = ""; // nf-fa-arrow_up
pub const DOWN_ARROW: &str = ""; // nf-fa-arrow_down
pub const WARNING: &str = ""; // nf-fa-exclamation_triangle
pub const ERROR: &str = ""; // nf-fa-times_circle
pub const SUCCESS: &str = ""; // nf-fa-check_circle
pub const INFO: &str = ""; // nf-fa-info_circle
pub const HISTORY: &str = ""; // nf-fa-history
pub const AUTO_CONNECT: &str = ""; // nf-fa-refresh
pub const HOTSPOT: &str = ""; // nf-fa-hotspot

// Interface state icons
pub const UP: &str = ""; // nf-fa-arrow_circle_up
pub const DOWN: &str = ""; // nf-fa-arrow_circle_down
pub const UNKNOWN: &str = ""; // nf-fa-question_circle

// Traffic direction icons
pub const RX: &str = ""; // nf-fa-download
pub const TX: &str = ""; // nf-fa-upload

// Application branding
pub const LANTERN: &str = ""; // nf-fa-lightbulb_o
pub const NETWORK: &str = ""; // nf-fa-sitemap

// Security type icons
pub const SECURITY_OPEN: &str = ""; // nf-fa-unlock
pub const SECURITY_WEP: &str = ""; // nf-fa-lock (weak)
pub const SECURITY_WPA: &str = ""; // nf-fa-shield
pub const SECURITY_WPA2: &str = ""; // nf-fa-shield
pub const SECURITY_WPA3: &str = ""; // nf-fa-shield (strongest)
pub const SECURITY_ENTERPRISE: &str = ""; // nf-fa-building (enterprise)

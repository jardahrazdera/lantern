// src/ui.rs
#![allow(clippy::map_clone)] // .map(|x| x.clone()) is clearer than .cloned() in some contexts
#![allow(clippy::option_as_ref_deref)] // Code clarity over micro-optimizations
#![allow(clippy::useless_format)] // Format strings may contain dynamic content in future
use crate::app::App;
use crate::icons;
use byte_unit::Byte;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("{} Lantern", icons::LANTERN),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(" - {} Network Interface Manager", icons::NETWORK)),
    ]))
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);
    f.render_widget(header, chunks[0]);

    // Main content area
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Interface list
    draw_interface_list(f, app, main_chunks[0]);

    // Details or stats
    if app.show_details {
        draw_interface_details(f, app, main_chunks[1]);
    } else {
        draw_interface_stats(f, app, main_chunks[1]);
    }

    // Footer
    draw_footer(f, app, chunks[2]);

    // Edit dialog
    if app.show_edit_dialog {
        draw_edit_dialog(f, app);
    }

    // WiFi loading dialog
    if app.show_wifi_loading_dialog {
        draw_wifi_loading_dialog(f, app);
    }

    // WiFi dialog
    if app.show_wifi_dialog {
        draw_wifi_dialog(f, app);
    }

    // WiFi connect dialog
    if app.show_wifi_connect_dialog {
        draw_wifi_connect_dialog(f, app);
    }

    // Enterprise WiFi dialog
    if app.show_wifi_enterprise_dialog {
        draw_wifi_enterprise_dialog(f, app);
    }

    // Hotspot dialog
    if app.show_hotspot_dialog {
        draw_hotspot_dialog(f, app);
    }

    // WiFi diagnostics dialog
    if app.show_wifi_diagnostics_dialog {
        draw_wifi_diagnostics_dialog(f, app);
    }
}

fn draw_interface_list(f: &mut Frame, app: &App, area: Rect) {
    let interfaces: Vec<ListItem> = app
        .interfaces
        .iter()
        .enumerate()
        .map(|(i, iface)| {
            let (state_icon, state_color) = match iface.state.as_str() {
                "UP" => (icons::UP, Color::Green),
                "DOWN" => (icons::DOWN, Color::Red),
                _ => (icons::UNKNOWN, Color::Yellow),
            };

            let ip = iface
                .ipv4_addresses
                .first()
                .cloned()
                .unwrap_or_else(|| "No IP".to_string());

            // Build WiFi info if available
            let mut content_spans = vec![
                Span::styled(
                    format!("{:<12}", iface.name),
                    if i == app.selected_index {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    },
                ),
                Span::raw(" "),
                Span::styled(
                    format!("{} {:<6}", state_icon, iface.state),
                    Style::default().fg(state_color),
                ),
                Span::raw(" "),
                Span::raw(format!("{:<15}", ip)),
            ];

            // Add WiFi info if this is a wireless interface
            if let Some(wifi_info) = &iface.wifi_info {
                if let Some(network) = &wifi_info.current_network {
                    content_spans.push(Span::styled(
                        format!(" {} {}", icons::WIFI, network.ssid),
                        Style::default().fg(Color::Cyan),
                    ));

                    // Show signal strength if available
                    if let Some(signal) = wifi_info.signal_strength {
                        let signal_color = match signal {
                            s if s > -50 => Color::Green,   // Excellent
                            s if s > -60 => Color::Yellow,  // Good
                            s if s > -70 => Color::Magenta, // Fair
                            _ => Color::Red,                // Poor
                        };
                        content_spans.push(Span::styled(
                            format!(" ({}dBm)", signal),
                            Style::default().fg(signal_color),
                        ));
                    }
                } else if iface.state == "UP" {
                    content_spans.push(Span::styled(
                        format!(" {} <disconnected>", icons::WIFI),
                        Style::default().fg(Color::Gray),
                    ));
                }
            }

            let content = Line::from(content_spans);

            ListItem::new(content)
        })
        .collect();

    let interfaces_list = List::new(interfaces)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("{} Interfaces [↑/↓ to navigate]", icons::ETHERNET)),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_widget(interfaces_list, area);
}

fn draw_interface_details(f: &mut Frame, app: &App, area: Rect) {
    if let Some(interface) = app.get_selected_interface() {
        let mut lines = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&interface.name),
            ]),
            Line::from(vec![
                Span::styled("MAC: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&interface.mac_address),
            ]),
            Line::from(vec![
                Span::styled("State: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(
                    &interface.state,
                    Style::default().fg(if interface.state == "UP" {
                        Color::Green
                    } else {
                        Color::Red
                    }),
                ),
            ]),
            Line::from(vec![
                Span::styled("MTU: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(interface.mtu.to_string()),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "IPv4 Addresses:",
                Style::default().add_modifier(Modifier::BOLD),
            )),
        ];

        for addr in &interface.ipv4_addresses {
            lines.push(Line::from(format!("  • {}", addr)));
        }
        if interface.ipv4_addresses.is_empty() {
            lines.push(Line::from("  None"));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Gateway: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(interface.gateway.as_deref().unwrap_or("None")),
        ]));

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "DNS Servers:",
            Style::default().add_modifier(Modifier::BOLD),
        )));

        if interface.dns_servers.is_empty() {
            lines.push(Line::from("  None"));
        } else {
            for dns in &interface.dns_servers {
                lines.push(Line::from(format!("  • {}", dns)));
            }
        }

        let details = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Interface Details [Enter to toggle]"),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(details, area);
    }
}

fn draw_interface_stats(f: &mut Frame, app: &App, area: Rect) {
    if let Some(interface) = app.get_selected_interface() {
        let rx_bytes =
            Byte::from_u128(interface.stats.rx_bytes as u128).unwrap_or(Byte::from_u64(0));
        let tx_bytes =
            Byte::from_u128(interface.stats.tx_bytes as u128).unwrap_or(Byte::from_u64(0));

        let stats_text = vec![
            Line::from(Span::styled(
                "Network Statistics",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    format!("{} RX: ", icons::RX),
                    Style::default().fg(Color::Green),
                ),
                Span::raw(format!(
                    "{:.2}",
                    rx_bytes.get_appropriate_unit(byte_unit::UnitType::Binary)
                )),
            ]),
            Line::from(vec![
                Span::raw("  Packets: "),
                Span::raw(interface.stats.rx_packets.to_string()),
            ]),
            Line::from(vec![
                Span::raw("  Errors: "),
                Span::raw(interface.stats.rx_errors.to_string()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    format!("{} TX: ", icons::TX),
                    Style::default().fg(Color::Blue),
                ),
                Span::raw(format!(
                    "{:.2}",
                    tx_bytes.get_appropriate_unit(byte_unit::UnitType::Binary)
                )),
            ]),
            Line::from(vec![
                Span::raw("  Packets: "),
                Span::raw(interface.stats.tx_packets.to_string()),
            ]),
            Line::from(vec![
                Span::raw("  Errors: "),
                Span::raw(interface.stats.tx_errors.to_string()),
            ]),
        ];

        let stats = Paragraph::new(stats_text).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Statistics [Enter for details]"),
        );

        f.render_widget(stats, area);
    }
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let mut footer_text = vec![Span::raw(
        "q: Quit | r: Refresh | e: Edit | u: Up/Down iface | w: WiFi | h: Hotspot | Enter: Details",
    )];

    if let Some((msg, time)) = &app.status_message {
        if time.elapsed().as_secs() < 3 {
            footer_text.push(Span::raw(" | "));
            footer_text.push(Span::styled(msg, Style::default().fg(Color::Yellow)));
        }
    }

    let footer = Paragraph::new(Line::from(footer_text))
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);

    f.render_widget(footer, area);
}

fn draw_edit_dialog(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 50, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(
            "Edit {}",
            app.edit_interface.as_ref().unwrap().name
        ))
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(inner);

    // DHCP toggle
    let dhcp_text = if app.use_dhcp {
        format!(
            "DHCP: [{}] Enabled (press Space to toggle)",
            icons::CONNECTED
        )
    } else {
        "DHCP: [ ] Disabled (press Space to toggle)".to_string()
    };
    let dhcp = Paragraph::new(dhcp_text);
    f.render_widget(dhcp, chunks[0]);

    if !app.use_dhcp {
        // IP input
        let ip_style = if app.active_input == 0 {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let ip = Paragraph::new(app.ip_input.value()).block(
            Block::default()
                .borders(Borders::ALL)
                .title("IP Address")
                .border_style(ip_style),
        );
        f.render_widget(ip, chunks[1]);

        // Gateway input
        let gw_style = if app.active_input == 1 {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let gateway = Paragraph::new(app.gateway_input.value()).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Gateway")
                .border_style(gw_style),
        );
        f.render_widget(gateway, chunks[2]);

        // DNS input
        let dns_style = if app.active_input == 2 {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let dns = Paragraph::new(app.dns_input.value()).block(
            Block::default()
                .borders(Borders::ALL)
                .title("DNS Servers (comma separated)")
                .border_style(dns_style),
        );
        f.render_widget(dns, chunks[3]);
    }

    // Instructions
    let instructions =
        Paragraph::new("Tab: Next field | Space: Toggle DHCP | s: Save | Esc: Cancel")
            .alignment(Alignment::Center);
    f.render_widget(instructions, chunks[5]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn draw_wifi_dialog(f: &mut Frame, app: &App) {
    let area = centered_rect(85, 75, f.area());
    f.render_widget(Clear, area);

    let mut networks: Vec<ListItem> = Vec::new();

    if app.wifi_scanning {
        networks.push(ListItem::new(format!(
            "{} Scanning for networks...",
            icons::SCANNING
        )));
    } else if app.wifi_networks.is_empty() {
        networks.push(ListItem::new(format!(
            "No networks found. Press 'r' to scan. {}",
            icons::REFRESH
        )));
    } else {
        for (i, network) in app.wifi_networks.iter().enumerate() {
            let signal_bars = match network.signal_strength {
                s if s > -50 => icons::SIGNAL_4,
                s if s > -60 => icons::SIGNAL_3,
                s if s > -70 => icons::SIGNAL_2,
                s if s > -80 => icons::SIGNAL_1,
                _ => icons::SIGNAL_0,
            };

            let security_icon = match network.security {
                crate::network::WifiSecurity::Open => icons::SECURITY_OPEN,
                crate::network::WifiSecurity::WEP => icons::SECURITY_WEP,
                crate::network::WifiSecurity::WPA => icons::SECURITY_WPA,
                crate::network::WifiSecurity::WPA2 => icons::SECURITY_WPA2,
                crate::network::WifiSecurity::WPA3 => icons::SECURITY_WPA3,
                crate::network::WifiSecurity::Enterprise => icons::SECURITY_ENTERPRISE,
            };

            // Check if this network is in connection history (optimized)
            let in_history = network.in_history;

            // Check auto-connect status for saved networks
            let auto_connect = if in_history {
                if let Some(interface) = app.get_selected_interface() {
                    app.config
                        .get_wifi_profile(&network.ssid, &interface.name)
                        .map(|profile| profile.auto_connect)
                        .unwrap_or(false)
                } else {
                    false
                }
            } else {
                false
            };

            // Show connected status, history, auto-connect, and selection
            let prefix = if network.connected {
                format!("{} ", icons::CONNECTED) // Connected network
            } else if i == app.selected_wifi_index {
                if in_history {
                    if auto_connect {
                        format!(
                            "{}{}{} ",
                            icons::SELECTED,
                            icons::HISTORY,
                            icons::AUTO_CONNECT
                        ) // Selected + saved + auto
                    } else {
                        format!("{}{} ", icons::SELECTED, icons::HISTORY) // Selected + saved
                    }
                } else {
                    format!("{} ", icons::SELECTED) // Selected network
                }
            } else if in_history {
                if auto_connect {
                    format!("{}{} ", icons::HISTORY, icons::AUTO_CONNECT) // Previously connected + auto
                } else {
                    format!("{} ", icons::HISTORY) // Previously connected
                }
            } else {
                "  ".to_string() // Normal network
            };

            let line = format!(
                "{}{} {} {} ({}dBm)",
                prefix, security_icon, network.ssid, signal_bars, network.signal_strength
            );

            let style = if network.connected {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else if i == app.selected_wifi_index {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else if in_history {
                Style::default().fg(Color::Cyan) // Cyan for previously connected
            } else {
                Style::default()
            };

            networks.push(ListItem::new(line).style(style));
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!("{} WiFi Networks [{} = Saved, {} = Auto | a: Auto | e: Enterprise | d: Diagnostics | ↑/↓: Navigate | Enter: Connect | r: Scan | Esc: Close]", 
            icons::WIFI, icons::HISTORY, icons::AUTO_CONNECT))
        .border_style(Style::default().fg(Color::Cyan));

    let wifi_list = List::new(networks)
        .block(block)
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_widget(wifi_list, area);
}

fn draw_wifi_connect_dialog(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 50, f.area());
    f.render_widget(Clear, area);

    if let Some(network) = &app.selected_wifi_network {
        let title = format!("Connect to: {}", network.ssid);

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Password
                Constraint::Length(3), // DHCP
                Constraint::Length(3), // IP (if static)
                Constraint::Length(3), // Gateway (if static)
                Constraint::Length(3), // DNS (if static)
                Constraint::Min(1),    // Instructions
            ])
            .split(area);

        f.render_widget(block, area);

        // Password input
        let password_style = if app.wifi_active_input == 0 {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else {
            Style::default()
        };

        let password_text = if network.security == crate::network::WifiSecurity::Open {
            "No password required"
        } else {
            // Mask password with asterisks for security
            &"*".repeat(app.wifi_password_input.value().len())
        };

        let password_input = Paragraph::new(password_text)
            .block(Block::default().borders(Borders::ALL).title("Password"))
            .style(password_style);
        f.render_widget(password_input, chunks[0]);

        // DHCP toggle
        let dhcp_style = if app.wifi_active_input == 1 {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else {
            Style::default()
        };

        let dhcp_text = if app.wifi_use_dhcp {
            "DHCP (Auto)"
        } else {
            "Static IP"
        };
        let dhcp_input = Paragraph::new(dhcp_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("IP Configuration [Space: Toggle]"),
            )
            .style(dhcp_style);
        f.render_widget(dhcp_input, chunks[1]);

        // Static IP fields (only if not DHCP)
        if !app.wifi_use_dhcp {
            let ip_style = if app.wifi_active_input == 2 {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };
            let ip_input = Paragraph::new(app.wifi_ip_input.value())
                .block(Block::default().borders(Borders::ALL).title("IP Address"))
                .style(ip_style);
            f.render_widget(ip_input, chunks[2]);

            let gateway_style = if app.wifi_active_input == 3 {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };
            let gateway_input = Paragraph::new(app.wifi_gateway_input.value())
                .block(Block::default().borders(Borders::ALL).title("Gateway"))
                .style(gateway_style);
            f.render_widget(gateway_input, chunks[3]);

            let dns_style = if app.wifi_active_input == 4 {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };
            let dns_input = Paragraph::new(app.wifi_dns_input.value())
                .block(Block::default().borders(Borders::ALL).title("DNS"))
                .style(dns_style);
            f.render_widget(dns_input, chunks[4]);
        }

        // Instructions
        let instructions_text = if network.security == crate::network::WifiSecurity::Enterprise {
            "Tab: Next field | Space: Toggle DHCP | Enter: Enterprise Config | Esc: Cancel"
        } else {
            "Tab: Next field | Space: Toggle DHCP | Enter: Connect | Esc: Cancel"
        };
        let instructions = Paragraph::new(instructions_text)
            .wrap(ratatui::widgets::Wrap { trim: true })
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(instructions, chunks[5]);
    }
}

fn draw_wifi_loading_dialog(f: &mut Frame, _app: &App) {
    let area = centered_rect(40, 20, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(format!("{} WiFi", icons::WIFI))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let loading_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("{} ", icons::SCANNING),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw("Loading..."),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Scanning for networks",
            Style::default().fg(Color::Gray),
        )),
    ];

    let loading = Paragraph::new(loading_text)
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(loading, area);
}

fn draw_wifi_enterprise_dialog(f: &mut Frame, app: &App) {
    let area = centered_rect(80, 80, f.area());
    f.render_widget(Clear, area);

    if let Some(network) = &app.selected_wifi_network {
        let title = format!("Enterprise WiFi: {}", network.ssid);

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Auth Method
                Constraint::Length(3), // Phase 2 Auth
                Constraint::Length(3), // Username
                Constraint::Length(3), // Password
                Constraint::Length(3), // Identity
                Constraint::Length(3), // CA Certificate
                Constraint::Length(3), // Client Certificate
                Constraint::Length(3), // Private Key
                Constraint::Length(3), // Private Key Password
                Constraint::Min(1),    // Instructions
            ])
            .split(area);

        f.render_widget(block, area);

        // Auth Method
        let auth_style = if app.enterprise_active_input == 0 {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else {
            Style::default()
        };
        let auth_method_text = format!("{:?}", app.enterprise_auth_method);
        let auth_method = Paragraph::new(auth_method_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Auth Method [1: Cycle]"),
            )
            .style(auth_style);
        f.render_widget(auth_method, chunks[0]);

        // Phase 2 Auth (only for PEAP/TTLS)
        let phase2_style = if app.enterprise_active_input == 1 {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else {
            Style::default()
        };
        let phase2_text = match &app.enterprise_phase2_auth {
            Some(phase2) => format!("{:?}", phase2),
            None => "None".to_string(),
        };
        let phase2_enabled = matches!(
            app.enterprise_auth_method,
            crate::network::EnterpriseAuthMethod::PEAP | crate::network::EnterpriseAuthMethod::TTLS
        );
        let phase2_title = if phase2_enabled {
            "Phase 2 Auth [2: Cycle]"
        } else {
            "Phase 2 Auth (N/A)"
        };
        let phase2_auth = Paragraph::new(phase2_text)
            .block(Block::default().borders(Borders::ALL).title(phase2_title))
            .style(if phase2_enabled {
                phase2_style
            } else {
                Style::default().fg(Color::DarkGray)
            });
        f.render_widget(phase2_auth, chunks[1]);

        // Username
        let username_style = if app.enterprise_active_input == 2 {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else {
            Style::default()
        };
        let username_input = Paragraph::new(app.enterprise_username_input.value())
            .block(Block::default().borders(Borders::ALL).title("Username"))
            .style(username_style);
        f.render_widget(username_input, chunks[2]);

        // Password (masked)
        let password_style = if app.enterprise_active_input == 3 {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else {
            Style::default()
        };
        let password_text = "*".repeat(app.enterprise_password_input.value().len());
        let password_input = Paragraph::new(password_text)
            .block(Block::default().borders(Borders::ALL).title("Password"))
            .style(password_style);
        f.render_widget(password_input, chunks[3]);

        // Identity (optional)
        let identity_style = if app.enterprise_active_input == 4 {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else {
            Style::default()
        };
        let identity_input = Paragraph::new(app.enterprise_identity_input.value())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Identity (optional)"),
            )
            .style(identity_style);
        f.render_widget(identity_input, chunks[4]);

        // CA Certificate
        let ca_cert_style = if app.enterprise_active_input == 5 {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else {
            Style::default()
        };
        let ca_cert_input = Paragraph::new(app.enterprise_ca_cert_input.value())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("CA Certificate Path (optional)"),
            )
            .style(ca_cert_style);
        f.render_widget(ca_cert_input, chunks[5]);

        // Client Certificate (for TLS)
        let client_cert_style = if app.enterprise_active_input == 6 {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else {
            Style::default()
        };
        let client_cert_enabled = matches!(
            app.enterprise_auth_method,
            crate::network::EnterpriseAuthMethod::TLS
        );
        let client_cert_title = if client_cert_enabled {
            "Client Certificate Path"
        } else {
            "Client Certificate (TLS only)"
        };
        let client_cert_input = Paragraph::new(app.enterprise_client_cert_input.value())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(client_cert_title),
            )
            .style(if client_cert_enabled {
                client_cert_style
            } else {
                Style::default().fg(Color::DarkGray)
            });
        f.render_widget(client_cert_input, chunks[6]);

        // Private Key (for TLS)
        let private_key_style = if app.enterprise_active_input == 7 {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else {
            Style::default()
        };
        let private_key_enabled = matches!(
            app.enterprise_auth_method,
            crate::network::EnterpriseAuthMethod::TLS
        );
        let private_key_title = if private_key_enabled {
            "Private Key Path"
        } else {
            "Private Key (TLS only)"
        };
        let private_key_input = Paragraph::new(app.enterprise_private_key_input.value())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(private_key_title),
            )
            .style(if private_key_enabled {
                private_key_style
            } else {
                Style::default().fg(Color::DarkGray)
            });
        f.render_widget(private_key_input, chunks[7]);

        // Private Key Password (for TLS)
        let key_pass_style = if app.enterprise_active_input == 8 {
            Style::default().bg(Color::Blue).fg(Color::White)
        } else {
            Style::default()
        };
        let key_pass_enabled = matches!(
            app.enterprise_auth_method,
            crate::network::EnterpriseAuthMethod::TLS
        );
        let key_pass_title = if key_pass_enabled {
            "Private Key Password (optional)"
        } else {
            "Private Key Password (TLS only)"
        };
        let key_pass_text = if key_pass_enabled {
            "*".repeat(app.enterprise_key_password_input.value().len())
        } else {
            String::new()
        };
        let key_pass_input = Paragraph::new(key_pass_text)
            .block(Block::default().borders(Borders::ALL).title(key_pass_title))
            .style(if key_pass_enabled {
                key_pass_style
            } else {
                Style::default().fg(Color::DarkGray)
            });
        f.render_widget(key_pass_input, chunks[8]);

        // Instructions
        let instructions = Paragraph::new(
            "Tab: Next field | 1: Auth Method | 2: Phase2 | Enter: Connect | Esc: Cancel",
        )
        .wrap(ratatui::widgets::Wrap { trim: true })
        .style(Style::default().fg(Color::Yellow));
        f.render_widget(instructions, chunks[9]);
    }
}

fn draw_hotspot_dialog(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 40, f.area());
    f.render_widget(Clear, area);

    let title = "Create WiFi Hotspot";

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // SSID
            Constraint::Length(3), // Password
            Constraint::Length(3), // Channel
            Constraint::Min(1),    // Instructions
        ])
        .split(area);

    f.render_widget(block, area);

    // SSID input
    let ssid_style = if app.hotspot_active_input == 0 {
        Style::default().bg(Color::Blue).fg(Color::White)
    } else {
        Style::default()
    };
    let ssid_input = Paragraph::new(app.hotspot_ssid_input.value())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Hotspot Name (SSID)"),
        )
        .style(ssid_style);
    f.render_widget(ssid_input, chunks[0]);

    // Password input (masked)
    let password_style = if app.hotspot_active_input == 1 {
        Style::default().bg(Color::Blue).fg(Color::White)
    } else {
        Style::default()
    };
    let password_text = "*".repeat(app.hotspot_password_input.value().len());
    let password_input = Paragraph::new(password_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Password (min 8 chars)"),
        )
        .style(password_style);
    f.render_widget(password_input, chunks[1]);

    // Channel selection
    let channel_style = if app.hotspot_active_input == 2 {
        Style::default().bg(Color::Blue).fg(Color::White)
    } else {
        Style::default()
    };
    let channel_text = format!("Channel {}", app.hotspot_channel);
    let channel_input = Paragraph::new(channel_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("WiFi Channel [Space: Cycle]"),
        )
        .style(channel_style);
    f.render_widget(channel_input, chunks[2]);

    // Instructions
    let instructions = Paragraph::new(
        "Tab: Next field | Space: Cycle Channel | Enter: Create Hotspot | Esc: Cancel",
    )
    .wrap(ratatui::widgets::Wrap { trim: true })
    .style(Style::default().fg(Color::Yellow));
    f.render_widget(instructions, chunks[3]);
}

fn draw_wifi_diagnostics_dialog(f: &mut Frame, app: &App) {
    let area = centered_rect(85, 85, f.area());
    f.render_widget(Clear, area);

    let title = "WiFi Diagnostics & Connection Details";

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if let Some(diagnostics) = &app.wifi_diagnostics_data {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(8), // Connection Info
                Constraint::Length(6), // Signal & Performance
                Constraint::Length(8), // Network Statistics
                Constraint::Min(1),    // Advanced Details
                Constraint::Length(2), // Instructions
            ])
            .split(inner);

        // Connection Info Section
        let connection_info = vec![
            Line::from(Span::styled(
                "🔗 Connection Information",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("Network: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&diagnostics.ssid),
            ]),
            Line::from(vec![
                Span::styled("BSSID: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&diagnostics.bssid),
            ]),
            Line::from(vec![
                Span::styled("Security: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{:?}", diagnostics.security)),
            ]),
            Line::from(vec![
                Span::styled(
                    "Encryption: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(diagnostics.encryption.join(", ")),
            ]),
            Line::from(vec![
                Span::styled(
                    "Connected Time: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(if let Some(uptime) = &diagnostics.connected_time {
                    format!("{} minutes", uptime.as_secs() / 60)
                } else {
                    "Unknown".to_string()
                }),
            ]),
        ];

        let connection_widget = Paragraph::new(connection_info)
            .block(Block::default().borders(Borders::ALL).title("Connection"));
        f.render_widget(connection_widget, chunks[0]);

        // Signal & Performance Section
        let signal_color = match diagnostics.signal_strength {
            s if s > -50 => Color::Green,
            s if s > -60 => Color::Yellow,
            s if s > -70 => Color::Magenta,
            _ => Color::Red,
        };

        let signal_info = vec![
            Line::from(Span::styled(
                "📶 Signal & Performance",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Green),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "Signal Strength: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{} dBm", diagnostics.signal_strength),
                    Style::default().fg(signal_color),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "Signal Quality: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(if let Some(quality) = diagnostics.signal_quality {
                    format!("{}%", quality)
                } else {
                    "Unknown".to_string()
                }),
            ]),
            Line::from(vec![
                Span::styled("Frequency: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!(
                    "{} MHz (Channel {})",
                    diagnostics.frequency, diagnostics.channel
                )),
            ]),
            Line::from(vec![
                Span::styled(
                    "Link Speed: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(if let Some(speed) = diagnostics.link_speed {
                    format!("{} Mbps", speed)
                } else {
                    "Unknown".to_string()
                }),
            ]),
        ];

        let signal_widget = Paragraph::new(signal_info).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Signal & Performance"),
        );
        f.render_widget(signal_widget, chunks[1]);

        // Network Statistics Section
        let rx_bytes = byte_unit::Byte::from_u128(diagnostics.rx_bytes as u128)
            .unwrap_or(byte_unit::Byte::from_u64(0));
        let tx_bytes = byte_unit::Byte::from_u128(diagnostics.tx_bytes as u128)
            .unwrap_or(byte_unit::Byte::from_u64(0));

        let stats_info = vec![
            Line::from(Span::styled(
                "📊 Network Statistics",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Yellow),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("📥 RX: ", Style::default().fg(Color::Green)),
                Span::raw(format!(
                    "{:.2} ({} packets)",
                    rx_bytes.get_appropriate_unit(byte_unit::UnitType::Binary),
                    diagnostics.rx_packets
                )),
            ]),
            Line::from(vec![
                Span::styled("📤 TX: ", Style::default().fg(Color::Blue)),
                Span::raw(format!(
                    "{:.2} ({} packets)",
                    tx_bytes.get_appropriate_unit(byte_unit::UnitType::Binary),
                    diagnostics.tx_packets
                )),
            ]),
            Line::from(vec![
                Span::styled("❌ RX Errors: ", Style::default().fg(Color::Red)),
                Span::raw(format!(
                    "{} errors, {} dropped",
                    diagnostics.rx_errors, diagnostics.rx_dropped
                )),
            ]),
            Line::from(vec![
                Span::styled("❌ TX Errors: ", Style::default().fg(Color::Red)),
                Span::raw(format!(
                    "{} errors, {} dropped, {} retries",
                    diagnostics.tx_errors, diagnostics.tx_dropped, diagnostics.tx_retries
                )),
            ]),
        ];

        let stats_widget = Paragraph::new(stats_info).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Network Statistics"),
        );
        f.render_widget(stats_widget, chunks[2]);

        // Advanced Details Section
        let advanced_info = vec![
            Line::from(Span::styled(
                "🔧 Advanced Details",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Magenta),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("TX Power: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(if let Some(power) = diagnostics.tx_power {
                    format!("{} dBm", power)
                } else {
                    "Unknown".to_string()
                }),
            ]),
            Line::from(vec![
                Span::styled(
                    "Packet Loss Rate: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw({
                    let total_tx = diagnostics.tx_packets;
                    let total_errors = diagnostics.tx_errors + diagnostics.tx_dropped;
                    if total_tx > 0 {
                        format!("{:.2}%", (total_errors as f64 / total_tx as f64) * 100.0)
                    } else {
                        "0.00%".to_string()
                    }
                }),
            ]),
            Line::from(vec![
                Span::styled(
                    "Connection Quality: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    {
                        let quality_score = if let Some(quality) = diagnostics.signal_quality {
                            match quality {
                                90..=100 => "Excellent",
                                70..=89 => "Good",
                                50..=69 => "Fair",
                                30..=49 => "Poor",
                                _ => "Very Poor",
                            }
                        } else {
                            "Unknown"
                        };
                        quality_score.to_string()
                    },
                    Style::default().fg(match diagnostics.signal_quality.unwrap_or(0) {
                        90..=100 => Color::Green,
                        70..=89 => Color::Yellow,
                        50..=69 => Color::Magenta,
                        _ => Color::Red,
                    }),
                ),
            ]),
        ];

        let advanced_widget = Paragraph::new(advanced_info).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Advanced Details"),
        );
        f.render_widget(advanced_widget, chunks[3]);

        // Instructions
        let instructions = Paragraph::new("Press Esc to close | r: Refresh diagnostics")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(instructions, chunks[4]);
    } else {
        // No WiFi connection or data available
        let no_data = vec![
            Line::from(""),
            Line::from(Span::styled(
                "❌ No WiFi Connection Found",
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Red),
            )),
            Line::from(""),
            Line::from("The selected interface is not connected to a WiFi network"),
            Line::from("or WiFi diagnostics data is not available."),
            Line::from(""),
            Line::from("Please:"),
            Line::from("• Connect to a WiFi network first"),
            Line::from("• Select a WiFi-enabled interface"),
            Line::from("• Ensure the interface is UP"),
        ];

        let no_data_widget = Paragraph::new(no_data)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("No Data Available"),
            )
            .alignment(Alignment::Center);
        f.render_widget(no_data_widget, inner);
    }
}

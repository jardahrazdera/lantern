// src/main.rs
mod app;
mod network;
mod ui;
mod config;
mod systemd;
mod utils;
mod iwd;
mod icons;

use anyhow::Result;
use clap::{Arg, Command};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{io::{self, Write}, time::Duration};
use tokio::sync::mpsc;

// Messages for non-blocking updates
#[derive(Debug)]
enum UpdateMessage {
    StatsUpdate(Vec<network::Interface>),
    InterfacesUpdate(Vec<network::Interface>),
    WiFiInfoUpdate(Vec<network::Interface>),
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let matches = Command::new("lantern")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .long_about("Lantern is a modern TUI for Linux network interface management.\n\nFeatures:\n• Network interface configuration (DHCP/static)\n• WiFi management with WPA/WPA2/WPA3/Enterprise support\n• WiFi hotspot creation\n• IPv6 configuration\n• WireGuard VPN management\n• Real-time network monitoring\n• systemd-networkd integration")
        .arg(Arg::new("cli")
            .long("cli")
            .short('c')
            .help("Force CLI mode (no TUI)")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("version")
            .long("version")
            .short('V')
            .help("Print version information")
            .action(clap::ArgAction::SetTrue))
        .get_matches();

    // Handle version flag
    if matches.get_flag("version") {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        println!("{}", env!("CARGO_PKG_DESCRIPTION"));
        println!("Repository: {}", env!("CARGO_PKG_REPOSITORY"));
        return Ok(());
    }

    // Force CLI mode if requested
    let force_cli = matches.get_flag("cli");

    // Check if running as root
    if !nix::unistd::Uid::effective().is_root() {
        eprintln!("{}  Lantern requires root privileges for network configuration", crate::icons::WARNING);
        eprintln!("   Please run with: sudo lantern");
        eprintln!("   This is required for:");
        eprintln!("   • Network interface management");
        eprintln!("   • WiFi configuration");
        eprintln!("   • VPN/WireGuard setup");
        eprintln!("   • systemd-networkd configuration");
        std::process::exit(1);
    }


    // Try to setup terminal, fall back to CLI mode if it fails or if forced
    if force_cli || enable_raw_mode().is_err() {
        if force_cli {
            eprintln!("{} Starting in CLI mode (--cli flag used)...", crate::icons::SETTINGS);
        } else {
            eprintln!("{} TUI mode not available, starting in CLI mode...", crate::icons::SETTINGS);
        }
        return run_cli_mode().await;
    }
    
    let mut stdout = io::stdout();
    if let Err(e) = execute!(stdout, EnterAlternateScreen, EnableMouseCapture) {
        disable_raw_mode().ok();
        eprintln!("{} Screen setup failed: {}", crate::icons::ERROR, e);
        eprintln!("   Your terminal may not support the required features.");
        std::process::exit(1);
    }
    
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = match Terminal::new(backend) {
        Ok(t) => t,
        Err(e) => {
            disable_raw_mode().ok();
            eprintln!("{} Terminal initialization failed: {}", crate::icons::ERROR, e);
            std::process::exit(1);
        }
    };

    // Create app and run
    let app = app::App::new().await?;
    let res = run_app(&mut terminal, app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("{} Application Error: {}", crate::icons::ERROR, err);
        
        // Provide helpful context for common errors
        let err_str = format!("{:?}", err);
        if err_str.contains("Permission denied") {
            eprintln!("{} This may be caused by insufficient privileges.", crate::icons::INFO);
            eprintln!("   Make sure you're running as root: sudo lantern");
        } else if err_str.contains("Command") && err_str.contains("not found") {
            eprintln!("{} Missing required system tools.", crate::icons::INFO);
            eprintln!("   Please install: iproute2, wireless-tools, wireguard-tools");
        } else if err_str.contains("systemd") {
            eprintln!("{} systemd-networkd may not be running.", crate::icons::INFO);
            eprintln!("   Try: sudo systemctl enable --now systemd-networkd");
        }
        
        std::process::exit(1);
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: app::App) -> Result<()> {
    // Create channel for non-blocking updates
    let (update_tx, mut update_rx) = mpsc::unbounded_channel::<UpdateMessage>();
    loop {
        // Process pending WiFi scan BEFORE checking for new events
        // This ensures the loading dialog is drawn first
        if app.wifi_scan_pending {
            // First, make sure loading dialog is visible
            terminal.draw(|f| ui::draw(f, &mut app))?;
            terminal.backend_mut().flush()?;
            
            // Small delay to ensure render
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            
            // Now do the actual scan
            app.process_wifi_scan_if_pending().await?;
            app.needs_redraw = true;
        }
        
        // Only redraw if needed (performance optimization)
        if app.needs_redraw() {
            terminal.draw(|f| ui::draw(f, &mut app))?;
            terminal.backend_mut().flush()?; // Force immediate flush
            app.mark_redrawn();
        }

        // Use shorter poll time for more responsive UI
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Char('r') if !app.show_wifi_dialog => {
                        app.manual_refresh_interfaces().await?;
                        app.needs_redraw = true;
                    }
                    // WiFi dialog navigation (only when connect and enterprise dialogs are NOT open)
                    KeyCode::Up | KeyCode::Char('k') if app.show_wifi_dialog && !app.show_wifi_connect_dialog && !app.show_wifi_enterprise_dialog && !app.show_hotspot_dialog => {
                        app.wifi_navigate_up();
                        app.needs_redraw = true;
                    }
                    KeyCode::Down | KeyCode::Char('j') if app.show_wifi_dialog && !app.show_wifi_connect_dialog && !app.show_wifi_enterprise_dialog && !app.show_hotspot_dialog => {
                        app.wifi_navigate_down();
                        app.needs_redraw = true;
                    }
                    KeyCode::Enter if app.show_wifi_dialog && !app.show_wifi_connect_dialog && !app.show_wifi_enterprise_dialog && !app.show_hotspot_dialog => {
                        app.open_wifi_connect_dialog();
                        app.needs_redraw = true;
                    }
                    KeyCode::Enter if app.show_wifi_connect_dialog && !app.show_wifi_enterprise_dialog && !app.show_hotspot_dialog => {
                        // Check if network is Enterprise and open Enterprise dialog
                        if let Some(network) = app.get_selected_wifi_network() {
                            if network.security == crate::network::WifiSecurity::Enterprise {
                                app.open_wifi_enterprise_dialog();
                            } else {
                                app.connect_to_selected_wifi().await?;
                            }
                        }
                        app.needs_redraw = true;
                    }
                    KeyCode::Enter if app.show_wifi_enterprise_dialog && !app.show_hotspot_dialog => {
                        app.connect_to_enterprise_wifi().await?;
                        app.needs_redraw = true;
                    }
                    // General navigation (only when no dialogs are active)
                    KeyCode::Up | KeyCode::Char('k') if !app.show_edit_dialog && !app.show_wifi_dialog && !app.show_wifi_connect_dialog && !app.show_wifi_enterprise_dialog && !app.show_hotspot_dialog => app.previous(),
                    KeyCode::Down | KeyCode::Char('j') if !app.show_edit_dialog && !app.show_wifi_dialog && !app.show_wifi_connect_dialog && !app.show_wifi_enterprise_dialog && !app.show_hotspot_dialog => app.next(),
                    KeyCode::Enter if !app.show_edit_dialog && !app.show_wifi_dialog && !app.show_wifi_connect_dialog && !app.show_wifi_enterprise_dialog && !app.show_hotspot_dialog => app.toggle_details(),
                    KeyCode::Char('e') if !app.show_edit_dialog && !app.show_wifi_dialog && !app.show_wifi_connect_dialog && !app.show_wifi_enterprise_dialog && !app.show_hotspot_dialog => {
                        app.edit_interface();
                        app.needs_redraw = true;
                    }
                    KeyCode::Char('u') if !app.show_edit_dialog && !app.show_wifi_dialog && !app.show_wifi_connect_dialog && !app.show_wifi_enterprise_dialog && !app.show_hotspot_dialog => {
                        app.toggle_interface_state().await?;
                        app.needs_redraw = true;
                    }
                    KeyCode::Char('h') if !app.show_edit_dialog && !app.show_wifi_dialog && !app.show_wifi_connect_dialog && !app.show_wifi_enterprise_dialog && !app.show_hotspot_dialog => {
                        app.open_hotspot_dialog();
                        app.needs_redraw = true;
                    }
                    KeyCode::Char('w') if !app.show_edit_dialog && !app.show_wifi_dialog && !app.show_wifi_connect_dialog && !app.show_wifi_enterprise_dialog && !app.show_hotspot_dialog => {
                        // Show loading dialog IMMEDIATELY in the event handler
                        app.show_wifi_loading_dialog = true;
                        app.wifi_scan_pending = true;
                        
                        // Force immediate redraw RIGHT NOW
                        terminal.draw(|f| ui::draw(f, &mut app))?;
                        // Multiple flushes to ensure it works in release mode
                        let _ = terminal.backend_mut().flush();
                        let _ = std::io::stdout().flush();
                        let _ = std::io::stderr().flush();
                    }
                    KeyCode::Char(' ') if app.show_edit_dialog => {
                        app.toggle_dhcp();
                        app.needs_redraw = true;
                    }
                    KeyCode::Tab if app.show_edit_dialog => {
                        app.next_input();
                        app.needs_redraw = true;
                    }
                    KeyCode::Esc => {
                        if app.show_wifi_diagnostics_dialog {
                            app.close_wifi_diagnostics_dialog();
                        } else if app.show_hotspot_dialog {
                            app.close_hotspot_dialog();
                        } else if app.show_wifi_enterprise_dialog {
                            app.close_wifi_enterprise_dialog();
                        } else if app.show_wifi_connect_dialog {
                            app.close_wifi_connect_dialog();
                        } else if app.show_wifi_loading_dialog {
                            app.show_wifi_loading_dialog = false;
                        } else if app.show_wifi_dialog {
                            app.close_wifi_dialog();
                        } else {
                            app.close_dialog();
                        }
                        app.needs_redraw = true;
                    }
                    KeyCode::Char('s') if app.show_edit_dialog => {
                        app.save_configuration().await?;
                        app.needs_redraw = true;
                    }
                    KeyCode::Char('r') if app.show_wifi_dialog && !app.show_wifi_connect_dialog && !app.show_wifi_enterprise_dialog && !app.show_hotspot_dialog => {
                        app.scan_wifi_networks().await?;
                        app.needs_redraw = true;
                    }
                    KeyCode::Char('a') if app.show_wifi_dialog && !app.show_wifi_connect_dialog && !app.show_wifi_enterprise_dialog && !app.show_hotspot_dialog => {
                        app.toggle_wifi_auto_connect()?;
                        app.needs_redraw = true;
                    }
                    KeyCode::Char('e') if app.show_wifi_dialog && !app.show_wifi_connect_dialog && !app.show_wifi_enterprise_dialog && !app.show_hotspot_dialog => {
                        app.open_wifi_enterprise_dialog();
                        app.needs_redraw = true;
                    }
                    KeyCode::Char('d') if app.show_wifi_dialog && !app.show_wifi_connect_dialog && !app.show_wifi_enterprise_dialog && !app.show_hotspot_dialog => {
                        app.open_wifi_diagnostics_dialog().await;
                        app.needs_redraw = true;
                    }
                    // WiFi connect dialog input
                    KeyCode::Tab if app.show_wifi_connect_dialog && !app.show_wifi_enterprise_dialog && !app.show_hotspot_dialog => {
                        app.wifi_connect_next_input();
                        app.needs_redraw = true;
                    }
                    KeyCode::Char(' ') if app.show_wifi_connect_dialog && !app.show_wifi_enterprise_dialog && !app.show_hotspot_dialog => {
                        app.wifi_connect_toggle_dhcp();
                        app.needs_redraw = true;
                    }
                    KeyCode::Char(c) if app.show_wifi_connect_dialog && !app.show_wifi_enterprise_dialog && !app.show_hotspot_dialog && c != ' ' => {
                        app.wifi_connect_input_char(c);
                        app.needs_redraw = true;
                    }
                    KeyCode::Backspace if app.show_wifi_connect_dialog && !app.show_wifi_enterprise_dialog && !app.show_hotspot_dialog => {
                        app.wifi_connect_delete_char();
                        app.needs_redraw = true;
                    }
                    // Enterprise WiFi dialog input
                    KeyCode::Tab if app.show_wifi_enterprise_dialog && !app.show_hotspot_dialog => {
                        app.enterprise_next_input();
                        app.needs_redraw = true;
                    }
                    KeyCode::Char('1') if app.show_wifi_enterprise_dialog && !app.show_hotspot_dialog => {
                        app.enterprise_cycle_auth_method();
                        app.needs_redraw = true;
                    }
                    KeyCode::Char('2') if app.show_wifi_enterprise_dialog && !app.show_hotspot_dialog => {
                        app.enterprise_cycle_phase2_auth();
                        app.needs_redraw = true;
                    }
                    KeyCode::Char(c) if app.show_wifi_enterprise_dialog && !app.show_hotspot_dialog && c != '1' && c != '2' => {
                        app.enterprise_input_char(c);
                        app.needs_redraw = true;
                    }
                    KeyCode::Backspace if app.show_wifi_enterprise_dialog && !app.show_hotspot_dialog => {
                        app.enterprise_delete_char();
                        app.needs_redraw = true;
                    }
                    // Hotspot dialog input
                    KeyCode::Tab if app.show_hotspot_dialog => {
                        app.hotspot_next_input();
                        app.needs_redraw = true;
                    }
                    KeyCode::Char(' ') if app.show_hotspot_dialog && app.hotspot_active_input == 2 => {
                        app.hotspot_cycle_channel();
                        app.needs_redraw = true;
                    }
                    KeyCode::Enter if app.show_hotspot_dialog => {
                        app.create_hotspot().await?;
                        app.needs_redraw = true;
                    }
                    KeyCode::Char(c) if app.show_hotspot_dialog && c != ' ' => {
                        app.hotspot_input_char(c);
                        app.needs_redraw = true;
                    }
                    KeyCode::Backspace if app.show_hotspot_dialog => {
                        app.hotspot_delete_char();
                        app.needs_redraw = true;
                    }
                    // WiFi diagnostics dialog input
                    KeyCode::Char('r') if app.show_wifi_diagnostics_dialog => {
                        app.refresh_wifi_diagnostics().await;
                        app.needs_redraw = true;
                    }
                    KeyCode::Char(c) if app.show_edit_dialog && c != ' ' => {
                        app.input_char(c);
                        app.needs_redraw = true;
                    }
                    KeyCode::Backspace if app.show_edit_dialog => {
                        app.delete_char();
                        app.needs_redraw = true;
                    }
                    _ => {}
                }
            }
        }

        // Check for non-blocking update results
        while let Ok(update) = update_rx.try_recv() {
            match update {
                UpdateMessage::StatsUpdate(updated_interfaces) => {
                    // Update stats only (preserve other interface data)
                    for (i, updated) in updated_interfaces.iter().enumerate() {
                        if let Some(interface) = app.interfaces.get_mut(i) {
                            interface.stats = updated.stats.clone();
                        }
                    }
                    app.needs_redraw = true;
                }
                UpdateMessage::InterfacesUpdate(interfaces) => {
                    app.interfaces = interfaces;
                    app.needs_redraw = true;
                }
                UpdateMessage::WiFiInfoUpdate(updated_interfaces) => {
                    // Update WiFi info only
                    for updated in updated_interfaces {
                        if let Some(interface) = app.interfaces.iter_mut().find(|i| i.name == updated.name) {
                            interface.wifi_info = updated.wifi_info;
                        }
                    }
                    app.needs_redraw = true;
                }
            }
        }

        // Start non-blocking updates when needed
        if app.should_refresh_stats() {
            let tx = update_tx.clone();
            let network_manager = app.network_manager.clone();
            let mut interfaces = app.interfaces.clone();
            tokio::spawn(async move {
                if let Ok(()) = network_manager.update_interface_stats(&mut interfaces).await {
                    let _ = tx.send(UpdateMessage::StatsUpdate(interfaces));
                }
            });
            app.mark_stats_refresh_started();
        }

        if app.should_refresh_interfaces() {
            let tx = update_tx.clone();
            let network_manager = app.network_manager.clone();
            tokio::spawn(async move {
                if let Ok(interfaces) = network_manager.get_interfaces().await {
                    let _ = tx.send(UpdateMessage::InterfacesUpdate(interfaces));
                }
            });
            app.mark_interface_refresh_started();
        }

        if app.should_update_wifi_info() {
            let tx = update_tx.clone();
            let network_manager = app.network_manager.clone();
            let interfaces = app.interfaces.clone();
            tokio::spawn(async move {
                let mut updated_interfaces = Vec::new();
                for interface in interfaces {
                    if interface.wifi_info.is_some() && interface.state == "UP" {
                        if let Ok(wifi_info) = network_manager.get_wifi_info(&interface.name).await {
                            let mut updated = interface.clone();
                            updated.wifi_info = wifi_info;
                            updated_interfaces.push(updated);
                        }
                    }
                }
                if !updated_interfaces.is_empty() {
                    let _ = tx.send(UpdateMessage::WiFiInfoUpdate(updated_interfaces));
                }
            });
            app.mark_wifi_update_started();
        }

        // Auto-connect check every 30 seconds
        if app.should_check_auto_connect() {
            // Run auto-connect in background (non-blocking)
            let mut app_clone = app.clone();
            tokio::spawn(async move {
                let _ = app_clone.check_auto_connect().await;
            });
            app.mark_auto_connect_check_started();
        }
    }
}

async fn run_cli_mode() -> Result<()> {
    use crate::network::NetworkManager;
    
    println!("{} Lantern Network Manager - CLI Mode", crate::icons::LANTERN);
    println!("======================================");
    
    let mut network_manager = NetworkManager::new();
    
    // Try to initialize iwd
    match network_manager.init_iwd().await {
        Ok(_) => println!("{} iwd integration enabled", crate::icons::SUCCESS),
        Err(_) => println!("{}  iwd not available, using fallback methods", crate::icons::WARNING),
    }
    println!();
    
    // Get and display interfaces
    match network_manager.get_interfaces().await {
        Ok(interfaces) => {
            println!("\n{} Network Interfaces:", crate::icons::ETHERNET);
            println!("   {:<12} {:<8} {:<15} {:<10} {:<10}", "Interface", "State", "IP Address", "RX", "TX");
            println!("   {}", "-".repeat(60));
            
            for interface in &interfaces {
                let ip = interface.ipv4_addresses.first()
                    .map(|addr| addr.split('/').next().unwrap_or("N/A"))
                    .unwrap_or("N/A");
                
                let rx = if interface.stats.rx_bytes > 1024 * 1024 {
                    format!("{:.1}MB", interface.stats.rx_bytes as f64 / 1024.0 / 1024.0)
                } else if interface.stats.rx_bytes > 1024 {
                    format!("{:.1}KB", interface.stats.rx_bytes as f64 / 1024.0)
                } else {
                    format!("{}B", interface.stats.rx_bytes)
                };
                
                let tx = if interface.stats.tx_bytes > 1024 * 1024 {
                    format!("{:.1}MB", interface.stats.tx_bytes as f64 / 1024.0 / 1024.0)
                } else if interface.stats.tx_bytes > 1024 {
                    format!("{:.1}KB", interface.stats.tx_bytes as f64 / 1024.0)
                } else {
                    format!("{}B", interface.stats.tx_bytes)
                };
                
                println!("   {:<12} {:<8} {:<15} {:<10} {:<10}", 
                    interface.name, 
                    &interface.state,
                    ip,
                    rx,
                    tx
                );
            }
            
            println!("\n{} Lantern CLI mode completed successfully!", crate::icons::SUCCESS);
            println!("{} For interactive management, run from a proper terminal with TUI support", crate::icons::INFO);
        }
        Err(e) => {
            eprintln!("{} Failed to get interfaces: {}", crate::icons::ERROR, e);
            return Err(e);
        }
    }
    
    Ok(())
}

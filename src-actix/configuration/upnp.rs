use crate::configuration::configuration_data::Configuration;
use igd::{self, PortMappingProtocol};
use log::{debug, error, info, warn};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::str::FromStr;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

// Singleton to manage the UPnP port mapping state
static UPNP_STATE: OnceLock<Arc<Mutex<Option<UPnPState>>>> = OnceLock::new();

// Helper function to get or initialize the UPNP_STATE
fn get_upnp_state() -> &'static Arc<Mutex<Option<UPnPState>>> {
    UPNP_STATE.get_or_init(|| Arc::new(Mutex::new(None)))
}

struct UPnPState {
    port: u16,
    is_forwarded: bool,
}

/// Initialize UPnP functionality based on the current configuration
pub fn initialize() {
    let config = Configuration::get();

    if config.upnp_enabled {
        update_port_forwarding(config.port);
    }
}

/// Update port forwarding when configuration changes
pub fn handle_config_change(old_config: &Configuration, new_config: &Configuration) {
    // If UPnP was disabled and is now enabled, start forwarding
    if !old_config.upnp_enabled && new_config.upnp_enabled {
        update_port_forwarding(new_config.port);
    }
    // If UPnP was enabled and is now disabled, stop forwarding
    else if old_config.upnp_enabled && !new_config.upnp_enabled {
        remove_port_forwarding();
    }
    // If UPnP is enabled and the port changed, update forwarding
    else if new_config.upnp_enabled && old_config.port != new_config.port {
        update_port_forwarding(new_config.port);
    }
}

/// Forward the specified port using UPnP
pub fn update_port_forwarding(port: u16) {
    // Remove any existing port forwarding first
    remove_port_forwarding();

    debug!("Forwarding port {} using UPnP...", port);

    // Get the local IP address
    let local_ip = match local_ipaddress::get() {
        Some(ip) => match Ipv4Addr::from_str(&ip) {
            Ok(addr) => addr,
            Err(e) => {
                error!("Failed to parse local IP address {}: {}", ip, e);
                return;
            }
        },
        None => {
            error!("Failed to get local IP address");
            return;
        }
    };

    // Try to discover the default gateway and set up port forwarding
    match igd::search_gateway(igd::SearchOptions {
        timeout: Some(Duration::from_secs(3)),
        ..Default::default()
    }) {
        Ok(gateway) => {
            // Forward the port
            let socket = SocketAddrV4::new(local_ip, port);
            match gateway.add_port(
                PortMappingProtocol::TCP,
                port,
                socket,
                0, // Lease duration of 0 means "forever" until explicitly removed
                "Filer Server",
            ) {
                Ok(_) => {
                    info!("Successfully forwarded port {} using UPnP", port);
                    // Store the port for later cleanup
                    let mut state = get_upnp_state().lock().unwrap();
                    *state = Some(UPnPState {
                        port,
                        is_forwarded: true,
                    });
                }
                Err(e) => {
                    error!("Failed to forward port {}: {}", port, e);
                }
            }
        }
        Err(e) => {
            error!("Failed to discover UPnP gateway: {}", e);
        }
    }
}

/// Remove port forwarding
fn remove_port_forwarding() {
    let mut state_guard = get_upnp_state().lock().unwrap();

    if let Some(state) = state_guard.as_ref() {
        if state.is_forwarded {
            debug!("Removing port forwarding for port {}", state.port);

            match igd::search_gateway(igd::SearchOptions {
                timeout: Some(Duration::from_secs(3)),
                ..Default::default()
            }) {
                Ok(gateway) => {
                    match gateway.remove_port(PortMappingProtocol::TCP, state.port) {
                        Ok(_) => {
                            info!("Successfully removed port forwarding for port {}", state.port);
                        }
                        Err(e) => {
                            warn!("Failed to remove port forwarding for port {}: {}", state.port, e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to discover UPnP gateway for port removal: {}", e);
                }
            }
        }
    }

    // Clear the state
    *state_guard = None;
}

/// Clean up UPnP port forwarding when the application exits
pub fn cleanup() {
    debug!("Cleaning up UPnP port forwarding...");
    remove_port_forwarding();
}

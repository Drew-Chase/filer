use actix_web::{
    http::header,
    HttpRequest, HttpResponse,
};
use crate::configuration::configuration_data::Configuration;
use std::str::FromStr;

// Simple function to add CORS headers to a response based on configuration
pub fn add_cors_headers(response: &mut HttpResponse) {
    let config = Configuration::get();

    if config.cors_enabled {
        // If CORS is enabled, add permissive CORS headers
        response.headers_mut().insert(
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            header::HeaderValue::from_static("*"),
        );
        response.headers_mut().insert(
            header::ACCESS_CONTROL_ALLOW_METHODS,
            header::HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"),
        );
        response.headers_mut().insert(
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            header::HeaderValue::from_static("Content-Type, Authorization, Accept"),
        );
        response.headers_mut().insert(
            header::ACCESS_CONTROL_MAX_AGE,
            header::HeaderValue::from_static("3600"),
        );
    }
}

// Function to check if a request is from an authorized host
pub fn is_authorized_host(req: &HttpRequest) -> bool {
    let config = Configuration::get();

    // Skip IP filtering if there are no authorized hosts
    if config.authorized_hosts.is_empty() {
        return true;
    }

    // Get client IP
    let connection_info = req.connection_info();
    let client_ip = connection_info.peer_addr().unwrap_or("unknown");

    // Extract just the IP part if it includes a port
    let client_ip = client_ip.split(':').next().unwrap_or(client_ip);

    // Check if client IP is in the authorized hosts list
    config.authorized_hosts.iter().any(|host| {
        // Check for an exact match (for hostnames like "localhost")
        if host == client_ip {
            return true;
        }

        // Try to parse as IP address for more precise comparison
        if let (Ok(host_ip), Ok(client_addr)) = (
            std::net::IpAddr::from_str(host),
            std::net::IpAddr::from_str(client_ip)
        ) {
            return host_ip == client_addr;
        }

        false
    })
}

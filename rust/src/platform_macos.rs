// Jetlag Core — macOS DNS implementation via `networksetup` CLI.
//
// macOS provides the `networksetup` command-line tool to manage network
// service configurations. DNS changes via this tool do not require elevated
// privileges for the current user's own network services.

use crate::models::*;
use std::process::Command;

/// The default network service name on macOS.
const DEFAULT_SERVICE: &str = "Wi-Fi";

/// Set DNS on macOS using `networksetup -setdnsservers`.
pub fn set_dns(provider: &DnsProvider, interface_name: Option<&str>) -> DnsChangeResult {
    let service = interface_name.unwrap_or(DEFAULT_SERVICE);

    // Build the command arguments
    let mut args = vec!["-setdnsservers", service, &provider.primary_dns, &provider.secondary_dns];

    // Add IPv6 addresses if available
    let ipv6_primary;
    let ipv6_secondary;
    if let Some(ref ipv6) = provider.primary_dns_ipv6 {
        ipv6_primary = ipv6.clone();
        args.push(&ipv6_primary);
    }
    if let Some(ref ipv6) = provider.secondary_dns_ipv6 {
        ipv6_secondary = ipv6.clone();
        args.push(&ipv6_secondary);
    }

    match Command::new("networksetup").args(&args).output() {
        Ok(output) => {
            if output.status.success() {
                // Flush DNS cache
                let _ = Command::new("dscacheutil").arg("-flushcache").output();
                let _ = Command::new("killall")
                    .args(["-HUP", "mDNSResponder"])
                    .output();

                DnsChangeResult::success(
                    DnsChangeMethod::Networksetup,
                    &format!(
                        "DNS set to {} ({}, {}) on {}",
                        provider.name, provider.primary_dns, provider.secondary_dns, service
                    ),
                )
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                DnsChangeResult::failure(
                    DnsChangeMethod::Networksetup,
                    &format!("networksetup failed: {}", stderr.trim()),
                )
            }
        }
        Err(e) => DnsChangeResult::failure(
            DnsChangeMethod::Networksetup,
            &format!("Failed to execute networksetup: {}", e),
        ),
    }
}

/// Reset DNS to defaults (DHCP) on macOS using `networksetup -setdnsservers <service> "Empty"`.
pub fn reset_dns(interface_name: Option<&str>) -> DnsChangeResult {
    let service = interface_name.unwrap_or(DEFAULT_SERVICE);

    match Command::new("networksetup")
        .args(["-setdnsservers", service, "Empty"])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                // Flush DNS cache
                let _ = Command::new("dscacheutil").arg("-flushcache").output();
                let _ = Command::new("killall")
                    .args(["-HUP", "mDNSResponder"])
                    .output();

                DnsChangeResult::success(
                    DnsChangeMethod::Networksetup,
                    &format!("DNS reset to defaults on {}", service),
                )
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                DnsChangeResult::failure(
                    DnsChangeMethod::Networksetup,
                    &format!("networksetup reset failed: {}", stderr.trim()),
                )
            }
        }
        Err(e) => DnsChangeResult::failure(
            DnsChangeMethod::Networksetup,
            &format!("Failed to execute networksetup: {}", e),
        ),
    }
}

/// Get current DNS servers for a network service on macOS.
pub fn get_current_dns(interface_name: Option<&str>) -> Vec<String> {
    let service = interface_name.unwrap_or(DEFAULT_SERVICE);

    match Command::new("networksetup")
        .args(["-getdnsservers", service])
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let text = stdout.trim();

            // "There aren't any DNS Servers set on Wi-Fi." means DHCP default
            if text.contains("aren't any") || text.is_empty() {
                return Vec::new();
            }

            text.lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect()
        }
        Err(_) => Vec::new(),
    }
}

/// List all active network services on macOS.
pub fn get_active_interfaces() -> Vec<NetworkInterface> {
    let services = match Command::new("networksetup")
        .args(["-listallnetworkservices"])
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout
                .lines()
                .skip(1) // first line is "An asterisk (*) denotes..."
                .filter(|line| !line.starts_with('*')) // asterisk = disabled
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect::<Vec<String>>()
        }
        Err(_) => Vec::new(),
    };

    // Check which services are actually active (have an IP address)
    services
        .into_iter()
        .enumerate()
        .filter_map(|(index, name)| {
            let is_active = is_service_active(&name);
            Some(NetworkInterface {
                name,
                index: index as u32,
                is_active,
            })
        })
        .collect()
}

/// Check if a network service is active by checking if it has an IP address.
fn is_service_active(service: &str) -> bool {
    match Command::new("networksetup")
        .args(["-getinfo", service])
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // If the service has an IP address, it's active
            stdout.lines().any(|line| {
                let line = line.trim();
                line.starts_with("IP address:") && !line.contains("none")
            })
        }
        Err(_) => false,
    }
}

#[cfg(test)]
#[cfg(target_os = "macos")]
mod tests {
    use super::*;

    #[test]
    fn test_get_active_interfaces() {
        let interfaces = get_active_interfaces();
        // On macOS, we should have at least one network service
        assert!(!interfaces.is_empty(), "Should find at least one network service");
        // Wi-Fi should exist on most Macs
        let has_wifi = interfaces.iter().any(|i| i.name == "Wi-Fi");
        println!("Interfaces found: {:?}", interfaces);
        println!("Has Wi-Fi: {}", has_wifi);
    }

    #[test]
    fn test_get_current_dns() {
        let dns = get_current_dns(None);
        println!("Current DNS servers: {:?}", dns);
        // This just verifies it doesn't crash — DNS may or may not be set
    }
}

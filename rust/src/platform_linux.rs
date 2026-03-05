// Jetlag Core — Linux DNS implementation via D-Bus to systemd-resolved.
//
// Uses `resolvectl` CLI commands as the primary mechanism for changing DNS.
// D-Bus via `zbus` is available when the `linux-dns` feature is enabled
// for more reliable, programmatic control.
//
// PolicyKit rules should be installed during .deb/.rpm packaging to allow
// silent DNS changes without password prompts.

use crate::models::*;
use std::process::Command;

/// Set DNS on Linux using `resolvectl dns`.
pub fn set_dns(provider: &DnsProvider, interface_name: Option<&str>) -> DnsChangeResult {
    let iface = match interface_name {
        Some(name) => name.to_string(),
        None => match detect_default_interface() {
            Some(name) => name,
            None => {
                return DnsChangeResult::failure(
                    DnsChangeMethod::Resolvectl,
                    "Could not detect active network interface. Please specify one.",
                )
            }
        },
    };

    // Set DNS servers
    let dns_args = format!("{} {}", provider.primary_dns, provider.secondary_dns);
    let result = Command::new("resolvectl")
        .args(["dns", &iface, &provider.primary_dns, &provider.secondary_dns])
        .output();

    match result {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return DnsChangeResult::failure(
                    DnsChangeMethod::Resolvectl,
                    &format!(
                        "resolvectl dns failed on {}: {}",
                        iface,
                        stderr.trim()
                    ),
                );
            }
        }
        Err(e) => {
            return DnsChangeResult::failure(
                DnsChangeMethod::Resolvectl,
                &format!("Failed to execute resolvectl: {}", e),
            );
        }
    }

    // Enable DNS-over-TLS if the provider supports it
    if provider.dot_hostname.is_some() {
        let _ = Command::new("resolvectl")
            .args(["dnstls", &iface, "yes"])
            .output();
    }

    // Set the route-only domain wildcard to force all traffic through our DNS
    let _ = Command::new("resolvectl")
        .args(["domain", &iface, "~."])
        .output();

    DnsChangeResult::success(
        DnsChangeMethod::Resolvectl,
        &format!(
            "DNS set to {} ({}, {}) on {}",
            provider.name, provider.primary_dns, provider.secondary_dns, iface
        ),
    )
}

/// Reset DNS to defaults on Linux using `resolvectl revert`.
pub fn reset_dns(interface_name: Option<&str>) -> DnsChangeResult {
    let iface = match interface_name {
        Some(name) => name.to_string(),
        None => match detect_default_interface() {
            Some(name) => name,
            None => {
                return DnsChangeResult::failure(
                    DnsChangeMethod::Resolvectl,
                    "Could not detect active network interface.",
                )
            }
        },
    };

    match Command::new("resolvectl")
        .args(["revert", &iface])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                DnsChangeResult::success(
                    DnsChangeMethod::Resolvectl,
                    &format!("DNS reset to defaults on {}", iface),
                )
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                DnsChangeResult::failure(
                    DnsChangeMethod::Resolvectl,
                    &format!("resolvectl revert failed: {}", stderr.trim()),
                )
            }
        }
        Err(e) => DnsChangeResult::failure(
            DnsChangeMethod::Resolvectl,
            &format!("Failed to execute resolvectl: {}", e),
        ),
    }
}

/// Get current DNS servers from `resolvectl status`.
pub fn get_current_dns(interface_name: Option<&str>) -> Vec<String> {
    let mut cmd = Command::new("resolvectl");
    cmd.arg("status");
    if let Some(iface) = interface_name {
        cmd.arg(iface);
    }

    match cmd.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            parse_resolvectl_dns(&stdout)
        }
        Err(_) => Vec::new(),
    }
}

/// Parse DNS servers from resolvectl status output.
fn parse_resolvectl_dns(output: &str) -> Vec<String> {
    let mut dns_servers = Vec::new();
    let mut in_dns_section = false;

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("DNS Servers:") || trimmed.starts_with("Current DNS Server:") {
            in_dns_section = true;
            // Extract the server from the same line if present
            if let Some(addr) = trimmed.split(':').nth(1) {
                let addr = addr.trim();
                if !addr.is_empty() {
                    dns_servers.push(addr.to_string());
                }
            }
        } else if in_dns_section {
            // Continuation lines for DNS servers are indented
            if trimmed.is_empty() || (!trimmed.starts_with(' ') && line == trimmed) {
                in_dns_section = false;
            } else {
                dns_servers.push(trimmed.to_string());
            }
        }
    }

    dns_servers
}

/// List active network interfaces on Linux.
pub fn get_active_interfaces() -> Vec<NetworkInterface> {
    // Try `ip -o link show up` to get active interfaces
    match Command::new("ip")
        .args(["-o", "link", "show", "up"])
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout
                .lines()
                .filter_map(|line| {
                    // Format: "2: eth0: <BROADCAST,MULTICAST,UP,LOWER_UP> ..."
                    let parts: Vec<&str> = line.splitn(3, ':').collect();
                    if parts.len() >= 2 {
                        let index: u32 = parts[0].trim().parse().ok()?;
                        let name = parts[1].trim().to_string();
                        // Skip loopback
                        if name == "lo" {
                            return None;
                        }
                        Some(NetworkInterface {
                            name,
                            index,
                            is_active: true,
                        })
                    } else {
                        None
                    }
                })
                .collect()
        }
        Err(_) => Vec::new(),
    }
}

/// Detect the default network interface (the one with the default route).
fn detect_default_interface() -> Option<String> {
    // Parse `ip route show default` to find the default interface
    match Command::new("ip")
        .args(["route", "show", "default"])
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Format: "default via 192.168.1.1 dev eth0 proto dhcp ..."
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(dev_pos) = parts.iter().position(|&p| p == "dev") {
                    if let Some(iface) = parts.get(dev_pos + 1) {
                        return Some(iface.to_string());
                    }
                }
            }
            None
        }
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_resolvectl_dns() {
        let output = r#"
Link 2 (eth0)
      Current Scopes: DNS
DefaultRoute setting: yes
       LLMNR setting: yes
MulticastDNS setting: no
  DNSOverTLS setting: no
      DNSSEC setting: no
    DNSSEC supported: no
  Current DNS Server: 1.1.1.1
         DNS Servers: 1.1.1.1
                      1.0.0.1
"#;
        let dns = parse_resolvectl_dns(output);
        assert!(dns.contains(&"1.1.1.1".to_string()));
        assert!(dns.contains(&"1.0.0.1".to_string()));
    }
}

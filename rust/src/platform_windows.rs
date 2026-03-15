// Jetlag Core — Windows DNS implementation.
//
// For the simple CLI approach, we use `netsh` and `powershell` commands.
// For production use, the Windows Service architecture (jetlag_service)
// should be used to avoid UAC prompts — see the plan document.
//
// The full Win32 API approach via SetInterfaceDnsSettings is available
// when compiled with the `windows-dns` feature.

use crate::models::*;
use log::{error, warn};
use std::process::Command;

fn escape_powershell_single_quoted(value: &str) -> String {
    value.replace('\'', "''")
}

fn run_ipconfig_flush() {
    match Command::new("ipconfig").args(["/flushdns"]).output() {
        Ok(flush_output) => {
            if !flush_output.status.success() {
                let stderr = String::from_utf8_lossy(&flush_output.stderr);
                let stdout = String::from_utf8_lossy(&flush_output.stdout);
                warn!(
                    "ipconfig /flushdns failed (status: {:?}): stderr='{}' stdout='{}'",
                    flush_output.status.code(),
                    stderr.trim(),
                    stdout.trim()
                );
            }
        }
        Err(e) => {
            warn!("Failed to execute ipconfig /flushdns: {}", e);
        }
    }
}

/// Set DNS on Windows using PowerShell `Set-DnsClientServerAddress`.
/// Note: This requires administrator privileges and will trigger UAC.
/// For seamless UX, use the jetlag_service Windows Service instead.
pub fn set_dns(provider: &DnsProvider, interface_name: Option<&str>) -> DnsChangeResult {
    let iface = match interface_name {
        Some(name) => name.to_string(),
        None => match detect_default_interface() {
            Some(name) => name,
            None => {
                return DnsChangeResult::failure(
                    DnsChangeMethod::WindowsApi,
                    "Could not detect active network interface.",
                )
            }
        },
    };

    let escaped_iface = escape_powershell_single_quoted(&iface);
    let escaped_primary_dns = escape_powershell_single_quoted(&provider.primary_dns);
    let escaped_secondary_dns = escape_powershell_single_quoted(&provider.secondary_dns);
    let mut address_parts = vec![
        format!("'{}'", escaped_primary_dns),
        format!("'{}'", escaped_secondary_dns),
    ];
    if let Some(ipv6) = &provider.primary_dns_ipv6 {
        if !ipv6.is_empty() {
            address_parts.push(format!("'{}'", escape_powershell_single_quoted(ipv6)));
        }
    }
    if let Some(ipv6) = &provider.secondary_dns_ipv6 {
        if !ipv6.is_empty() {
            address_parts.push(format!("'{}'", escape_powershell_single_quoted(ipv6)));
        }
    }
    let addresses = address_parts.join(",");
    let script = format!(
        "Set-DnsClientServerAddress -InterfaceAlias '{}' -ServerAddresses ({})",
        escaped_iface, addresses
    );

    match Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &script])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                // Flush DNS cache
                run_ipconfig_flush();

                DnsChangeResult::success(
                    DnsChangeMethod::WindowsApi,
                    &format!(
                        "DNS set to {} ({}, {}) on {}",
                        provider.name, provider.primary_dns, provider.secondary_dns, iface
                    ),
                )
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                let error_msg = if !stderr.trim().is_empty() {
                    stderr.to_string()
                } else {
                    stdout.to_string()
                };

                // Check if it's a permission error (case-insensitive)
                let lower = error_msg.to_ascii_lowercase();
                if lower.contains("access is denied")
                    || lower.contains("requires elevation")
                {
                    DnsChangeResult::failure(
                        DnsChangeMethod::WindowsApi,
                        "Administrator privileges required. The Jetlag DNS Service should handle this automatically.",
                    )
                } else if lower.contains("not recognized") {
                    DnsChangeResult::failure(
                        DnsChangeMethod::WindowsApi,
                        "PowerShell or Set-DnsClientServerAddress cmdlet not recognized. Check your PATH/installation.",
                    )
                } else {
                    DnsChangeResult::failure(
                        DnsChangeMethod::WindowsApi,
                        &format!("PowerShell command failed: {}", error_msg.trim()),
                    )
                }
            }
        }
        Err(e) => DnsChangeResult::failure(
            DnsChangeMethod::WindowsApi,
            &format!("Failed to execute PowerShell: {}", e),
        ),
    }
}

/// Reset DNS to DHCP defaults on Windows.
pub fn reset_dns(interface_name: Option<&str>) -> DnsChangeResult {
    let iface = match interface_name {
        Some(name) => name.to_string(),
        None => match detect_default_interface() {
            Some(name) => name,
            None => {
                return DnsChangeResult::failure(
                    DnsChangeMethod::WindowsApi,
                    "Could not detect active network interface.",
                )
            }
        },
    };

    let escaped_iface = escape_powershell_single_quoted(&iface);
    let script = format!(
        "Set-DnsClientServerAddress -InterfaceAlias '{}' -ResetServerAddresses",
        escaped_iface
    );

    match Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &script])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                run_ipconfig_flush();
                DnsChangeResult::success(
                    DnsChangeMethod::WindowsApi,
                    &format!("DNS reset to DHCP defaults on {}", iface),
                )
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                let error_msg = if !stderr.trim().is_empty() {
                    stderr.to_string()
                } else {
                    stdout.to_string()
                };
                let lower = error_msg.to_ascii_lowercase();
                if lower.contains("access is denied")
                    || lower.contains("requires elevation")
                {
                    DnsChangeResult::failure(
                        DnsChangeMethod::WindowsApi,
                        "Administrator privileges required. The Jetlag DNS Service should handle this automatically.",
                    )
                } else if lower.contains("not recognized") {
                    DnsChangeResult::failure(
                        DnsChangeMethod::WindowsApi,
                        "PowerShell or Set-DnsClientServerAddress cmdlet not recognized. Check your PATH/installation.",
                    )
                } else {
                    DnsChangeResult::failure(
                        DnsChangeMethod::WindowsApi,
                        &format!("DNS reset failed: {}", error_msg.trim()),
                    )
                }
            }
        }
        Err(e) => DnsChangeResult::failure(
            DnsChangeMethod::WindowsApi,
            &format!("Failed to execute PowerShell: {}", e),
        ),
    }
}

/// Get current DNS servers on Windows.
pub fn get_current_dns(interface_name: Option<&str>) -> Vec<String> {
    let alias = match interface_name {
        Some(iface) => iface.to_string(),
        None => match detect_default_interface() {
            Some(iface) => iface,
            None => return Vec::new(),
        },
    };

    let script = {
        let escaped_iface = escape_powershell_single_quoted(&alias);
        format!(
            "Get-DnsClientServerAddress -InterfaceAlias '{}' | Select-Object -ExpandProperty ServerAddresses",
            escaped_iface
        )
    };

    match Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &script])
        .output()
    {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("get_current_dns PowerShell failed: {}", stderr.trim());
                return Vec::new();
            }
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect()
        }
        Err(e) => {
            error!("failed to spawn get_current_dns powershell command: {:?}", e);
            Vec::new()
        }
    }
}

/// List active network interfaces on Windows.
pub fn get_active_interfaces() -> Vec<NetworkInterface> {
    let script = "Get-NetAdapter | Where-Object {$_.Status -eq 'Up'} | Select-Object -Property Name,ifIndex | ForEach-Object { \"$($_.Name)|$($_.ifIndex)\" }";

    match Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script])
        .output()
    {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!("get_active_interfaces PowerShell failed: {}", stderr.trim());
                return Vec::new();
            }
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.trim().split('|').collect();
                    if parts.len() == 2 {
                        let name = parts[0].trim().to_string();
                        let index: u32 = parts[1].trim().parse().ok()?;
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
        Err(e) => {
            error!("failed to spawn get_active_interfaces powershell command: {:?}", e);
            Vec::new()
        }
    }
}

/// Detect the default network interface on Windows.
fn detect_default_interface() -> Option<String> {
    let script = r#"$v4 = Get-NetRoute -AddressFamily IPv4 -DestinationPrefix '0.0.0.0/0' -ErrorAction SilentlyContinue | Where-Object { $_.NextHop -ne '' } | Sort-Object { $_.RouteMetric + $_.InterfaceMetric } | Select-Object -First 1 -ExpandProperty InterfaceAlias; $v6 = Get-NetRoute -AddressFamily IPv6 -DestinationPrefix '::/0' -ErrorAction SilentlyContinue | Where-Object { $_.NextHop -ne '' } | Sort-Object { $_.RouteMetric + $_.InterfaceMetric } | Select-Object -First 1 -ExpandProperty InterfaceAlias; if ($v4) { $v4 } elseif ($v6) { $v6 }"#;

    match Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script])
        .output()
    {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!("detect_default_interface PowerShell failed: {}", stderr.trim());
                return None;
            }
            let stdout = String::from_utf8_lossy(&output.stdout);
            let name = stdout.trim().to_string();
            if name.is_empty() {
                None
            } else {
                Some(name)
            }
        }
        Err(e) => {
            error!("failed to spawn detect_default_interface powershell command: {:?}", e);
            None
        }
    }
}

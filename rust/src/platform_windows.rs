// Jetlag Core — Windows DNS implementation.
//
// For the simple CLI approach, we use `netsh` and `powershell` commands.
// For production use, the Windows Service architecture (jetlag_service)
// should be used to avoid UAC prompts — see the plan document.
//
// The full Win32 API approach via SetInterfaceDnsSettings is available
// when compiled with the `windows-dns` feature.

use crate::models::*;
use std::process::Command;

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

    let addresses = format!("'{}','{}'", provider.primary_dns, provider.secondary_dns);
    let script = format!(
        "Set-DnsClientServerAddress -InterfaceAlias '{}' -ServerAddresses ({})",
        iface, addresses
    );

    match Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &script])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                // Flush DNS cache
                let _ = Command::new("ipconfig").args(["/flushdns"]).output();

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
                let error_msg = if !stderr.is_empty() {
                    stderr.to_string()
                } else {
                    stdout.to_string()
                };

                // Check if it's a permission error
                if error_msg.contains("Access is denied")
                    || error_msg.contains("not recognized")
                    || error_msg.contains("requires elevation")
                {
                    DnsChangeResult::failure(
                        DnsChangeMethod::WindowsApi,
                        "Administrator privileges required. The Jetlag DNS Service should handle this automatically.",
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

    let script = format!(
        "Set-DnsClientServerAddress -InterfaceAlias '{}' -ResetServerAddresses",
        iface
    );

    match Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &script])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                let _ = Command::new("ipconfig").args(["/flushdns"]).output();
                DnsChangeResult::success(
                    DnsChangeMethod::WindowsApi,
                    &format!("DNS reset to DHCP defaults on {}", iface),
                )
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                DnsChangeResult::failure(
                    DnsChangeMethod::WindowsApi,
                    &format!("DNS reset failed: {}", stderr.trim()),
                )
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
    let script = match interface_name {
        Some(iface) => format!(
            "Get-DnsClientServerAddress -InterfaceAlias '{}' -AddressFamily IPv4 | Select-Object -ExpandProperty ServerAddresses",
            iface
        ),
        None => "Get-DnsClientServerAddress -AddressFamily IPv4 | Where-Object {{ $_.ServerAddresses.Count -gt 0 }} | Select-Object -First 1 -ExpandProperty ServerAddresses".to_string(),
    };

    match Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &script])
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect()
        }
        Err(_) => Vec::new(),
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
        Err(_) => Vec::new(),
    }
}

/// Detect the default network interface on Windows.
fn detect_default_interface() -> Option<String> {
    let script = "Get-NetAdapter | Where-Object {$_.Status -eq 'Up'} | Where-Object {$_.InterfaceDescription -notmatch 'Virtual|Loopback'} | Select-Object -First 1 -ExpandProperty Name";

    match Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", script])
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let name = stdout.trim().to_string();
            if name.is_empty() {
                None
            } else {
                Some(name)
            }
        }
        Err(_) => None,
    }
}

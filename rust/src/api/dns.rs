// Jetlag Core — DNS API exposed to Flutter via flutter_rust_bridge.
//
// This module provides the high-level functions that the Dart side calls.
// FRB automatically generates Dart bindings for all public functions here.

use crate::models::*;
use crate::providers;

// ============================================================
// Provider registry API
// ============================================================

/// Get all built-in DNS providers.
#[flutter_rust_bridge::frb(sync)]
pub fn get_builtin_providers() -> Vec<DnsProvider> {
    providers::builtin_providers()
}

/// Get a single provider by ID.
#[flutter_rust_bridge::frb(sync)]
pub fn get_provider(id: String) -> Option<DnsProvider> {
    providers::get_provider_by_id(&id)
}

/// Create a custom DNS provider. Returns an error string if validation fails.
#[flutter_rust_bridge::frb(sync)]
pub fn create_custom_provider(
    id: String,
    name: String,
    primary_dns: String,
    secondary_dns: String,
    doh_url: Option<String>,
    dot_hostname: Option<String>,
) -> Result<DnsProvider, String> {
    if !providers::validate_dns_address(&primary_dns) {
        return Err(format!("Invalid primary DNS address: {}", primary_dns));
    }
    if !providers::validate_dns_address(&secondary_dns) {
        return Err(format!("Invalid secondary DNS address: {}", secondary_dns));
    }
    Ok(providers::custom_provider(
        id,
        name,
        primary_dns,
        secondary_dns,
        doh_url,
        dot_hostname,
    ))
}

/// Validate a DNS address (IPv4 or IPv6).
#[flutter_rust_bridge::frb(sync)]
pub fn validate_address(addr: String) -> bool {
    providers::validate_dns_address(&addr)
}

// ============================================================
// Platform DNS operations (desktop only — mobile uses MethodChannel)
// ============================================================

/// Set DNS on the current platform (desktop: Windows/Linux/macOS).
/// On mobile platforms, this returns a failure — use MethodChannel instead.
pub fn set_dns(provider: DnsProvider, interface_name: Option<String>) -> DnsChangeResult {
    #[cfg(target_os = "macos")]
    {
        return crate::platform_macos::set_dns(&provider, interface_name.as_deref());
    }

    #[cfg(all(target_os = "windows", feature = "windows-dns"))]
    {
        return crate::platform_windows::set_dns(&provider, interface_name.as_deref());
    }

    #[cfg(all(target_os = "linux", feature = "linux-dns"))]
    {
        return crate::platform_linux::set_dns(&provider, interface_name.as_deref());
    }

    #[allow(unreachable_code)]
    DnsChangeResult::failure(
        DnsChangeMethod::PrivateDnsSettings,
        "DNS changes on this platform must be done via MethodChannel (mobile) or platform feature not enabled",
    )
}

/// Reset DNS to system defaults on the current platform (desktop only).
pub fn reset_dns(interface_name: Option<String>) -> DnsChangeResult {
    #[cfg(target_os = "macos")]
    {
        return crate::platform_macos::reset_dns(interface_name.as_deref());
    }

    #[cfg(all(target_os = "windows", feature = "windows-dns"))]
    {
        return crate::platform_windows::reset_dns(interface_name.as_deref());
    }

    #[cfg(all(target_os = "linux", feature = "linux-dns"))]
    {
        return crate::platform_linux::reset_dns(interface_name.as_deref());
    }

    #[allow(unreachable_code)]
    DnsChangeResult::failure(
        DnsChangeMethod::PrivateDnsSettings,
        "DNS reset on this platform must be done via MethodChannel (mobile) or platform feature not enabled",
    )
}

/// Get the current system DNS servers (desktop only).
#[flutter_rust_bridge::frb(sync)]
pub fn get_current_dns(interface_name: Option<String>) -> Vec<String> {
    #[cfg(target_os = "macos")]
    {
        return crate::platform_macos::get_current_dns(interface_name.as_deref());
    }

    #[cfg(all(target_os = "windows", feature = "windows-dns"))]
    {
        return crate::platform_windows::get_current_dns(interface_name.as_deref());
    }

    #[cfg(all(target_os = "linux", feature = "linux-dns"))]
    {
        return crate::platform_linux::get_current_dns(interface_name.as_deref());
    }

    #[allow(unreachable_code)]
    Vec::new()
}

/// List active network interfaces (desktop only).
#[flutter_rust_bridge::frb(sync)]
pub fn get_active_interfaces() -> Vec<NetworkInterface> {
    #[cfg(target_os = "macos")]
    {
        return crate::platform_macos::get_active_interfaces();
    }

    #[cfg(all(target_os = "windows", feature = "windows-dns"))]
    {
        return crate::platform_windows::get_active_interfaces();
    }

    #[cfg(all(target_os = "linux", feature = "linux-dns"))]
    {
        return crate::platform_linux::get_active_interfaces();
    }

    #[allow(unreachable_code)]
    Vec::new()
}

// Jetlag Core — Shared data models for DNS configuration.

use serde::{Deserialize, Serialize};

/// Represents a DNS provider with all its configuration details.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DnsProvider {
    /// Unique identifier for the provider (e.g., "cloudflare", "google", "custom_1")
    pub id: String,
    /// Human-readable name (e.g., "Cloudflare")
    pub name: String,
    /// Primary DNS server IPv4 address
    pub primary_dns: String,
    /// Secondary DNS server IPv4 address
    pub secondary_dns: String,
    /// Optional primary IPv6 DNS address
    pub primary_dns_ipv6: Option<String>,
    /// Optional secondary IPv6 DNS address
    pub secondary_dns_ipv6: Option<String>,
    /// DNS-over-HTTPS URL (e.g., "<https://cloudflare-dns.com/dns-query>")
    pub doh_url: Option<String>,
    /// DNS-over-TLS hostname (e.g., "cloudflare-dns.com")
    pub dot_hostname: Option<String>,
    /// Short description of the provider
    pub description: String,
    /// Whether this is a user-defined custom provider
    pub is_custom: bool,
}

/// The current connection status of DNS configuration.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DnsConnectionStatus {
    /// No custom DNS is active; using system defaults
    Disconnected,
    /// DNS change is in progress
    Connecting,
    /// Custom DNS is active and verified
    Connected,
    /// An error occurred while setting DNS
    Error,
}

/// Platform-specific method used to change DNS.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DnsChangeMethod {
    /// Android: Open Private DNS settings for user to configure
    PrivateDnsSettings,
    /// Android: Direct change via root access (settings put global)
    RootAccess,
    /// iOS: NEDNSSettingsManager programmatic profile
    NetworkExtension,
    /// iOS fallback: Open Settings app for manual DNS activation
    SettingsRedirect,
    /// Windows: SetInterfaceDnsSettings via elevated service
    WindowsApi,
    /// Linux: D-Bus call to systemd-resolved
    Resolvectl,
    /// macOS: networksetup CLI command
    Networksetup,
}

/// Represents a network interface on the system.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkInterface {
    /// Interface name (e.g., "Wi-Fi", "eth0", "Ethernet")
    pub name: String,
    /// Interface index (used by Linux D-Bus API)
    pub index: u32,
    /// Whether this interface is currently active/up
    pub is_active: bool,
}

/// The full DNS state exposed to the Flutter UI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DnsState {
    /// Currently selected DNS provider (None if no provider selected)
    pub selected_provider: Option<DnsProvider>,
    /// Current connection status
    pub connection_status: DnsConnectionStatus,
    /// Which method was used to apply DNS (None if disconnected)
    pub active_method: Option<DnsChangeMethod>,
    /// The DNS servers currently configured on the system (if readable)
    pub current_system_dns: Vec<String>,
    /// Whether the device is rooted (Android only, None on other platforms)
    pub is_rooted: Option<bool>,
    /// Error message if status is Error
    pub error_message: Option<String>,
}

impl DnsState {
    /// Create a default disconnected state.
    pub fn disconnected() -> Self {
        Self {
            selected_provider: None,
            connection_status: DnsConnectionStatus::Disconnected,
            active_method: None,
            current_system_dns: Vec::new(),
            is_rooted: None,
            error_message: None,
        }
    }
}

/// Result of a DNS change operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DnsChangeResult {
    /// Whether the operation succeeded
    pub success: bool,
    /// The method that was used
    pub method: DnsChangeMethod,
    /// Human-readable message
    pub message: String,
    /// If the user needs to manually complete a step (e.g., iOS Settings activation)
    pub requires_user_action: bool,
    /// Instructions for the user if manual action is needed
    pub user_action_message: Option<String>,
}

impl DnsChangeResult {
    pub fn success(method: DnsChangeMethod, message: &str) -> Self {
        Self {
            success: true,
            method,
            message: message.to_string(),
            requires_user_action: false,
            user_action_message: None,
        }
    }

    pub fn success_with_action(
        method: DnsChangeMethod,
        message: &str,
        action_message: &str,
    ) -> Self {
        Self {
            success: true,
            method,
            message: message.to_string(),
            requires_user_action: true,
            user_action_message: Some(action_message.to_string()),
        }
    }

    pub fn failure(method: DnsChangeMethod, message: &str) -> Self {
        Self {
            success: false,
            method,
            message: message.to_string(),
            requires_user_action: false,
            user_action_message: None,
        }
    }
}

/// Errors that can occur during DNS operations.
#[derive(Debug, thiserror::Error)]
pub enum DnsError {
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Interface not found: {0}")]
    InterfaceNotFound(String),

    #[error("Platform not supported: {0}")]
    PlatformNotSupported(String),

    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    #[error("D-Bus error: {0}")]
    DbusError(String),

    #[error("Windows API error: {0}")]
    WindowsApiError(String),

    #[error("Invalid DNS address: {0}")]
    InvalidAddress(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_state_disconnected() {
        let state = DnsState::disconnected();
        assert_eq!(state.connection_status, DnsConnectionStatus::Disconnected);
        assert!(state.selected_provider.is_none());
        assert!(state.active_method.is_none());
    }

    #[test]
    fn test_dns_change_result_success() {
        let result = DnsChangeResult::success(DnsChangeMethod::Resolvectl, "DNS set");
        assert!(result.success);
        assert!(!result.requires_user_action);
    }

    #[test]
    fn test_dns_change_result_with_action() {
        let result = DnsChangeResult::success_with_action(
            DnsChangeMethod::NetworkExtension,
            "DNS profile installed",
            "Please enable the DNS profile in Settings",
        );
        assert!(result.success);
        assert!(result.requires_user_action);
        assert!(result.user_action_message.is_some());
    }

    #[test]
    fn test_dns_provider_serialization() {
        let provider = DnsProvider {
            id: "cloudflare".to_string(),
            name: "Cloudflare".to_string(),
            primary_dns: "1.1.1.1".to_string(),
            secondary_dns: "1.0.0.1".to_string(),
            primary_dns_ipv6: Some("2606:4700:4700::1111".to_string()),
            secondary_dns_ipv6: Some("2606:4700:4700::1001".to_string()),
            doh_url: Some("https://cloudflare-dns.com/dns-query".to_string()),
            dot_hostname: Some("cloudflare-dns.com".to_string()),
            description: "Fast & private".to_string(),
            is_custom: false,
        };
        let json = serde_json::to_string(&provider).unwrap();
        let deserialized: DnsProvider = serde_json::from_str(&json).unwrap();
        assert_eq!(provider, deserialized);
    }
}

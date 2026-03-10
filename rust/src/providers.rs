// Jetlag Core — Built-in DNS provider registry.

use crate::models::DnsProvider;

/// Returns all built-in DNS providers.
pub fn builtin_providers() -> Vec<DnsProvider> {
    vec![
        cloudflare(),
        google(),
        adguard(),
        adguard_family(),
        quad9(),
        opendns(),
        opendns_family(),
    ]
}

/// Get a built-in provider by its ID. Returns None if not found.
pub fn get_provider_by_id(id: &str) -> Option<DnsProvider> {
    builtin_providers().into_iter().find(|p| p.id == id)
}

/// Cloudflare DNS — 1.1.1.1
pub fn cloudflare() -> DnsProvider {
    DnsProvider {
        id: "cloudflare".to_string(),
        name: "Cloudflare".to_string(),
        primary_dns: "1.1.1.1".to_string(),
        secondary_dns: "1.0.0.1".to_string(),
        primary_dns_ipv6: Some("2606:4700:4700::1111".to_string()),
        secondary_dns_ipv6: Some("2606:4700:4700::1001".to_string()),
        doh_url: Some("https://cloudflare-dns.com/dns-query".to_string()),
        dot_hostname: Some("one.one.one.one".to_string()),
        description: "Fast, privacy-first DNS by Cloudflare. No logging of your IP address."
            .to_string(),
        is_custom: false,
    }
}

/// Google Public DNS — 8.8.8.8
pub fn google() -> DnsProvider {
    DnsProvider {
        id: "google".to_string(),
        name: "Google".to_string(),
        primary_dns: "8.8.8.8".to_string(),
        secondary_dns: "8.8.4.4".to_string(),
        primary_dns_ipv6: Some("2001:4860:4860::8888".to_string()),
        secondary_dns_ipv6: Some("2001:4860:4860::8844".to_string()),
        doh_url: Some("https://dns.google/dns-query".to_string()),
        dot_hostname: Some("dns.google".to_string()),
        description: "Google's reliable public DNS with global infrastructure.".to_string(),
        is_custom: false,
    }
}

/// AdGuard DNS — Default (ad-blocking)
pub fn adguard() -> DnsProvider {
    DnsProvider {
        id: "adguard".to_string(),
        name: "AdGuard DNS".to_string(),
        primary_dns: "94.140.14.14".to_string(),
        secondary_dns: "94.140.15.15".to_string(),
        primary_dns_ipv6: Some("2a10:50c0::ad1:ff".to_string()),
        secondary_dns_ipv6: Some("2a10:50c0::ad2:ff".to_string()),
        doh_url: Some("https://dns.adguard-dns.com/dns-query".to_string()),
        dot_hostname: Some("dns.adguard-dns.com".to_string()),
        description: "Blocks ads, trackers, and malware domains. Privacy-focused.".to_string(),
        is_custom: false,
    }
}

/// AdGuard DNS — Family Protection
pub fn adguard_family() -> DnsProvider {
    DnsProvider {
        id: "adguard_family".to_string(),
        name: "AdGuard Family".to_string(),
        primary_dns: "94.140.14.15".to_string(),
        secondary_dns: "94.140.15.16".to_string(),
        primary_dns_ipv6: Some("2a10:50c0::bad1:ff".to_string()),
        secondary_dns_ipv6: Some("2a10:50c0::bad2:ff".to_string()),
        doh_url: Some("https://family.adguard-dns.com/dns-query".to_string()),
        dot_hostname: Some("family.adguard-dns.com".to_string()),
        description: "AdGuard with safe search and adult content blocking for families."
            .to_string(),
        is_custom: false,
    }
}

/// Quad9 DNS — 9.9.9.9
pub fn quad9() -> DnsProvider {
    DnsProvider {
        id: "quad9".to_string(),
        name: "Quad9".to_string(),
        primary_dns: "9.9.9.9".to_string(),
        secondary_dns: "149.112.112.112".to_string(),
        primary_dns_ipv6: Some("2620:fe::fe".to_string()),
        secondary_dns_ipv6: Some("2620:fe::9".to_string()),
        doh_url: Some("https://dns.quad9.net/dns-query".to_string()),
        dot_hostname: Some("dns.quad9.net".to_string()),
        description:
            "Security-focused DNS that blocks malicious domains. Non-profit backed."
                .to_string(),
        is_custom: false,
    }
}

/// OpenDNS (Cisco) — Standard
pub fn opendns() -> DnsProvider {
    DnsProvider {
        id: "opendns".to_string(),
        name: "OpenDNS".to_string(),
        primary_dns: "208.67.222.222".to_string(),
        secondary_dns: "208.67.220.220".to_string(),
        primary_dns_ipv6: Some("2620:119:35::35".to_string()),
        secondary_dns_ipv6: Some("2620:119:53::53".to_string()),
        doh_url: Some("https://doh.opendns.com/dns-query".to_string()),
        dot_hostname: None, // OpenDNS DoT not widely documented
        description: "Cisco's OpenDNS with built-in phishing and content filtering.".to_string(),
        is_custom: false,
    }
}

/// OpenDNS FamilyShield
pub fn opendns_family() -> DnsProvider {
    DnsProvider {
        id: "opendns_family".to_string(),
        name: "OpenDNS FamilyShield".to_string(),
        primary_dns: "208.67.222.123".to_string(),
        secondary_dns: "208.67.220.123".to_string(),
        primary_dns_ipv6: None,
        secondary_dns_ipv6: None,
        doh_url: Some("https://doh.familyshield.opendns.com/dns-query".to_string()),
        dot_hostname: None,
        description: "Pre-configured to block adult content. No account required.".to_string(),
        is_custom: false,
    }
}

/// Create a custom DNS provider from user input.
pub fn custom_provider(
    id: String,
    name: String,
    primary_dns: String,
    secondary_dns: String,
    doh_url: Option<String>,
    dot_hostname: Option<String>,
) -> DnsProvider {
    DnsProvider {
        id,
        name,
        primary_dns,
        secondary_dns,
        primary_dns_ipv6: None,
        secondary_dns_ipv6: None,
        doh_url,
        dot_hostname,
        description: "User-defined custom DNS server.".to_string(),
        is_custom: true,
    }
}

/// Validate an IPv4 address format.
pub fn validate_ipv4(addr: &str) -> bool {
    addr.parse::<std::net::Ipv4Addr>().is_ok()
}

/// Validate an IPv6 address format (basic validation).
pub fn validate_ipv6(addr: &str) -> bool {
    addr.parse::<std::net::Ipv6Addr>().is_ok()
}

/// Validate a DNS address (IPv4 or IPv6).
pub fn validate_dns_address(addr: &str) -> bool {
    validate_ipv4(addr) || validate_ipv6(addr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_providers_count() {
        let providers = builtin_providers();
        assert_eq!(providers.len(), 7);
    }

    #[test]
    fn test_get_provider_by_id() {
        assert!(get_provider_by_id("cloudflare").is_some());
        assert!(get_provider_by_id("google").is_some());
        assert!(get_provider_by_id("nonexistent").is_none());
    }

    #[test]
    fn test_no_builtin_is_custom() {
        for provider in builtin_providers() {
            assert!(!provider.is_custom, "Provider {} should not be custom", provider.name);
        }
    }

    #[test]
    fn test_custom_provider() {
        let p = custom_provider(
            "my_dns".into(),
            "My DNS".into(),
            "10.0.0.1".into(),
            "10.0.0.2".into(),
            None,
            None,
        );
        assert!(p.is_custom);
        assert_eq!(p.id, "my_dns");
    }

    #[test]
    fn test_validate_ipv4() {
        assert!(validate_ipv4("1.1.1.1"));
        assert!(validate_ipv4("255.255.255.255"));
        assert!(validate_ipv4("0.0.0.0"));
        assert!(!validate_ipv4("256.1.1.1"));
        assert!(!validate_ipv4("1.1.1"));
        assert!(!validate_ipv4("abc.def.ghi.jkl"));
        assert!(!validate_ipv4(""));
    }

    #[test]
    fn test_validate_ipv6() {
        assert!(validate_ipv6("2606:4700:4700::1111"));
        assert!(validate_ipv6("::1"));
        assert!(!validate_ipv6("not-an-ipv6"));
        assert!(!validate_ipv6("1.1.1.1"));
    }

    #[test]
    fn test_validate_dns_address() {
        assert!(validate_dns_address("1.1.1.1"));
        assert!(validate_dns_address("2606:4700:4700::1111"));
        assert!(!validate_dns_address("not-valid"));
    }
}

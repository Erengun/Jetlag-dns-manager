// Jetlag Core — Cross-platform DNS configuration library.
//
// This crate provides the shared DNS provider definitions, configuration types,
// and platform-specific DNS change implementations for the Jetlag DNS Manager.

mod frb_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */

pub mod api;
pub mod models;
pub mod providers;

#[cfg(all(target_os = "windows", feature = "windows-dns"))]
pub mod platform_windows;

#[cfg(all(target_os = "linux", feature = "linux-dns"))]
pub mod platform_linux;

#[cfg(target_os = "macos")]
pub mod platform_macos;

// Re-exports for convenience
pub use models::*;
pub use providers::*;

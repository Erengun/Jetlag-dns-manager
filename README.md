[![Stand With Palestine](https://raw.githubusercontent.com/TheBSD/StandWithPalestine/main/banner-no-action.svg)](https://thebsd.github.io/StandWithPalestine)
# Jetlag DNS Manager

A cross-platform Flutter application to easily change your DNS settings on Android, iOS, Windows, and Linux. Enhance your privacy and security by using trusted public DNS providers like Cloudflare, Google, AdGuard, and more.

## Features

*   **Cross-Platform Support**: Runs on Android, iOS, Windows, and Linux.
*   **Adaptive UI**: Automatically switches between Material Design (Android/Desktop) and Cupertino (iOS) for a native feel.
*   **No Root Required (Android)**: Works on non-rooted devices by guiding users to system settings (Android 9+). Supports direct changes on rooted devices.
*   **Secure DNS**: Supports DNS-over-HTTPS (DoH) and DNS-over-TLS (DoT) where applicable.
*   **State Management**: Built with `flutter_riverpod` for robust and testable state management.
*   **Extensible**: Easily add custom DNS providers.

## Getting Started

### Prerequisites

*   Flutter SDK (3.10.0 or later)
*   Dart SDK
*   Platform-specific development tools (Android Studio, Xcode, Visual Studio, etc.)

### Installation

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/Erengun/jetlag_dns_manager.git
    cd jetlag_dns_manager
    ```

2.  **Install dependencies:**
    ```bash
    flutter pub get
    ```

3.  **Run the code generator (for Riverpod):**
    ```bash
    dart run build_runner build --delete-conflicting-outputs
    ```

### Platform-Specific Setup

#### Android
*   **Non-Rooted**: The app uses a `MethodChannel` to open the Android Private DNS settings. No special permissions required.
*   **Rooted**: Requires root access to execute `su` commands for direct DNS setting.

#### iOS
*   **Entitlements**: This app requires the **Network Extension** entitlement (`com.apple.developer.networking.networkextension`) to manage DNS settings programmatically.
*   **Provisioning**: You must have a paid Apple Developer account to use this entitlement.
*   **Configuration**: The `NEDNSSettingsManager` is implemented in `AppDelegate.swift`.

#### Windows
*   **Permissions**: The app executes PowerShell scripts to change DNS settings. This triggers a UAC prompt for Administrator privileges when connecting.

#### Linux
*   **Dependencies**: Relies on `resolvectl` (part of `systemd-resolved`). Ensure your distribution uses `systemd-resolved`.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## Contact

- [https://www.erengun.dev](https://www.erengun.dev)

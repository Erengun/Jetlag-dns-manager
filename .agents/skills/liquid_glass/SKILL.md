---
name: liquid-glass
description: Expert guidance for implementing the "Liquid Glass" design language (iOS 26 Style) in Flutter. Focuses on translucency, spatial depth, "ExtendBody" scroll management, and the "Extra Button" contextual action pattern.
metadata:
  version: "1.0"
---

# Liquid Glass Design System

## Overview

Liquid Glass is a design philosophy (introduced in iOS 26) that prioritizes translucency and spatial depth to create a sense of openness in mobile applications. By making UI "chrome" (App Bars, Bottom Bars) translucent and allowing content to flow behind them, the application feels more spacious and "alive." This skill provides the architectural blueprints for implementing these patterns in Flutter.

## Quick Reference

**Core Principles:**
1. **Transparency**: UI elements are overlays, not walls.
2. **Spatial Depth**: Content flows edge-to-edge behind bars.
3. **Contextual Action**: Navigation bars morph to accommodate screen-specific actions.

**The "ExtendBody" Rule:**
Always use `Scaffold(extendBody: true)` to inflate `MediaQuery.padding.bottom` and allow content to scroll behind the bottom bar.

**Scroll Spacer Pattern (for CustomScrollView):**
```dart
SliverToBoxAdapter(
  child: SizedBox(height: MediaQuery.paddingOf(context).bottom),
)
```

## Layout & Scroll Management

To achieve the glass effect, content must exist *underneath* the UI bars.

### Using Scaffold(extendBody: true)
Setting `extendBody: true` tells the Scaffold to extend its body to the bottom of the screen, even if a `bottomNavigationBar` is present. It also inflates the `MediaQuery.padding.bottom` to match the bar's height.

*   **ListView/GridView**: Automatically respect the inflated padding.
*   **CustomScrollView**: **Ignores** it. You must manually add a bottom spacer sliver.
*   **SingleChildScrollView**: Requires manual padding inside its child.

## Component Implementation

### 1. Independent Overlay App Bar
Avoid `SliverAppBar` for complex Liquid Glass effects because it ties foreground and background heights together. Instead, use a `Stack` at the Scaffold level.

**Layering Strategy:**
1. **Scroll View**: The main content.
2. **Darkening Overlay**: A `LinearGradient` that fades in as the user scrolls.
3. **Foreground**: The Title, Back button, and Actions.

**The "No Hard Edge" Gradient:**
Use non-linear stops for a natural falloff:
```dart
LinearGradient(
  stops: [0.0, 0.3, 0.5, 0.65, 0.8],
  colors: [
    for (final alpha in [1.0, 0.98, 0.64, 0.19, 0.0])
      baseColor.withValues(alpha: alpha * scrollProgress),
  ],
)
```

### 2. The Morphing Bottom Bar
The bar is composed of two independent containers in a `Row`: the **Tabs Area** and the **Extra Button**.

**Morphing Rules:**
*   **Geometry Only**: Animate `width` and `borderRadius`. Never animate glass `opacity` (causes blur artifacts).
*   **Content Gap Crossfade**:
    *   Old content fades out: 0.0 → 0.4
    *   Gap (empty): 0.4 → 0.6
    *   New content fades in: 0.6 → 1.0
*   **Physics**: Use `CupertinoMotion` with `duration: 350ms` and `bounce: 0.25`.

### 3. Syncing Blur (Pinned Headers)
When a `SliverPersistentHeader` pins, it should signal the App Bar to also activate blur.
*   Use `overlapsContent` from `SliverPersistentHeaderDelegate`.
*   Communicate state via a `ChangeNotifier` to the Scaffold.

## Best Practices

### Design Principles
*   **Never box content**: Avoid opaque horizontal bars.
*   **Use Gradients for Fading**: Simple gradients are often better than heavy shaders for "no hard edge" effects.
*   **Platform Mental Models**: Remember that iOS handles scroll insets automatically (`adjustedContentInset`), while Flutter requires manual coordination for low-level widgets.

### Implementation Guidelines
*   **Centralize with Wrappers**: Create a `LiquidGlassScaffold` that handles the `CustomScrollView` bottom spacer and App Bar layering automatically.
*   **Primitive Abstraction**: Keep your rendering engine (like `liquid_glass_renderer`) behind a clean boundary so it can be swapped if Flutter adds official support.
*   **Position FABs Precisely**: Set `resizeToAvoidBottomInset: false` and position FABs using the inflated `MediaQuery.padding.bottom` so they sit above the bar and shift with the keyboard.

## Resources

### References
*   [GitHub Issue #105027](https://github.com/flutter/flutter/issues/105027) - Discussion on Scaffold padding and CustomScrollView.
*   `liquid_glass_renderer` - Community package for glass primitives.
*   `motor` - Package for `CupertinoMotion` spring physics.

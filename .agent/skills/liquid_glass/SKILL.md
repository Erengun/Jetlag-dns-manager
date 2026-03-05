---
name: flutter-liquid-glass-architecture
description: Trigger this skill when tasked with building translucent UI components, "Liquid Glass" bottom navigation bars (iOS 26 style), or resolving scrollable content getting trapped behind bottom bars in Flutter.
---

# Context & Philosophy
This skill implements the "Liquid Glass" pattern to eliminate claustrophobic UI (e.g., stacked opaque app bars, persistent footers, and bottom navs taking up ~170px of screen space). By using translucent overlays and merging contextual actions into an expandable "extra button" in the nav bar, content flows edge-to-edge, making the screen feel significantly larger.

# Architecture & Layout Rules

## 1. Scaffold vs. Stack (The Scroll Trap)
**Never use a `Stack` to overlay a translucent bottom bar.** * `Stack` is a pure layout widget and does not modify `MediaQuery`. Scroll views will only see the physical device safe area (e.g., the iOS home indicator) and will trap the bottom content behind the glass.
* **Mandatory Approach:** Always use `Scaffold` with `extendBody: true` and place the custom bar in the `bottomNavigationBar` slot.
* *Why:* Flutter's internal `_BodyBuilder` recalculates `MediaQuery.padding.bottom` using `math.max(metrics.padding.bottom, bodyConstraints.bottomWidgetsHeight)`. This natively inflates the padding so the framework knows the bar exists.

## 2. Scroll View Handling (Crucial)
Different scroll views react differently to the inflated `MediaQuery.padding.bottom`. You must implement the correct handling based on the widget type:

### `ListView` & `GridView` (BoxScrollView)
* **Action:** No manual intervention needed.
* **Mechanism:** They extend `BoxScrollView`. Inside `buildSlivers()`, Flutter automatically reads the inflated `MediaQuery.padding`, wraps the content in `SliverPadding`, and injects a new `MediaQuery` for descendants with the consumed padding removed. Content scrolls past the glass automatically.

### `CustomScrollView`
* **Action:** You MUST manually inject a bottom spacer.
* **Mechanism:** `CustomScrollView.buildSlivers()` literally returns the slivers exactly as passed. It deliberately ignores `MediaQuery.padding` to give developers low-level control (for pinned headers, etc.).
* **Implementation:** Always append this exact sliver to the end of your `CustomScrollView` slivers list:
  ```dart
  SliverToBoxAdapter(
    child: SizedBox(height: MediaQuery.paddingOf(context).bottom),
  )
	```

## 3. The Morphing Bottom Bar Layout

Do not build the bottom bar as a single monolithic pill. It must be split to handle dynamic contextual actions (like replacing a persistent footer).

* **Structure:** Use a `Row` containing two separate glass containers (e.g., `LiquidGlassLayer`):
1. `MorphingTabsArea` (Standard navigation icons)
2. `MorphingExtraButton` (Contextual actions)


* **Animation:** The total `Row` width remains constant. Drive the `width` of both child containers using a single, shared spring animation value so they resize synchronously (one shrinks while the other grows).
* **Extra Button Modes:** The `MorphingExtraButton` should listen to route changes and render one of three states depending on context:
1. *Default:* A simple icon button.
2. *Search Mode:* Expands into a full search bar visual.
3. *Action Mode:* Expands into segmented contextual buttons (e.g., Buy/Sell).



# Rules & Constraints

* **Decouple Rendering:** Keep the complex morphing layout logic strictly separated from the glass rendering primitives. Use external packages (like `liquid_glass_renderer`) or a dedicated internal rendering layer for the actual blur/frost effect.
* **No `SafeArea` Wrappers on Body:** Do not wrap the `Scaffold.body` in a `SafeArea` widget. This defeats the purpose of `extendBody: true` and will prevent the content from flowing behind the glass.
* **Consistent Padding:** Rely exclusively on `MediaQuery.paddingOf(context).bottom` for spacing calculations related to the bottom bar; do not hardcode pixel values for bottom padding.

# Code Example: The Core Shell

```dart
Scaffold(
  extendBody: true, // CRITICAL
  resizeToAvoidBottomInset: false,
  bottomNavigationBar: Row(
    children: [
      SizedBox(width: tabsWidth, child: MorphingTabsArea()),
      const SizedBox(width: 8), 
      SizedBox(width: extraButtonWidth, child: MorphingExtraButton()),
    ],
  ),
  body: CustomScrollView(
    slivers: [
      // ... your content slivers ...
      
      // CRITICAL: Manual padding for CustomScrollView
      SliverToBoxAdapter(
        child: SizedBox(height: MediaQuery.paddingOf(context).bottom),
      ),
    ],
  ),
)

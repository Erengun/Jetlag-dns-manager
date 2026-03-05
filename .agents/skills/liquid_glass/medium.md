**We Shipped Liquid Glass in Our Flutter Fintech App. Here’s What We Learned**
==============================================================================

_Migrating to iOS 26’s design language, rethinking persistent footers, and the scroll padding gotchas nobody warned us about._
------------------------------------------------------------------------------------------------------------------------------

[![Furkan Acar](https://miro.medium.com/v2/resize:fill:64:64/1*y75scInv2dx_79h2sR14dg.jpeg)](https://medium.com/@furkanacardev?source=post_page---byline--ba381c13e3cc---------------------------------------)

[Furkan Acar](https://medium.com/@furkanacardev?source=post_page---byline--ba381c13e3cc---------------------------------------)

18 min read

·

Feb 18, 2026

[nameless link](https://medium.com/m/signin?actionUrl=https%3A%2F%2Fmedium.com%2F_%2Fvote%2Fp%2Fba381c13e3cc&operation=register&redirect=https%3A%2F%2Fmedium.com%2F%40furkanacardev%2Fwe-shipped-liquid-glass-in-our-flutter-fintech-app-heres-what-we-learned-ba381c13e3cc&user=Furkan+Acar&userId=3ab703644454&source=---header_actions--ba381c13e3cc---------------------clap_footer------------------)

--

1

[nameless link](https://medium.com/m/signin?actionUrl=https%3A%2F%2Fmedium.com%2F_%2Fbookmark%2Fp%2Fba381c13e3cc&operation=register&redirect=https%3A%2F%2Fmedium.com%2F%40furkanacardev%2Fwe-shipped-liquid-glass-in-our-flutter-fintech-app-heres-what-we-learned-ba381c13e3cc&source=---header_actions--ba381c13e3cc---------------------bookmark_footer------------------)

Listen

Share

When Apple introduced Liquid Glass at WWDC 2025, the Flutter community responded fast. Blog posts analyzing the implications, proof-of-concept implementations, community packages exploring different approaches. Good work was happening. But we were in a different position, we were a startup in the middle of a redesign, staring at some screens that felt like they were crowded.

This article isn’t a tutorial on how to build individual Liquid Glass components. The community has already done great work there with articles and packages that address a lot of the common challenges. We chose [_liquid_glass_renderer_](https://pub.dev/packages/liquid_glass_renderer) for our glass components and it served us well (thanks to [Tim](https://github.com/timcreatedit) for the work on it). You can use whatever library fits your needs. What we needed were the glass primitives, and _liquid_glass_renderer_ provided those nicely. The more complex elements, like the bottom nav bar with its morphing extra button, we built ourselves. That actually worked out well for us, it made it easier to customize things like the extra button behavior that we’ll talk about later, and we have plans to push it further. This article is more about how our migration went, the architectural decisions we made, and the problems we ran into along the way. If you’re looking for details on the glass components themselves, I’d recommend checking out the [_liquid_glass_renderer_](https://pub.dev/packages/liquid_glass_renderer) _package._

There were things about Fennel, our fintech app, that just weren’t sitting right with us. Three opaque horizontal bars stacked on some screens: an app bar at the top, persistent footer buttons in the middle, and a bottom navigation bar at the bottom. On a 6.1-inch phone, that’s roughly 170 pixels (it could be more or less) of solid chrome. The content area was shrinking and the whole thing felt claustrophobic.

![Three opaque bars stacking on screen. 56px app bar, persistent footer with Buy/Sell, and bottom nav. Content area squeezed in between.](https://miro.medium.com/v2/resize:fit:1196/format:webp/1*CIGEPSm1hGybAU-dj0cPmA.png)

We were already planning a redesign when iOS 26 dropped, and two things clicked:

1. **The transparency philosophy:** Making bars translucent means content flows behind them. Even with the same number of UI elements, the screen _feels_ bigger because there are no visual walls.

2. **The extra button pattern:** iOS 26 apps were merging contextual actions into the nav bar itself. A search icon that expands into a search bar. A button that morphs into controls. This could replace some of our persistent footers.

We decided to fold Liquid Glass into the redesign we were already doing. Two birds, one stone. Here’s how it went.

### **Part 1: The Problem — Three Walls and No Room to Breathe**

**Persistent Footers Were the Worst Offender**

In our company detail screen, we had Buy and Sell buttons as persistent footer widgets sitting above the bottom navigation bar. In the explore screen, we had search-related actions. Multiple screens across the app are used `_persistentFooterButtons_` or equivalent patterns to keep important actions visible.

The problem wasn’t just pixel count, it was the _feeling_. Every time you opened a screen, you were greeted by layers of opaque bars framing a small window of actual content. On smaller devices, it was worse. Users were scrolling through content that felt boxed in.

**The iOS 26 Insight**

I’m not the biggest fan of iOS 26’s overall direction, but I’ll give it this: seeing how those apps handled contextual actions made us rethink where ours should live.

Apple’s approach: if a screen needs a special action (like search), don’t add another bar. Put an extra button next to the tab bar. When the user needs it, the button expands, the tabs shrink, and the extra space becomes your action area. When they leave that screen, it collapses back. The traditional bottom nav bar is becoming more of an action bar.

![captionless image](https://miro.medium.com/v2/resize:fit:998/format:webp/1*Dm4yW0mKhMTd0hOr3zIsGw.gif)![Default tab bar state with small extra button vs. Expanded state where tabs collapse and extra button fills remaining space with search/Buy-Sell controls](https://miro.medium.com/v2/resize:fit:592/format:webp/1*lGYk4VfXlRnOQ7UZQ_8H7w.gif)

Of course, not every persistent footer belongs in the extra button, overloading it would hurt the experience. But for high-traffic content pages like company detail (Buy/Sell) or Explore (search), it was a perfect fit. For the rest, we’d need a different approach (Part 5).

**The Transparency Dividend**

Removing bars was only part of it. Making the remaining bars transparent had a compounding effect on the whole layout:

Even though we still have UI elements at the top and bottom, the user _perceives_ more space because the content extends edge-to-edge behind translucent surfaces. The bars become overlays on the content rather than walls around it.

To be clear, Apple didn’t invent any of this, translucent/glass components and expandable buttons have been around. But seeing it all come together in a system-wide design language was the nudge we needed to rethink our own layout.

— -

**Part 2: The Bottom Nav Bar Scroll Problem**

When you make a bottom navigation bar translucent, your content needs to extend behind it. That’s the whole point. Content scrolls behind the glass bar, creating the depth effect. But if you’re migrating an existing app to this pattern, be careful: _depending on how you implement it, your content can get trapped behind the bar_**.**

**The Problem**

There are two common approaches to make content extend behind a translucent bottom bar in Flutter, and they behave differently:

**Approach 1: Stack.** You put your scrollable content and bottom bar in a `_Stack_`. The content takes the full screen, and the bar is positioned at the bottom with padding. The problem is that `_Stack_` is a pure layout widget, and it doesn’t modify `_MediaQuery_`. Your scroll views only see `_MediaQuery.padding.bottom_` as the device safe area, not the bar height.

Where does this value come from? `_MediaQuery.padding_` is propagated by the OS. It represents the physical safe area the system reserves so UI doesn’t overlap hardware features (home indicator, notch, etc.). When you use a`_Stack_`, this value is all your scroll views see. They have no idea there’s an additional translucent bar sitting on top of them. Every scroll view type `(_ListView_, _CustomScrollView_, _SingleChildScrollView, …_)` will have content stuck behind the bar. You could override `MediaQuery.padding.bottom` yourself inside the Stack, but honestly, there’s no reason to when Scaffold already does it for you.

**Approach 2: Scaffold with** `**extendBody: true**`**.** You put your bar in the `_bottomNavigationBar_` slot and set `extendBody: true`. Scaffold inflates `MediaQuery.padding.bottom` from the device safe area to include the additional bar height (we’ll look at the exact source code below). Your scroll views now _know_ the bar exists through `MediaQuery`. But not all of them act on it:

- `ListView` and `GridView` auto-read the inflated padding and add bottom space. Content scrolls past the bar without you doing anything.

- `CustomScrollView` ignores it. Content gets trapped behind the bar.

![captionless image](https://miro.medium.com/v2/resize:fit:1002/format:webp/1*HGP6wU2XipahGq2D0bB6Lw.png)![ListView with extendBody: true content scrolls fully past the bar & CustomScrollView with extendBody: true last items stuck behind the bar](https://miro.medium.com/v2/resize:fit:1002/format:webp/1*zmNbKQI6nAo2Oj6spTVpxg.png)

You might use a different library or a custom approach entirely, but the underlying issue is the same. If your scroll view doesn’t read `MediaQuery.padding` and add bottom spacing automatically, the content will get trapped behind the bar. This isn’t specific to `Scaffold` or `Stack`. Any approach that overlays a translucent bar on top of scrollable content will hit this if the scroll view doesn’t account for the overlap.

The user can see the last item through the glass, but they can’t scroll far enough to read it fully. `CustomScrollView` doesn’t read `MediaQuery.padding`. It returns your slivers exactly as you passed them, no automatic bottom spacing.

**How Apple Solves This**

In native iOS, this problem doesn’t exist. Apple controls the entire stack and built the solution into the framework.

**UIKit:** `UITabBarController` propagates safe area insets to its child view controllers. `UIScrollView` has a property called `adjustedContentInset`a read-only, system-calculated value that combines the developer-set `contentInset` with safe area insets. The key property is `contentInsetAdjustmentBehavior` which defaults to `.automatic`. When a scroll view is the content view of a view controller inside a tab or navigation controller, `.automatic` applies full inset adjustment. The scroll view adds extra space at the bottom so content can scroll past the bar. No code from the developer.

And `.automatic` isn’t a blind “always apply” flag. It’s context-aware, checking the view hierarchy and applying adjustments based on the scroll view’s position relative to bar controllers. Outside of navigation or tab bar controller contexts, `.automatic` behaves similarly to `.scrollableAxes`, adjusting only the scrollable axes.

**SwiftUI:** `TabView` automatically propagates safe area insets. `ScrollView`, `List`, and `Form` automatically respect them. For custom overlays, Apple provides `safeAreaInset(edge: .bottom)`, a single modifier that simultaneously places a view at an edge AND injects safe area insets into the layout system. One primitive does both jobs.

Here’s what matters: `contentInsetAdjustmentBehavior` lives on `UIScrollView`, the **base class**. Every scroll view subclass inherits this behavior: `UITableView`, `UICollectionView`, `UITextView`, even custom subclasses. Write a scroll view in iOS, and it already knows how to handle bars. You don’t opt in per screen. You don’t add spacers.

**How Flutter’s Scaffold Handles This**

Flutter’s answer is `Scaffold(extendBody: true)` combined with `bottomNavigationBar`. You could also do this with `SafeArea` and overriding `MediaQuery.padding.bottom` manually, but Scaffold already handles it for you. When you put your glass bar in the `bottomNavigationBar` slot and set `extendBody: true`, Scaffold overrides the `MediaQuery` for the body internally.

If you dig into `Scaffold`’s source code, there’s a private widget called `_BodyBuilder` (scaffold.dart). When `extendBody` is true, it recalculates the bottom padding:

```
// From Flutter's Scaffold._BodyBuilder (scaffold.dart):
final double bottom = extendBody
? math.max(metrics.padding.bottom, bodyConstraints.bottomWidgetsHeight)
: metrics.padding.bottom;
```

It inflates `MediaQuery.padding.bottom` to be `max(deviceSafeArea, barHeight)`. If the device’s home indicator is 34px and your bar is 80px, descendants see `padding.bottom = 80.` If the device's safe area is somehow larger than the bar, that wins instead.

This is kind of Flutter’s equivalent of iOS’s `adjustedContentInset`. The framework handles it, you don’t need to build a custom system.

In practice, the shell looks like this:

```
Scaffold(
extendBody: true,
bottomNavigationBar: liquidGlassBottomBar,
body: shellContent, // We use GoRouter's ShellRoute here, but any navigation approach works
)
```

Scaffold handles overlay positioning and padding injection in a single widget.

**What Flutter Auto-Pads (and What It Doesn’t)**

Not all scroll views respond to the inflated `MediaQuery.padding.bottom` the same way. This is where the per-screen effort varies.

`**ListView**` **and** `**GridView**` **— automatic:**

These extend `[_BoxScrollView_](https://github.com/flutter/flutter/blob/master/packages/flutter/lib/src/widgets/scroll_view.dart)`, whose `buildSlivers()` method checks `MediaQuery.padding` when you haven’t provided explicit padding:

```
// From BoxScrollView.buildSlivers() - scroll_view.dart
// When padding is null (the default):
final MediaQueryData? mediaQuery = MediaQuery.maybeOf(context);
if (mediaQuery != null) {
// For vertical scroll, consume top+bottom padding:
final EdgeInsets mediaQueryVerticalPadding =
mediaQuery.padding.copyWith(left: 0.0, right: 0.0);
effectivePadding = mediaQueryVerticalPadding;
}
// …wraps content in SliverPadding(padding: effectivePadding)
```

It reads the inflated `padding.bottom` (which now includes the bar height) and wraps the content in `SliverPadding`. It also wraps children in a new `MediaQuery` with the consumed padding removed so descendants don’t double-apply the same padding. Content scrolls past the bar automatically, no per-screen code needed.

`**CustomScrollView**` **— completely manual:**

`CustomScrollView.buildSlivers()` (`scroll_view.dart`) is literally one line:

```
// From CustomScrollView.buildSlivers() - scroll_view.dart:
List<Widget> buildSlivers(BuildContext context) => slivers;
```

No `MediaQuery` reading. No `SliverPadding` wrapping. Your slivers are returned exactly as you passed them. If the last sliver sits at the bottom of the screen, it’s behind the bar. You need a manual spacer: a widget that reads `MediaQuery.paddingOf(context).bottom` and adds a `SliverToBoxAdapter(child: SizedBox(height: bottomPadding))` at the end of your sliver list.

This isn’t accidental. [_GitHub issue #105027_](https://github.com/flutter/flutter/issues/105027) documented this interaction `CustomScrollView` not consuming the inflated padding that Scaffold publishes, but `CustomScrollView` itself deliberately doesn’t auto-consume that padding. It’s the low-level API when you use it, you’re opting into full manual control. Auto-padding would break that contract. You might have pinned headers, custom gaps, or slivers that shouldn’t be padded. `BoxScrollView` is the high-level convenience wrapper where opinionated defaults like auto-padding belong.

This is our most common scroll view. Most of our screens use `CustomScrollView` with sliver-based layouts for the flexibility it provides (pinned headers, sliver groups, mixed sliver types). So yeah, we had a lot of screens to touch.

`**SingleChildScrollView**` **— also manual:**

`SingleChildScrollView` isn’t even a `ScrollView` subclass. It builds its own `Viewport` directly, no `buildSlivers()` override, no automatic padding injection. If your content is inside a`SingleChildScrollView`, you need padding _inside_ the scroll view.

The architectural difference comes down to where the auto-padding lives. iOS puts it on `UIScrollView`, the base class, so every subclass inherits it. Flutter puts it on `BoxScrollView`, a specific high-level subclass. `CustomScrollView`, the lower-level API, deliberately skips it. It’s a reasonable trade-off, but it does mean more migration work on your end.

We use `CustomScrollView` on most screens through a custom scaffold wrappers (`LiquidGlassScaffold & FennelScaffold`). Since the wrappers build the `CustomScrollView` internally, we were able to add the spacer once in the wrappers definition, and every screen that uses it gets the bottom spacer for free. If you don’t have centralized wrappers like this, you’ll need to add a spacer to each `CustomScrollView` individually.

**Part 3: Building an App Bar That Doesn’t Box You In**

**Why We Built Our Own App Bar**

Flutter’s `SliverAppBar` has a constraint that didn’t quite work for what we needed: _the background and foreground must have the same height._ If your app bar is 56px tall, the background is 56px tall. If you increase `expandedHeight`, the content area shifts down by the same amount.

We wanted something different. A gradient that starts at the app bar and fades smoothly into the content area, maybe 90 pixels deep depending on the screen, with no hard edge. But with`SliverAppBar`, changing the background height means changing the foreground height, which pushes content down. That defeats the purpose. This is perfectly reasonable behavior for what `SliverAppBar` is designed to do, we just wanted something slightly different.

`SliverAppBar` treats background and foreground as a single unit. We needed them to be independent.

**The Independent Overlay Pattern**

Our solution was to separate the app bar into independent layers, stacked at the scaffold level:

```
Stack(
children: [scrollView, // Layer 1: CustomScrollView (content)
darkeningOverlay, // Layer 2: Gradient that darkens with scroll
appBarForeground, // Layer 3: Title, back button, trailing icons
],
);
```

The `CustomScrollView` still contains a `SliverAppBar`, but it’s essentially a transparent spacer. It reserves space for the foreground elements but renders nothing visible. You could do this without `SliverAppBar` entirely, but we preferred this approach. The actual visual elements (gradient, title, buttons) are `Positioned` widgets at the `Stack` level.

This gives us complete independence:

- The **gradient background** can extend well beyond the app bar height (120px, 200px, whatever the screen needs) and shrink as you scroll

- The **foreground** (title, buttons) stays at its own fixed height regardless of what the background does

- The **darkening overlay** animates independently based on scroll position

- Content scrolls naturally behind all of them

This directly helps with the spacious feel we were going for. With a standard `AppBar`, there’s a hard visual boundary between the bar and content. With this approach, the gradient fades smoothly into the content area. The app bar feels like part of the content rather than something sitting on top of it. None of this is new, but it was new to us, and adopting it made a noticeable difference.

![captionless image](https://miro.medium.com/v2/resize:fit:800/format:webp/1*qqpCMCAOIJpKBsd1WgzyHA.gif)

**Scroll-Driven Visuals**

Making this work required tracking the scroll position carefully. Different visual elements respond to scroll in different ways, so our scaffold tracks three separate values:

- **A clamped offset.** The darkening overlay and blur effect only need to respond to the first ~100 pixels of scroll. After that, they’re at full intensity. Clamping this prevents unnecessary rebuilds during long scrolls further down the page.

- **An unclamped offset** for the background height, which needs to keep shrinking as content scrolls up, even beyond what the overlay cares about.

- **Overscroll** for when users pull down past the top of the page. We stretch the background image for an iOS-style rubber band effect. Only relevant on platforms that support it (iOS/macOS).

These aren’t specific to Liquid Glass, they’re things we had to build ourselves once we stepped outside Flutter’s standard `SliverAppBar`. And that’s fair. `SliverAppBar` is a genuinely well-built widget that handles a lot of complexity behind the scenes: collapsing, pinning, floating, stretch, safe area, scroll coordination. You don’t appreciate how much it does for you until you try to do it yourself. But if you want something it wasn’t designed for, you take on all of that responsibility. Building our custom scaffold (`LiquidGlassScaffold`) gave us a much deeper understanding of what Flutter handles in the background. The details deserve their own post, so I won’t go deeper here.

**The Gradient That Fades Into Content**

When we started building this, I had a bunch of approaches in mind. Blur backgrounds, shaders, you name it. But in the end, a dead simple gradient worked surprisingly well for us. We do use blur separately for pinned headers, but for the fading app bar effect, gradients were more than enough.

`LiquidGlassDarkeningOverlay` is what creates the “no hard edge” effect. It renders a `LinearGradient` whose alpha values are multiplied by scroll progress:

```
LinearGradient(
begin: Alignment.topCenter,
end: Alignment.bottomCenter,
stops: [0.0, 0.3, 0.5, 0.65, 0.8],
colors: [for (final alpha in [1.0, 0.98, 0.64, 0.19, 0.0])
baseColor.withValues(alpha: alpha * scrollProgress),
],
);
```

When `scrollProgress` is 0 (top of page), every alpha is multiplied by 0 so the gradient is invisible. As you scroll, it fades in. The non-linear stops (0.0, 0.3, 0.5, 0.65, 0.8) create a natural-feeling falloff rather than a uniform fade.

At the top of a page, you see your gradient background flowing seamlessly into content. As you scroll, a subtle darkening appears to keep text legible over the scrolling content behind. There’s never a hard line where “app bar ends, and content begins.”

Each screen can configure its own background style through a simple config object.

**Blur on Pin: Syncing the App Bar with Pinned Headers**

Some of our screens have pinned section headers, column labels that stick to the top as you scroll through a list. On the collection detail screen, for example, there’s a pinned header showing “Equities” and “Share Value” labels. When the user scrolls and content starts passing behind that header, we activate a blur effect so the overlapping content doesn’t create visual noise.

Flutter’s `SliverPersistentHeaderDelegate` gives you an `overlapsContent` boolean in its `build` method. When content is scrolling behind the pinned header, Flutter sets this to `true`. We use that signal to activate blur on the header itself:

```
@override
Widget build(BuildContext context, double shrinkOffset, bool overlapsContent) {
onOverlapsContentChanged(overlapsContent);
return _ColumnLabelsContent(showBlur: overlapsContent);
}
```

When the pinned header blurs, the app bar area above it should blur too. Otherwise, you get a blurred header with a clear app bar right above it, and it looks disconnected. So the pinned header communicates its state upward through a `PinnedHeaderController`, a simple `ChangeNotifier`. The scaffold listens to it and activates its own blur to match.

The app bar and the pinned header each render their own `BackdropFilter` independently. Because these are two separate blur layers, there’s a subtle visible line where they meet, almost like a divider. We actually liked this, it gives the pinned header a distinct edge and makes the layout feel more structured, so we kept it. If you’d prefer a seamless look with no visible boundary, you could either render a single blur layer that covers both areas, or use Flutter’s `[_BackdropGroup_](https://api.flutter.dev/flutter/widgets/BackdropGroup-class.html)` with `[_BackdropFilter.grouped_](https://api.flutter.dev/flutter/widgets/BackdropFilter/BackdropFilter.grouped.html)` to merge multiple blur layers into one render pass.

![captionless image](https://miro.medium.com/v2/resize:fit:800/format:webp/1*WwN3sl7q0Umq5TlPZudsVA.gif)

**Part 4: The Bottom Nav Bar**

**Two Shapes, Not One**

The bottom bar isn’t a single monolithic pill. It’s two independent glass containers sitting in a `Row`: the **_tabs area_** on the left and the **_extra button_** on the right. Both are separate `LiquidGlassLayer` widgets with their own shape, border radius, and content.

```
Row(
children: [SizedBox(width: tabsWidth, child: MorphingTabsArea(…)),
SizedBox(width: 8), // spacing
SizedBox(width: extraButtonWidth, child: MorphingExtraButton(…)),
],
);
```

The total width is always constant (the full bar width minus horizontal padding). What changes is how that width is distributed between the two containers. In the default state, the tabs area takes most of the space and the extra button is a small circle. When the extra button activates, the distribution shifts, the tabs shrink, the button grows. Both widths are driven by the same spring animation value, so they move in sync. More on that in Part 5.

**### Scaffold Integration**

This ties directly back to Part 2. The bar goes into the `bottomNavigationBar` slot with `extendBody: true`:

```
FennelScaffold(
extendBody: true,
resizeToAvoidBottomInset: false,
bottomNavigationBar: liquidGlassBottomBar,
body: shellContent,
)
```

Scaffold handles the `MediaQuery.padding.bottom` inflation automatically. Every `ListView` and `GridView` inside the shell gets the correct bottom spacing for free. Our `CustomScrollView` screens get it through the spacer we added in `LiquidGlassScaffold` (Part 2). One integration point, no manual padding math.

**Part 5: The Extra Button — Merging Actions Into the Nav Bar**

This is the piece that let us remove most of our persistent footers. The idea of an expandable extra button next to the tab bar turned out to be way more versatile than we initially expected.

**Three Modes, One Button**

Our extra button supports three modes:

1. **Button mode** (default): a simple icon button. Shown on most screens.

2. **Search bar mode:** expands into a search bar visual. Shown on our Explore & Search screen.

3. **Buy/Sell mode:** expands into segmented Buy and Sell buttons. Shown on the company detail.

The mode is determined by the current route. The tab bar screen watches navigation changes and decides which mode fits the current context. Navigate to Explore, it switches to search bar mode. Open a company detail, buy/sell. Everything else gets the default button.

When the mode changes, the button morphs, the tabs collapse, and the whole thing animates together.

**The Morphing Animation**

We borrowed a principle from iOS 26 for the animation: **_the glass layer never fades, it only morphs shape_.** Fading a glass/blur layer creates ugly artifacts. The blur intensity changes, edges become visible, it just looks broken. Instead, we keep the glass at full opacity and only animate its width, height, and border radius.

Content _inside_ the glass fades, using a quick crossfade with a deliberate gap:

```
// Quick crossfade with gap timing:
// Button content fades out: 0.0 → 0.4
// Gap (both invisible): 0.4 → 0.6
// Expanded content fades in: 0.6 → 1.0
final buttonOpacity = Curves.easeInOut.transform(
((0.4 - clampedAnimValue) / 0.4),
);
final expandedOpacity = Curves.easeInOut.transform(
((clampedAnimValue - 0.6) / 0.4)
);
```

The 0.4–0.6 gap is subtle but important. Without it, both contents are partially visible at the same time, and you get a muddy crossfade. The gap makes it cleaner: old content disappears, a brief beat, and new content appears.

**Spring Physics**

We use the `[motor](https://pub.dev/packages/motor)` package with `CupertinoMotion` for iOS style spring animations:

```
CupertinoMotion(duration: Duration(milliseconds: 350), bounce: 0.25)
```

If you’re interested in the details of how the spring parameterization works, the `[motor](https://pub.dev/packages/motor)`package is worth a look.

**How the Navbar Shrinks**

When the extra button expands, the tabs area needs to make room. It’s not a simple show/hide. It’s a coordinated width animation where both elements morph at the same time:

![captionless image](https://miro.medium.com/v2/resize:fit:592/format:webp/1*OOhj9ZtN9D--LD1suEncLw.gif)

The layout is a `Row` where both the tab area and the extra button have explicitly animated widths. The total width stays constant (the full bar width). What changes is how that width is _distributed_ between the two.

When the extra button activates, the tabs area interpolates from its full width down to a small circle, just enough to show the last active tab icon so users have a way back. The extra button takes whatever space the tabs released. Both widths are driven by the same spring animation value, so they move in perfect sync. There’s never a gap or overlap during the transition.

The extra button also morphs its border radius during this animation: from a perfect circle (when collapsed) to a pill shape (when expanded). Combined with the width change, it creates the effect of the button “growing” into its expanded form rather than just resizing.

**Buy/Sell: The Persistent Footer Killer**

The most impactful use case was company detail. Before, we had Buy and Sell buttons as persistent footer widgets, a full-width bar sitting above the bottom nav. Now they live inside the extra button.

When a user navigates to a company detail screen, the tab bar detects the route, switches the extra button to `buySell` mode, and the button expands to show Buy and Sell controls. The buttons have full state management: enabled/disabled based on asset restrictions, shimmer loading states, warning indicators for trading restrictions, visibility toggling (Sell hides when the user doesn’t own shares, and Buy expands to full width).

When the user navigates away, the button collapses back to a circle. The persistent footer is gone. The company detail screen has its full height for charts, financials, and analysis. It’s a genuinely noticeable difference.

**What About Actions That Don’t Fit?**

Not everything could merge into the extra button. For actions that are screen-specific but not tied to the navigation paradigm (like a “Create” button or a “Place Order” confirmation), we converted persistent footers to floating action buttons.

These float above the nav bar using `FennelScaffold`’s built-in FAB positioning. The scaffold reads `MediaQuery.of(context).padding.bottom` to position the FAB above the bar, and since Scaffold inflates this value to include the bar height, the FAB automatically positions itself correctly. When the keyboard opens, the FAB shifts up smoothly. When it closes, it settles back down.

**No Official Flutter Support Yet**

The Flutter team has a [plan for Liquid Glass support](https://github.com/flutter/flutter/issues/170310), but they’re rearchitecting their design system libraries first and building these features in new standalone packages. That takes time, and there’s no official solution available today. In the meantime, the community has stepped up with public packages that make it possible to ship this now.

Whatever library you use, my advice: build common components and use them everywhere. Don’t scatter glass layers across individual screens. We wrapped ours into reusable pieces, a circular glass button, an expanded pill container, a morphing tabs area, and every screen uses those abstractions rather than working with the glass renderer directly. If Flutter ships official support tomorrow, we’d swap the rendering layer inside those components and everything else stays the same. That’s the migration path we’re counting on, and it’s only possible because we kept the glass implementation behind a clean boundary.

_We’re Fennel, a small fintech startup building with Flutter. We’re still learning and trying to keep up with where things are heading. If you’re working on something similar, we’d love to hear about your experience._
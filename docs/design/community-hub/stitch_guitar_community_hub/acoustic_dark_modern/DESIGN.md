---
name: Acoustic Dark Modern
colors:
  surface: '#12131a'
  surface-dim: '#12131a'
  surface-bright: '#383940'
  surface-container-lowest: '#0c0e14'
  surface-container-low: '#1a1b22'
  surface-container: '#1e1f26'
  surface-container-high: '#282a31'
  surface-container-highest: '#33343c'
  on-surface: '#e2e1eb'
  on-surface-variant: '#d5c4ab'
  inverse-surface: '#e2e1eb'
  inverse-on-surface: '#2f3037'
  outline: '#9e8f78'
  outline-variant: '#514532'
  surface-tint: '#ffba20'
  primary: '#ffdca1'
  on-primary: '#412d00'
  primary-container: '#ffb800'
  on-primary-container: '#6b4c00'
  inverse-primary: '#7c5800'
  secondary: '#dec1af'
  on-secondary: '#3f2c20'
  secondary-container: '#574335'
  on-secondary-container: '#ccb09f'
  tertiary: '#e3e0df'
  on-tertiary: '#313030'
  tertiary-container: '#c7c4c4'
  on-tertiary-container: '#525151'
  error: '#ffb4ab'
  on-error: '#690005'
  error-container: '#93000a'
  on-error-container: '#ffdad6'
  primary-fixed: '#ffdea8'
  primary-fixed-dim: '#ffba20'
  on-primary-fixed: '#271900'
  on-primary-fixed-variant: '#5e4200'
  secondary-fixed: '#fbddca'
  secondary-fixed-dim: '#dec1af'
  on-secondary-fixed: '#28180d'
  on-secondary-fixed-variant: '#574335'
  tertiary-fixed: '#e5e2e1'
  tertiary-fixed-dim: '#c9c6c5'
  on-tertiary-fixed: '#1c1b1b'
  on-tertiary-fixed-variant: '#474646'
  background: '#12131a'
  on-background: '#e2e1eb'
  surface-variant: '#33343c'
typography:
  display-lg:
    fontFamily: Hanken Grotesk
    fontSize: 48px
    fontWeight: '800'
    lineHeight: 56px
    letterSpacing: -0.02em
  headline-lg:
    fontFamily: Hanken Grotesk
    fontSize: 32px
    fontWeight: '700'
    lineHeight: 40px
  headline-lg-mobile:
    fontFamily: Hanken Grotesk
    fontSize: 28px
    fontWeight: '700'
    lineHeight: 36px
  headline-md:
    fontFamily: Hanken Grotesk
    fontSize: 24px
    fontWeight: '600'
    lineHeight: 32px
  body-lg:
    fontFamily: Hanken Grotesk
    fontSize: 18px
    fontWeight: '400'
    lineHeight: 28px
  body-md:
    fontFamily: Hanken Grotesk
    fontSize: 16px
    fontWeight: '400'
    lineHeight: 24px
  label-md:
    fontFamily: JetBrains Mono
    fontSize: 14px
    fontWeight: '500'
    lineHeight: 20px
    letterSpacing: 0.05em
  label-sm:
    fontFamily: JetBrains Mono
    fontSize: 12px
    fontWeight: '500'
    lineHeight: 16px
    letterSpacing: 0.05em
rounded:
  sm: 0.25rem
  DEFAULT: 0.5rem
  md: 0.75rem
  lg: 1rem
  xl: 1.5rem
  full: 9999px
spacing:
  base: 4px
  xs: 4px
  sm: 8px
  md: 16px
  lg: 24px
  xl: 40px
  container-margin-mobile: 16px
  container-margin-desktop: 32px
  gutter: 16px
---

## Brand & Style
The design system is built to evoke the feeling of a high-end recording studio or a vintage instrument gallery. It targets guitarists who are passionate about their craft, blending a professional, technical edge with the warmth of an organic community.

The design style is **Modern Dark with Tonal Layering**. It avoids pure blacks in favor of deep charcoal and "espresso" neutrals, creating a sophisticated backdrop for vibrant amber accents. The aesthetic is "Sleek-Organic": it uses precision-engineered typography and layout alongside subtle gradients and textures that mimic the high-gloss finish of a guitar body. It is professional enough for educational content but expressive enough for a creative social hub.

## Colors
The palette is rooted in the "Golden Hour" of a performance. 

- **Primary (Amber):** Used for key actions, progress bars, and focus states. It mimics the glow of a vacuum tube amplifier.
- **Secondary (Burnt Sienna):** A deep wood-inspired tone used for subtle backgrounds, secondary buttons, and decorative accents.
- **Surface Palette:** We use a tiered dark system. The base background is nearly black (`#0D0D0D`), while elevated surfaces use progressively lighter shades of charcoal to create depth without relying on heavy shadows.
- **Semantic Colors:** Success is a moss green, Warning is a soft orange, and Error is a vivid crimson, all adjusted for high legibility against dark backgrounds.

## Typography
The typography strategy balances modern precision with technical utility. 

**Hanken Grotesk** serves as the primary typeface. Its sharp terminals and contemporary proportions provide a clean, high-tech feel for a platform that involves digital tabs and gear specs. 

**JetBrains Mono** is used for labels, metadata (like BPM or tuning), and technical data. This monospaced font reinforces the "hub" aspect—suggesting a workspace where guitarists analyze and learn.

Use heavy weights (700+) for headlines to create strong visual anchors. Body text should maintain a generous line height (1.5x) to ensure readability during long practice sessions or while reading tabs on a mobile device.

## Layout & Spacing
This design system utilizes a **Fluid Grid** system based on an 8px rhythm, scaled down to 4px for fine-grained UI details. 

- **Mobile (Base):** A 4-column layout with 16px side margins. Elements are primarily stacked to prioritize vertical scrolling of feeds and lessons.
- **Tablet:** An 8-column layout with 24px margins. This allows for side-by-side placement of video content and notation/tabs.
- **Desktop:** A 12-column layout with a maximum content width of 1280px. 

Spacing is used to group related "gear." For example, a video lesson and its description should have `sm` (8px) spacing, while the transition to the next lesson in a list should use `lg` (24px) to provide a clear break.

## Elevation & Depth
Depth is created through **Tonal Layering** and **Subtle Inner Glows**, rather than traditional drop shadows.

1.  **Level 0 (Floor):** `#0D0D0D` - The canvas.
2.  **Level 1 (Cards/Items):** `#1A1A1A` - Standard cards for lessons or forum posts.
3.  **Level 2 (Modals/Overlays):** `#262626` - High-priority containers.

To enhance the premium feel, use a 1px "inner stroke" or "top-edge highlight" on elevated cards using a low-opacity version of the primary color or a neutral grey (`rgba(255, 255, 255, 0.05)`). This mimics the way light catches the edge of a guitar fret or a control knob. Glassmorphism is reserved for navigation bars to maintain content context while scrolling.

## Shapes
The shape language is **Rounded**, reflecting the ergonomic curves of a guitar body. 

- **Standard Elements:** 0.5rem (8px) radius for buttons and input fields.
- **Cards & Large Containers:** 1rem (16px) radius for lesson thumbnails and community posts.
- **Media/Video:** 1.5rem (24px) radius for featured video hero sections.

This consistent rounding softens the dark, technical aesthetic, making the community feel more approachable and less like a rigid software tool.

## Components
- **Buttons:** Primary buttons are solid Amber with black text. Secondary buttons are outlined with a Burnt Sienna border. Ghost buttons use JetBrains Mono for a technical, low-emphasis look.
- **Cards:** Use "Level 1" surface color. Media (images/video) should always be at the top of the card with no margin to the edges. Content inside has `md` (16px) padding.
- **Input Fields:** Darker than the card background with a subtle Amber bottom border on focus. Labels use `label-sm` in JetBrains Mono.
- **Chips/Tags:** Used for "Genres" or "Difficulty Levels." These use the Secondary color (`#3D2B1F`) with Amber text for a "wood and wire" aesthetic.
- **Tablature Display:** A custom component. Use a dark grey background with high-contrast white lines. The active note or "playhead" should be highlighted in Primary Amber.
- **Progress Bars:** Thin, high-contrast Amber lines used for course completion and audio playback.
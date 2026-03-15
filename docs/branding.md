# Branding

| Attribute | Value |
| --- | --- |
| **Product** | DISTIRA |
| **Tagline** | The AI Context Compiler |
| **Sub-tagline** | Compile the smallest useful context before every LLM call |
| **Brand Core** | Compression frame / vice symbol in premium dark blue |

## Colors

| Token | Hex | Usage |
| --- | --- | --- |
| `--bg-900` | `#050914` | Deep backgrounds |
| `--bg-800` | `#0d1422` | Panels and logo cards |
| `--primary` | `#39d3ff` | Primary K gradient stop |
| `--secondary` | `#9f8bff` | Secondary K gradient stop |
| `--text` | `#eaf0ff` | Main wordmark text |
| `--muted` | `#93a1b2` | Secondary copy |

## Core assets

- `brand/distira_symbol.svg` - symbol only, transparent background
- `brand/distira_symbol_mono.svg` - monochrome symbol variant
- `brand/distira_symbol_1024.png` - raster symbol (1024px)
- `brand/distira_symbol_mono_1024.png` - raster monochrome symbol
- `brand/distira_logo_horizontal_dark.svg` - horizontal logo, dark background
- `brand/distira_logo_horizontal_light.svg` - horizontal logo, light background
- `brand/distira_app_icon.svg` - app icon
- `brand/distira_app_icon_1024.png` - raster app icon (1024px)
- `brand/favicon.ico` + `brand/favicon-*.png` - favicon set

## Channel-ready assets

- `brand/distira_banner_1600x900.svg` — primary product banner (1600x900)
- `brand/distira_banner_1600x900.png` — raster banner for platforms requiring PNG
- `brand/distira_banner_1600x900.jpg` — JPG export for platforms requiring JPEG
- `brand/distira_social_banner_1500x500.svg` — social/Twitter cover (1500x500)
- `brand/distira_social_banner_1500x500.png` — raster social banner PNG
- `brand/distira_social_banner_1500x500.jpg` — raster social banner JPG
- `brand/distira_avatar_512.png` — GitHub avatar / profile picture (512px)
- `brand/distira_brand_preview.jpg` — brand overview preview
- `brand/distira_concept_board_frame.png` — compression frame concept board
- `brand/distira_concept_board_vise.png` — compression vice concept board

## Usage rules

- Keep a clear space around the logo equal to at least `0.5x` the monogram width.
- Minimum monogram size: `24px`; minimum full logo height: `40px`.
- Do not replace the custom K with a font letter.
- Keep the K gradient direction top-left to bottom-right.
- On light backgrounds, place logo files on a dark panel or export a dark-text variant first.
- For GitHub README and Markdown previews, prefer `brand/distira_mark.png` to avoid JPG compression artifacts.

## Dashboard alignment

Dashboard favicon and mark are synced with the same K geometry:

- `dashboard/ui-vue/public/favicon.svg`
- `dashboard/ui-vue/src/assets/distira-mark.svg`

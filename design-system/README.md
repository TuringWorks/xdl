# XDL Design System

XDL renders the same data through **four independent backends**:

| Surface | Crate | Output |
|---|---|---|
| plotters | `xdl-stdlib/src/graphics/` | PNG |
| ECharts | `xdl-charts/` | HTML |
| Three.js / WebGPU | `xdl-viz3d-web`, `xdl-viz3d-threejs` | HTML |
| egui | `xdl-gui/` | native window |

They must not look like four different products, and — more important — **they
must not encode the same data with different colours.** In a scientific plotting
tool, a colour is a value. Two backends disagreeing about a colormap is not a
styling inconsistency; it is two different answers to the same question.

This directory is the shared contract. It is not optional: a reviewer will reject
a change that introduces a hard-coded colour, a fourth colormap registry, or a
figure with no axis labels.

| File | Read when |
|---|---|
| [foundations/color.md](./foundations/color.md) | **Always** — UI colour, data colormaps, and the registry problem |
| [foundations/layout.md](./foundations/layout.md) | Type scale, spacing, figure proportions |
| [components/figure.md](./components/figure.md) | Building any HTML chart or viewer page |
| [tokens.css](./tokens.css) | Looking up a CSS custom property |

---

## The ten rules

1. **Never write a hex colour** in a template, a chart option, or a shader. UI
   colour comes from `tokens.css`; data colour comes from a colormap in the
   registry. There are currently **34 hard-coded hex literals** across
   `xdl-charts/src/echarts.rs` (23), `xdl-chart-viewer/src/main.rs` (6),
   `xdl-viz3d-web/src/template.rs` (4), and `xdl-viz3d-threejs/src/templates.rs` (1).
   Replace one when you touch its file.
2. **One colormap registry.** Do not add a fifth. See
   [foundations/color.md § The three registries](./foundations/color.md#the-three-registries-and-why-that-is-a-bug).
3. **A colormap request that cannot be satisfied is an error, not a substitution.**
   `ColorTable::load_table` currently returns grayscale for every unknown table
   number, so `LOADCT, 5` silently produces a monochrome figure the user believes
   is table 5. Absent stays absent — see
   [AGENTS.md → Numerical Honesty](../AGENTS.md#numerical-honesty--a-result-that-cannot-be-wrong-is-not-a-result).
4. **Sequential data gets a perceptually uniform colormap by default.** Rainbow
   and jet invent structure that is not in the data. They stay available for IDL
   compatibility; they are not the default for anything new.
5. **Every figure carries axes, tick labels, and units.** A plot without units is
   an unlabelled measurement. If the data has no units, say `(dimensionless)`.
6. **Every HTML surface defines its tokens on `:root` and honours
   `prefers-color-scheme`.** A figure unreadable in the user's theme is broken
   output, not a preference.
7. **Spacing is multiples of 4px**, from `--space-1 … --space-8`. No
   `padding: 13px 17px`.
8. **Interactive elements are `<button>` / `<a>`,** never `<div onclick>`. Icons
   are inline SVG with `stroke="currentColor"` — never emoji or `▶ ▼ ×`.
9. **Wide content scrolls inside its own container.** The page body never scrolls
   horizontally.
10. **Render it before you call it done.** Open the PNG or the HTML, check both
    themes, and use a **non-square** input so a transposed axis shows up.

---

## Quick reference

### UI colour

| Need | Token |
|---|---|
| Page background | `var(--bg-page)` |
| Figure/plot background | `var(--bg-figure)` |
| Panel / card surface | `var(--bg-surface)` |
| Primary text | `var(--text-primary)` |
| Axis labels, secondary text | `var(--text-secondary)` |
| Muted / disabled | `var(--text-muted)` |
| Axis lines, ticks | `var(--axis-line)` |
| Grid lines | `var(--grid-line)` |
| Border | `var(--border)` |
| Accent / selection | `var(--accent)` |
| Error | `var(--error)` |
| Warning | `var(--warning)` |
| Success | `var(--success)` |

### Data colour

**Never a UI token.** Data colour comes from the colormap registry:

| Data shape | Use |
|---|---|
| Sequential (0 → max) | `viridis` (default), `plasma`, `inferno` |
| Diverging (−x → 0 → +x) | `blue_red`, centred on zero |
| Cyclic (phase, angle) | a cyclic map — never a sequential one |
| Categorical (≤ 8 series) | the categorical series list, in order |
| Categorical (> 8 series) | you have too many series; aggregate or facet |
| IDL compatibility | `LOADCT, n` — the numbered tables, exactly as IDL defines them |

### Figure proportions

| Kind | Aspect | Notes |
|---|---|---|
| Line / scatter | 4:3 | the default |
| Time series | 16:9 | wide, so the x-axis reads |
| Image / map | **1:1 per data pixel** | never stretch to fill; a stretched image is a wrong image |
| Surface / 3D | 4:3 | with a stated viewing angle |

---

## How each backend consumes this

| Backend | UI colour | Data colour | Text |
|---|---|---|---|
| plotters (PNG) | the Rust constants mirroring `tokens.css` | `graphics::state::ColorTable` | plotters font config |
| ECharts (HTML) | `tokens.css` via `var(--…)` in the option blob | the registry, serialised into `visualMap` | inherits the page |
| Three.js (HTML) | `tokens.css` | `colormaps::generate_colormap` | inherits the page |
| egui (native) | the egui theme built from the same constants | `graphics::state::ColorTable` | egui font config |

A PNG has no CSS, so plotters and egui read Rust constants. **Those constants and
`tokens.css` are two copies of one palette** — when you change one, change the
other, or the same script produces two different-looking figures depending on how
it was rendered. That is the same duplicated-contract trap as the built-in name
lists: see [AGENTS.md → Duplicated contracts drift silently](../AGENTS.md#duplicated-contracts-drift-silently).

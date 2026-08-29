# Colour

Two colour systems live in XDL and they must never be confused.

| | **UI colour** | **Data colour** |
|---|---|---|
| Encodes | interface structure | a measurement |
| Source | `tokens.css` (`--text-*`, `--axis-line`, `--accent`) | the colormap registry |
| Changes with | the user's theme | never — it is the data |
| Wrong value means | it looks off | **the figure is wrong** |

**A data colour that changes with the theme is a bug.** A user reading a value off
a colourbar must get the same value in light and dark mode. UI furniture — axis
lines, tick labels, background, legend text — adapts to the theme. The mapped
data does not.

---

## The three registries, and why that is a bug

XDL currently defines colormaps in three unrelated places, with three different
naming schemes and three different defaults:

| Where | Names | Unknown request → |
|---|---|---|
| `xdl-stdlib/src/graphics/state.rs` `ColorTable::load_table` | numbers: 0 grayscale, 1 blue_red, 2 blue_white, 3 grn_red_blu_wht, 13 rainbow | **grayscale, silently** |
| `xdl-viz3d-threejs/src/colormaps.rs` `generate_colormap` | strings: `VIRIDIS`, `RAINBOW`, `PLASMA`, `INFERNO`, `TURBO`, `GRAYSCALE` | **viridis, silently** |
| `xdl-charts/src/echarts.rs` | none — an 11-stop diverging ramp hard-coded inline | n/a |

Consequences, all of them live today:

- **`LOADCT, 5` produces grayscale.** IDL defines 41 numbered tables; XDL
  implements five. The user gets a monochrome figure and no indication that the
  table they asked for does not exist. This is the
  [fallback-default data-integrity bug](../../AGENTS.md#a-fallback-default-is-a-data-integrity-bug)
  in its purest form: the output is plausible and wrong.
- **The same script produces different colours on different backends.** `RAINBOW`
  in Three.js and table 13 in plotters are separately hand-written ramps; nothing
  makes them agree.
- **A surface plotted to PNG and to HTML are not comparable figures**, which is
  precisely the thing a plotting library exists to make possible.

### The rule

**One registry, keyed both ways.** A colormap has a canonical name *and*, where
IDL defines one, a table number. Every backend resolves through the same
function; the backends differ only in how they *emit* the resulting ramp (RGB
triples for plotters, `visualMap` stops for ECharts, a texture for Three.js).

Until that consolidation lands:

- **Do not add a fourth registry.** A new colormap goes into the existing ones
  and is added to *all* of them in the same commit.
- **Do not add another silent fallback.** An unknown table number or name is an
  `XdlError` naming what was asked for and listing what exists. Fixing
  `load_table`'s `_ => Self::grayscale()` is a small, high-value change; take it
  when you are next in that file.
- **Do not hard-code a ramp inline.** The 11 stops in `echarts.rs` are a
  RdYlBu diverging map. It belongs in the registry under that name.

---

## Choosing a colormap

The colormap encodes the *shape* of the data. Picking the wrong family is a
correctness error, not a taste error.

| Data | Family | Default | Why |
|---|---|---|---|
| Sequential — 0 to max, one direction | perceptually uniform sequential | `viridis` | equal steps in data look like equal steps in colour |
| Diverging — signed, meaningful zero | diverging, **centred on zero** | `blue_red` | the neutral colour must land on 0, not on the midpoint of the data range |
| Cyclic — phase, angle, time of day | cyclic | — | a sequential map puts a false discontinuity at the wrap |
| Categorical — unordered classes | the eight `--series-*` tokens | — | a continuous ramp implies an order the data does not have |
| Binary mask / classification | two high-contrast colours | — | never a ramp |

### Rainbow and jet

They remain available — `LOADCT, 13` is real IDL and XDL is a compatibility
project. **They are not the default for anything new**, and they should not be
chosen for a new figure in the docs. Rainbow is not perceptually uniform: it
compresses large parts of the data range into visually similar greens and
introduces a sharp apparent edge at the cyan and yellow bands that is an artefact
of the palette, not a feature of the data. In greyscale — printing, photocopying,
a monochrome screenshot — it is not monotonic, so the figure becomes unreadable
rather than merely colourless.

### Diverging maps must be centred, not auto-scaled

A diverging map on a range of `[-1, +9]` with its neutral colour placed at the
range midpoint puts "white" at 4, and every reader will interpret 4 as the
neutral value. **Centre the map on the meaningful zero and let the two arms be
unequal**, or state in the caption that the scale is asymmetric.

### Colour-blind safety

~8% of men have a red–green colour vision deficiency.

- The `--series-*` order is chosen so the **first four** stay distinguishable
  under deuteranopia and protanopia. If you need five or more lines, add a second
  channel: dash pattern, marker shape, or direct labelling.
- **Never encode meaning in hue alone** for a binary distinction — pass/fail,
  above/below. Pair it with a shape, a position, or a label.
- `viridis`, `plasma`, and `inferno` are monotonic in luminance, so they survive
  both colour-blindness and greyscale. That is the reason they are the default,
  not fashion.

---

## Using UI colour

```css
/* right */
.axis-label { fill: var(--text-secondary); font-size: var(--text-xs); }
.grid line  { stroke: var(--grid-line); }

/* wrong — invisible in dark mode, and unfindable when the palette changes */
.axis-label { fill: #4a5560; }
```

For an ECharts option blob, read the token once and interpolate:

```rust
// xdl-charts: resolve tokens at template time, not by hard-coding the value.
let axis = "var(--axis-line)";
```

If a surface genuinely cannot resolve a CSS variable (a shader uniform, a PNG),
read the value from the shared Rust palette constants that mirror `tokens.css` —
**do not re-type the hex.**

---

## Contrast

| Element | Minimum |
|---|---|
| Body text, axis labels | 4.5:1 against its background |
| Tick labels, footnotes | 4.5:1 — small text has no exemption |
| Axis lines, borders, focus rings | 3:1 |
| A data colour against the figure background | 3:1, so the mark is visible at all |

The tokens meet these in both themes. A colour you introduce yourself is your
responsibility to check.

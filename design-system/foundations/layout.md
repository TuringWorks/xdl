# Layout, type, and figure proportions

## Type scale

Five sizes. If you need a sixth, you are building a different page.

| Token | Size | Use |
|---|---|---|
| `--text-xs` | 12px | tick labels, footnotes, units in parentheses |
| `--text-sm` | 14px | axis labels, legends, table cells, body in dense views |
| `--text-base` | 16px | body text |
| `--text-lg` | 18px | figure title |
| `--text-xl` | 22px | page title |

Weights: `--weight-regular` (400) for everything, `--weight-medium` (500) for
labels that need to separate from their value, `--weight-bold` (600) for titles.
There is no 700 — a heavier weight is not more important, it is louder.

Numbers in tables, coordinates, and any column the reader compares down use
`--font-mono` with `font-variant-numeric: tabular-nums`, so digits align.

## Spacing — the 4px grid

`--space-1` (4px) through `--space-8` (48px). Every margin, padding, and gap is
one of these. `padding: 13px 17px` is not a considered decision, it is a
measurement someone took off a screenshot.

| Relationship | Space |
|---|---|
| Inside a control (button padding) | `--space-2` / `--space-3` |
| Between related controls | `--space-2` |
| Between a label and its field | `--space-1` |
| Between form rows | `--space-4` |
| Between sections | `--space-6` |
| Page margin | `--space-6` (mobile) → `--space-8` (desktop) |

Proximity carries meaning: things closer together are read as related. A legend
`--space-6` from its chart belongs to neither.

## Figure proportions

| Kind | Aspect | Rule |
|---|---|---|
| Line / scatter | 4:3 | the default |
| Time series | 16:9 | wide, so the x-axis is legible |
| Image, map, `TV`, `TVSCL` | **square data pixels** | never stretch to fill the container |
| Surface / 3D | 4:3 | state the viewing angle in the caption |
| Small multiples | identical axes on every panel | different scales make the grid meaningless |

**A stretched image is a wrong image.** `TVSCL` on a 512×128 array rendered into
a 600×600 box has changed the aspect ratio of the data. Letterbox it. This is the
same class of error as a wrong colormap: the picture is plausible and misreports
the measurement.

## Axes

Every figure carries all four:

1. **Both axes labelled**, with units in parentheses: `Wavelength (nm)`. If the
   quantity is genuinely dimensionless, write `(dimensionless)` — a missing unit
   and a dimensionless quantity look identical, and only one of them is finished.
2. **Tick labels the reader can parse.** Prefer 4–7 ticks. Use SI prefixes or a
   stated exponent (`×10⁶` on the axis) rather than six-digit tick labels.
3. **A stated range.** If the axis does not start at zero and the reader might
   assume it does — any bar chart, any magnitude comparison — say so, or start at
   zero.
4. **A colourbar wherever colour encodes a value**, with its own label and units.
   A colour-mapped figure without a colourbar is unreadable by design.

Grid lines are `--grid-line` and sit *behind* the data. If a grid line is
competing with a data line for attention, the grid is too strong.

## Legends

- Fewer than three series: **label the lines directly** and drop the legend. A
  legend is an indirection the reader has to resolve on every glance.
- Three to eight: a legend, ordered to match the data at the right edge of the
  plot, not alphabetically.
- More than eight: facet into small multiples. A nine-line chart is not a chart.

## Responsive

Relative units, flexbox or grid, `max-width: 100%` on images and canvases. Wide
content — tables, a wide time series, a code block — scrolls inside its own
`overflow-x: auto` container. **The page body never scrolls horizontally.**

A chart below ~480px wide cannot carry a legend, a title, and readable ticks at
once. Drop the legend first (direct-label instead), then the title, and keep the
ticks.

## Motion

`--transition` (120ms) for hover and focus. Nothing animates on load, and nothing
animates a data value — a bar that grows into place makes the reader wait to read
a number. Respect `prefers-reduced-motion: reduce` by disabling transitions
entirely.

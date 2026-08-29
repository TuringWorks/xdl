# Figure — the HTML chart skeleton

Every HTML surface — an `xdl-charts` ECharts page, an `xdl-viz3d-web` viewer, an
`xdl-chart-viewer` window — uses the same skeleton. It exists so that a title, a
caption, and a colourbar land in the same place on every page XDL emits, and so
the same CSS covers all of them.

```html
<figure class="xdl-figure">
  <figcaption class="xdl-figure__title">Rayleigh–Taylor instability, t = 0.4 s</figcaption>

  <div class="xdl-figure__canvas" role="img"
       aria-label="Density field, 512 by 128, viridis, 0 to 2.4 kg/m³">
    <!-- the chart: a <canvas>, an ECharts container, or an <svg> -->
  </div>

  <div class="xdl-figure__colorbar">
    <!-- present whenever colour encodes a value -->
  </div>

  <p class="xdl-figure__caption">
    Density (kg/m³). Grid 512 × 128, dx = 1 mm. Colormap: viridis, linear, 0–2.4.
  </p>
</figure>
```

```css
.xdl-figure {
  margin: 0 0 var(--space-6);
  padding: var(--space-4);
  background: var(--bg-figure);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
}
.xdl-figure__title {
  font-size: var(--text-lg);
  font-weight: var(--weight-bold);
  line-height: var(--leading-tight);
  margin-bottom: var(--space-3);
}
.xdl-figure__canvas {
  /* Never a fixed height. Set an aspect-ratio so the data keeps its shape. */
  aspect-ratio: 4 / 3;
  width: 100%;
  max-width: 100%;
}
.xdl-figure__colorbar { margin-top: var(--space-3); }
.xdl-figure__caption {
  margin: var(--space-3) 0 0;
  font-size: var(--text-sm);
  color: var(--text-secondary);
}
```

## Rules

1. **`<figure>` and `<figcaption>`, not `<div>`.** A screen reader announces a
   figure; it does not announce a div.
2. **The canvas has an `aria-label` describing what is plotted** — quantity,
   dimensions, colormap, range. A `<canvas>` is opaque to assistive technology,
   so this label is the only description that exists. Generate it from the same
   values that drive the plot, so it cannot drift.
3. **`aspect-ratio`, never a fixed pixel height.** A fixed height plus a fluid
   width silently restretches the data on every window size.
4. **The caption states what a reader needs to reproduce the figure**: the
   quantity and its units, the grid or sample count, the colormap and its range,
   and any transform (log, normalised, smoothed). "Density" is a label; the
   caption above is a figure.
5. **A colourbar whenever colour encodes a value.** No exceptions — a
   colour-mapped image without one cannot be read.
6. **Loading, empty, and error are real states**, not a blank canvas:

```html
<div class="xdl-figure__canvas xdl-state">Rendering…</div>
<div class="xdl-figure__canvas xdl-state">No data in the selected range.</div>
<div class="xdl-figure__canvas xdl-state xdl-state--error">
  CONTOUR failed: array must be 2-D, got rank 3.
</div>
```

An empty figure and a broken figure look identical to a user, and only one of
them is their fault. Say which it is, and put the interpreter's own error text in
the error state — the user is going to grep for it.

## Toolbars

Controls sit above the figure, never on top of it.

- `<button>` elements, `--space-2` gap, `--text-sm`.
- Icon-only buttons carry an `aria-label`.
- Icons are inline SVG with `stroke="currentColor"` and `stroke-width="1.5"`.
  **No emoji, no `▶ ▼ ◀ ×`** — they render differently on every platform and
  cannot be themed.
- A toggle uses `aria-pressed`. A tab strip uses `role="tablist"` / `role="tab"` /
  `aria-selected`.

## Before you call it done

- [ ] Rendered it and looked at it
- [ ] Checked light **and** dark
- [ ] Used a **non-square** input, so a transposed axis is visible
- [ ] Axis labels carry units; the colourbar carries units
- [ ] Resized the window — the data kept its aspect ratio
- [ ] The page body does not scroll horizontally

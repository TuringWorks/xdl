# XDL Graphics Quick Start 🚀

## ✅ Ready to Use NOW!

Your graphics system is **fully functional** with GUI support!

---

## 🎯 Run Your First Plot (30 seconds)

### Step 1: Launch the GUI
```bash
cd /Users/ravindraboddipalli/sources/gdl/xdl
./target/release/xdl-gui
```

### Step 2: Load Demo Script
In the GUI menu:
- **File** → **Open**
- Navigate to: `examples/plot_demo.xdl`
- Click **Open**

### Step 3: Execute
Click the **Execute** button

### Step 4: Watch Magic! ✨
An interactive plot window will appear with a sine wave!

---

## 🎨 What You Get

✅ **44 Graphics Procedures** - All working
✅ **Interactive FLTK Windows** - Resizable, beautiful
✅ **Color Tables** - 5 predefined (grayscale, rainbow, etc.)
✅ **PNG Fallback** - Works in CLI mode too
✅ **Multiple Windows** - Create as many plots as you need

---

## 📝 Simple Examples

### Example 1: Basic Plot
```xdl
x = FINDGEN(100) / 10.0
y = SIN(x)
PLOT, y, x
```

### Example 2: Histogram
```xdl
seed = 42
data = RANDOMU(seed, 1000)
HISTOGRAM, data
```

### Example 3: Bar Chart
```xdl
values = [10, 25, 15, 30, 20]
BAR_PLOT, values
```

### Example 4: With Colors
```xdl
LOADCT, 13  ; Rainbow colors
x = FINDGEN(50)
y = SIN(x / 5.0)
PLOT, y, x
```

---

## 🛠️ Available Commands

### Plotting
- `PLOT` - Line plots
- `OPLOT` - Overplot
- `CONTOUR` - Contour plots
- `SURFACE` - 3D surfaces
- `HISTOGRAM` - Histograms
- `BAR_PLOT` - Bar charts
- `PLOT3D` - 3D line plots

### Window Management
- `WINDOW, n` - Create window n
- `WSET, n` - Switch to window n
- `WDELETE, n` - Delete window n
- `ERASE` - Clear current window

### Colors
- `LOADCT, table_num` - Load color table
  - 0 = Grayscale
  - 1 = Blue-Red
  - 2 = Blue-White
  - 3 = Green-Red-Blue-White
  - 13 = Rainbow

### Image Display
- `TV, image` - Display image
- `TVSCL, image` - Display with scaling

---

## 🎮 GUI Features

**In the main GUI window:**
- **File → Open** - Load .xdl scripts
- **File → Save** - Save your code
- **Execute Button** - Run current script
- **Clear Button** - Reset editor
- **Variable Browser** - View all variables

**In plot windows:**
- **Resizable** - Drag corners to resize
- **Plot Info Button** - View statistics
- **Close** - Continue script execution

---

## 💡 Pro Tips

1. **Use the GUI** for interactive work
2. **Use CLI** (`./target/release/xdl script.xdl`) for batch jobs
3. **Close plot windows** to continue script execution
4. **Multiple plots?** Each PLOT command creates a new window
5. **Save plots?** Take screenshots or export to PNG

---

## 🐛 Troubleshooting

**Q: Plot window doesn't appear**
A: Make sure you're using `xdl-gui`, not `xdl` CLI

**Q: Get PNG files instead**
A: That's the CLI fallback - use GUI for interactive plots

**Q: GUI won't start**
A: Make sure FLTK dependencies are installed (they should be if it compiled)

---

## 📚 More Information

- **Full Documentation**: `docs/GRAPHICS_IMPLEMENTATION.md`
- **Examples**: `examples/plot_demo.xdl`
- **Test Scripts**: `tests/test_graphics.xdl`

---

## 🎉 You're Ready!

**Try it now:**
```bash
./target/release/xdl-gui
```

Load `examples/plot_demo.xdl` and click **Execute**!

**Enjoy your new graphics system!** 🎨📊✨

# VIZ3D Quick Start Guide

## 🚀 In 30 Seconds

```xdl
; 1. Create volume
volume = FLTARR(64, 64, 64)
; ... fill with your data ...

; 2. Visualize
VIZ3D_INIT
VIZ3D_COLORMAP, 'VIRIDIS'
VIZ3D_VOLUME, volume, DIMENSIONS=[64, 64, 64]
VIZ3D_RENDER, /INTERACTIVE

; 3. Done! Browser opens automatically 🎉
```

## ✨ What You Get

- ✅ **Automatic browser window** - No manual setup
- ✅ **60 FPS rendering** - GPU-accelerated WebGPU
- ✅ **Interactive controls** - Rotate, zoom, adjust parameters
- ✅ **Multiple tabs** - Run unlimited visualizations
- ✅ **No crashes** - Stable and reliable

## 🎨 Available Colormaps

```xdl
VIZ3D_COLORMAP, 'VIRIDIS'   ; Default - blue to yellow
VIZ3D_COLORMAP, 'RAINBOW'   ; Full spectrum
VIZ3D_COLORMAP, 'PLASMA'    ; Purple to yellow
VIZ3D_COLORMAP, 'INFERNO'   ; Black to white (hot)
VIZ3D_COLORMAP, 'TURBO'     ; Vibrant rainbow
VIZ3D_COLORMAP, 'GRAYSCALE' ; Black to white
```

## 📋 Full Example

```xdl
; Create a 32x32x32 sphere
grid = 32
volume = FLTARR(grid, grid, grid)
center = grid / 2.0

FOR i = 0, grid-1 DO BEGIN
    FOR j = 0, grid-1 DO BEGIN
        FOR k = 0, grid-1 DO BEGIN
            dx = i - center
            dy = j - center
            dz = k - center
            r = SQRT(dx*dx + dy*dy + dz*dz)
            volume[i, j, k] = EXP(-r / 5.0)
        END
    END
END

; Visualize
VIZ3D_INIT, TITLE='My Sphere'
VIZ3D_COLORMAP, 'RAINBOW'
VIZ3D_CAMERA, POSITION=[0.0, 0.0, 3.0]
VIZ3D_VOLUME, volume, DIMENSIONS=[grid, grid, grid]
VIZ3D_RENDER, /INTERACTIVE
```

## 🎮 Browser Controls

Once the browser opens:

| Action | Control |
|--------|---------|
| **Rotate** | Left mouse drag |
| **Zoom** | Mouse wheel |
| **Reset view** | Click "Reset Camera" button |
| **Change colormap** | Select from dropdown |
| **Adjust opacity** | Use opacity slider |
| **Filter noise** | Use threshold slider |

## 🔧 Advanced Options

### Non-Interactive Mode (No Browser)

```xdl
VIZ3D_RENDER  ; Without /INTERACTIVE
```

### Custom Window Size

```xdl
VIZ3D_INIT, WINDOW_SIZE=[1920, 1080]
```

### Camera Position

```xdl
VIZ3D_CAMERA, POSITION=[x, y, z], TARGET=[0, 0, 0], FOV=45.0
```

### Fallback to Native Window

```bash
VIZ3D_BROWSER=0 ./target/release/xdl your_script.xdl
```

## 🧪 Test It

```bash
# Quick test (32³ sphere, < 1 second)
./target/release/xdl examples/demo/viz3d_browser_test.xdl

# Full showcase (4 demos, ~60 seconds)
./target/release/xdl examples/demo/viz3d_showcase.xdl
```

## 📊 Performance

| Volume Size | Voxels | Load Time | FPS |
|-------------|--------|-----------|-----|
| 32³ | 32,768 | < 100ms | 60 |
| 64³ | 262,144 | < 500ms | 45-60 |
| 128³ | 2,097,152 | < 2s | 30-40 |

## ❓ Common Issues

**Browser doesn't open?**

- Look for URL in console: `http://localhost:XXXXX`
- Manually open that URL

**Black screen in browser?**

- Check volume data isn't all zeros
- Try a different colormap
- Open browser DevTools (F12) for errors

**"WebGPU not supported" error?**

- Update browser to Chrome 113+
- Or enable WebGPU in browser flags

## 📚 More Info

- **Full Guide**: `VIZ3D_BROWSER_GUIDE.md`
- **Test Results**: `VIZ3D_BROWSER_TEST_RESULTS.md`
- **Implementation**: `VIZ3D_BROWSER_IMPLEMENTATION_SUMMARY.md`

## 💡 Tips

1. **Start small**: Test with 32³ before going to 128³
2. **Use keyboard shortcuts**: Browser has its own dev tools
3. **Multiple tabs**: Each demo opens in a new tab - view side by side!
4. **Keep XDL running**: Server needs to stay active for browser to work

---

**Ready to visualize?** Just run your XDL script and watch your data come to life! 🎨

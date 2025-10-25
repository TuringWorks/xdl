# VIZ3D Three.js Performance Tuning Guide

**Date:** 2025-10-25
**Version:** Phase 2

---

## Overview

This guide helps you optimize Three.js volume rendering performance based on your volume size and hardware capabilities.

---

## Key Parameters

### 1. Step Size (`u_stepSize`)
**Default:** `0.01`

Controls the distance between ray samples. Smaller = better quality, slower performance.

| Step Size | Quality | Performance | Use Case |
|-----------|---------|-------------|----------|
| 0.001 | Excellent | Very Slow | High-quality screenshots, small volumes (< 16³) |
| 0.005 | High | Slow | Detailed exploration, medium volumes (32³-64³) |
| 0.01 | Good | Fast | Default, balanced quality/speed |
| 0.02 | Medium | Very Fast | Large volumes (> 128³), real-time interaction |
| 0.05 | Low | Fastest | Initial exploration, huge volumes |

**Formula:** `stepSize ≈ 1.0 / (volumeDim × desiredSamples)`

Example for 64³ volume:
- High quality: `1.0 / (64 × 4) = 0.0039`
- Medium: `1.0 / (64 × 2) = 0.0078`
- Fast: `1.0 / (64 × 1) = 0.0156`

### 2. Max Steps (`u_maxSteps`)
**Default:** `256`

Maximum number of ray samples per pixel.

| Max Steps | Memory | Performance | Use Case |
|-----------|--------|-------------|----------|
| 64 | Low | Very Fast | Quick previews |
| 128 | Low | Fast | Interactive exploration |
| 256 | Medium | Good | Default balanced setting |
| 512 | High | Slow | High-quality rendering |

**Auto-calculation:**
```
maxSteps = ceil(sqrt(3) / stepSize)
```
This ensures rays can traverse the volume diagonal.

### 3. Threshold (`u_threshold`)
**Default:** `0.1`

Filters out low-density voxels.

- Lower (0.0-0.1): Show all data, slower
- Medium (0.1-0.3): Good default, balanced
- Higher (0.3-0.5): Only show dense regions, faster

**Performance Impact:** Higher thresholds can improve performance by 2-5x by skipping empty space.

### 4. Lighting (`u_enableLighting`)
**Default:** `true`

Gradient-based Phong lighting.

**Performance Cost:** ~15-20% FPS reduction
**Visual Benefit:** Much better depth perception and structure visibility

**Recommendation:**
- Disable for fast interaction
- Enable for final visualization

---

## Recommended Settings by Volume Size

### Small Volumes (< 32³, < 32K voxels)

```xdl
; Optimize for quality
stepSize: 0.005
maxSteps: 512
enableLighting: true
threshold: 0.05
```

**Expected FPS:** 60 FPS

### Medium Volumes (32³-64³, 32K-262K voxels)

```xdl
; Balanced quality/performance
stepSize: 0.01
maxSteps: 256
enableLighting: true
threshold: 0.1
```

**Expected FPS:** 30-45 FPS

### Large Volumes (64³-128³, 262K-2M voxels)

```xdl
; Optimize for performance
stepSize: 0.015
maxSteps: 192
enableLighting: false
threshold: 0.2
```

**Expected FPS:** 15-30 FPS

### Huge Volumes (> 128³, > 2M voxels)

```xdl
; Maximum performance
stepSize: 0.02
maxSteps: 128
enableLighting: false
threshold: 0.3
```

**Expected FPS:** 5-15 FPS

---

## Hardware Considerations

### GPU Performance Tiers

**High-End (e.g., NVIDIA RTX 4080, Apple M3 Max)**
- Can handle large volumes at default settings
- Increase maxSteps to 512 for better quality
- Reduce stepSize to 0.005 for maximum detail

**Mid-Range (e.g., NVIDIA GTX 1660, Apple M1)**
- Use recommended settings above
- May need to reduce quality for volumes > 64³

**Low-End (e.g., Integrated graphics)**
- Reduce stepSize to 0.02
- Reduce maxSteps to 128
- Disable lighting
- Increase threshold to 0.3+

### Browser Performance

**Chrome/Edge (Chromium)**
- Best WebGL performance
- Use default settings

**Firefox**
- ~10-15% slower than Chrome
- Reduce maxSteps by 25% if needed

**Safari**
- ~20-30% slower than Chrome
- Use more aggressive performance settings

---

## Optimization Techniques

### 1. Adaptive Quality

Start with low quality for interaction, increase for final view:

```javascript
// In GUI controller
params.interacting = false;

controls.addEventListener('start', () => {
    params.interacting = true;
    material.uniforms.u_stepSize.value = 0.02;  // Fast
    material.uniforms.u_maxSteps.value = 128;
});

controls.addEventListener('end', () => {
    params.interacting = false;
    material.uniforms.u_stepSize.value = 0.01;  // Quality
    material.uniforms.u_maxSteps.value = 256;
});
```

### 2. Level of Detail (LOD)

Use lower resolution volumes for distant views:

```xdl
; Create LOD hierarchy
volume_high = CONGRID(volume, 128, 128, 128)  ; Full detail
volume_med = CONGRID(volume, 64, 64, 64)      ; 1/8 voxels
volume_low = CONGRID(volume, 32, 32, 32)      ; 1/64 voxels

; Switch based on zoom level
```

### 3. Empty Space Skipping

Increase threshold to skip transparent regions:

```javascript
// Automatically adjust threshold based on histogram
const histogram = computeHistogram(volumeData);
const cumulativeSum = computeCumulativeSum(histogram);
const threshold = findPercentile(cumulativeSum, 0.05);  // Skip bottom 5%
```

---

## Profiling

### Browser DevTools

Open Chrome DevTools (F12) → Performance tab:

1. Start recording
2. Interact with volume
3. Stop recording
4. Look for:
   - **GPU utilization:** Should be > 80%
   - **Frame time:** < 16ms for 60 FPS
   - **JavaScript time:** Should be minimal (< 1ms)

### Performance Metrics

Add FPS counter to GUI:

```javascript
const stats = {
    fps: 0,
    frameCount: 0,
    lastTime: performance.now(),
};

function animate() {
    const now = performance.now();
    stats.frameCount++;
    if (now - stats.lastTime > 1000) {
        stats.fps = Math.round(stats.frameCount / ((now - stats.lastTime) / 1000));
        stats.frameCount = 0;
        stats.lastTime = now;
        console.log(`FPS: ${stats.fps}`);
    }

    requestAnimationFrame(animate);
    renderer.render(scene, camera);
}
```

---

## Troubleshooting

### Problem: Low FPS (< 15)

**Solutions:**
1. Increase stepSize to 0.02
2. Reduce maxSteps to 128
3. Disable lighting
4. Increase threshold
5. Close other GPU-intensive applications

### Problem: Blocky/Pixelated Appearance

**Solutions:**
1. Reduce stepSize (more samples)
2. Increase maxSteps
3. Enable lighting for better structure
4. Check volume data quality

### Problem: Empty/Black Window

**Solutions:**
1. Lower threshold to 0.0
2. Check volume data range (normalize to [0, 1])
3. Increase opacity
4. Verify colormap is loading

### Problem: Browser Crashes

**Solutions:**
1. Reduce volume size
2. Reduce maxSteps significantly (< 128)
3. Check available GPU memory
4. Try a different browser (Chrome recommended)

---

## Best Practices

1. **Start with defaults** - Adjust only if needed
2. **Profile first** - Use DevTools to identify bottlenecks
3. **Test incrementally** - Change one parameter at a time
4. **Cache volumes** - Don't regenerate data unnecessarily
5. **Use appropriate data types** - Float32 for volume data
6. **Normalize data** - Keep values in [0, 1] range
7. **Monitor GPU memory** - Large volumes can exhaust VRAM

---

## Example Configurations

### High-Quality Screenshot

```javascript
params.stepSize = 0.003;
params.maxSteps = 512;
params.enableLighting = true;
params.threshold = 0.02;
params.ambient = 0.2;
params.diffuse = 0.7;
params.specular = 0.4;
params.shininess = 64.0;
```

### Real-Time Interaction

```javascript
params.stepSize = 0.015;
params.maxSteps = 192;
params.enableLighting = false;
params.threshold = 0.15;
```

### Balanced Default

```javascript
params.stepSize = 0.01;
params.maxSteps = 256;
params.enableLighting = true;
params.threshold = 0.1;
params.ambient = 0.3;
params.diffuse = 0.6;
params.specular = 0.3;
params.shininess = 32.0;
```

---

## Performance Checklist

- [ ] Volume size appropriate for hardware?
- [ ] Step size optimized for volume dimensions?
- [ ] Max steps set based on quality needs?
- [ ] Threshold adjusted to skip empty space?
- [ ] Lighting disabled if not needed?
- [ ] Browser using GPU acceleration?
- [ ] No other GPU-intensive apps running?
- [ ] Volume data normalized to [0, 1]?
- [ ] FPS monitoring enabled?
- [ ] Tested in target browser?

---

**Status:** Phase 2 Complete
**Last Updated:** 2025-10-25

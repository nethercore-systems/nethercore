# Unified Shading State - Billboard/Particle Rendering Issue

## Problem
Billboard and platformer examples show some sprites but particles/multiple objects aren't rendering correctly.

## What's Working
✅ Shading state interning - 38 unique states created with correct colors
✅ Color values in shading states include proper alpha (e.g., `0xFFF30457` = RGBA)
✅ Triangle example works correctly
✅ Some billboards render (trees, UI)

## What's Not Working
❌ Particles with varying alpha don't appear
❌ Multiple colored sprites in same scene

## Investigation Findings

### Color Flow
1. **Game code**: Particles create colors like `0xFFF30457` (RGBA: 255,243,4,87)
2. **Billboard generation**: Extracts RGB only for vertices (no alpha in vertex data)
   ```rust
   let r = ((color >> 24) & 0xFF) as f32 / 255.0;  // 1.0
   let g = ((color >> 16) & 0xFF) as f32 / 255.0;  // 0.95
   let b = ((color >> 8) & 0xFF) as f32 / 255.0;   // 0.016
   // Alpha NOT extracted - vertex format is RGB only
   ```
3. **Shading state**: Gets full RGBA color `0xFFF30457`
4. **Shader**: 
   - Unpacks shading state color as RGBA
   - Multiplies RGB with vertex RGB
   - Returns shading state alpha

### Vertex Format Limitation
- `FORMAT_COLOR` = RGB only (3 floats, 12 bytes)
- No per-vertex alpha support
- Alpha must come from shading state

### Potential Issues
1. **Texture alpha multiplication**: Particle texture has alpha, might be multiplying with shading alpha
2. **Depth testing**: Particles might be behind other objects
3. **Blend mode**: Might not be set correctly for particles
4. **Shader color multiplication**: Vertex RGB * Shading RGB might produce very dark colors

## Next Steps
1. Add logging to see actual rendered commands
2. Check if particles are being culled
3. Verify blend modes are correct
4. Test with simpler colors (full opacity white particles)
5. Check if texture alpha is being handled correctly in shader

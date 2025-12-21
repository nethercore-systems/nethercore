// =============================================================================
// MANUALLY MAINTAINED HELPER FUNCTIONS
// =============================================================================
// These helpers provide language-specific conveniences for C/C++ developers

// Color packing helpers
static inline uint32_t nczx_rgba(uint8_t r, uint8_t g, uint8_t b, uint8_t a) {
    return ((uint32_t)r << 24) | ((uint32_t)g << 16) | ((uint32_t)b << 8) | (uint32_t)a;
}

static inline uint32_t nczx_rgb(uint8_t r, uint8_t g, uint8_t b) {
    return nczx_rgba(r, g, b, 255);
}

// Math helpers
static inline float nczx_clampf(float val, float min, float max) {
    return (val < min) ? min : ((val > max) ? max : val);
}

static inline float nczx_lerpf(float a, float b, float t) {
    return a + (b - a) * t;
}

static inline float nczx_minf(float a, float b) {
    return (a < b) ? a : b;
}

static inline float nczx_maxf(float a, float b) {
    return (a > b) ? a : b;
}

static inline float nczx_absf(float x) {
    return (x < 0.0f) ? -x : x;
}

// String literal helpers (use sizeof() for compile-time length calculation)
#define EWZX_LOG(str) log((const uint8_t*)(str), sizeof(str) - 1)

#define EWZX_DRAW_TEXT(str, x, y, size, color) \
    draw_text((const uint8_t*)(str), sizeof(str) - 1, (x), (y), (size), (color))

// ROM loading helpers
#define EWZX_ROM_TEXTURE(id) rom_texture((uint32_t)(id), sizeof(id) - 1)
#define EWZX_ROM_MESH(id) rom_mesh((uint32_t)(id), sizeof(id) - 1)
#define EWZX_ROM_SOUND(id) rom_sound((uint32_t)(id), sizeof(id) - 1)
#define EWZX_ROM_FONT(id) rom_font((uint32_t)(id), sizeof(id) - 1)
#define EWZX_ROM_SKELETON(id) rom_skeleton((uint32_t)(id), sizeof(id) - 1)

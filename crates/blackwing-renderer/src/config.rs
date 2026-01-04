//! Renderer configuration.

/// Configuration for the renderer.
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// Target render width in pixels (internal resolution).
    /// NES Zelda visible area: 256 pixels wide.
    pub target_width: u32,
    /// Target render height in pixels (internal resolution).
    /// NES Zelda visible area: ~176 pixels (11 tiles * 16).
    pub target_height: u32,
    /// Scale factor for display (integer scaling).
    pub scale: u32,
    /// Whether to use pixel-perfect (integer) scaling only.
    pub pixel_perfect: bool,
    /// Background color (RGBA).
    pub clear_color: [f32; 4],
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            target_width: 256,
            target_height: 176, // 11 tiles * 16 pixels
            scale: 3,
            pixel_perfect: true,
            clear_color: [0.0, 0.0, 0.0, 1.0],
        }
    }
}

impl RenderConfig {
    /// Create a config with NES Zelda-style dimensions.
    pub fn zelda() -> Self {
        Self::default()
    }

    /// Create a config with custom dimensions.
    pub fn custom(width: u32, height: u32, scale: u32) -> Self {
        Self {
            target_width: width,
            target_height: height,
            scale,
            ..Default::default()
        }
    }

    /// Get the display width (target * scale).
    pub fn display_width(&self) -> u32 {
        self.target_width * self.scale
    }

    /// Get the display height (target * scale).
    pub fn display_height(&self) -> u32 {
        self.target_height * self.scale
    }
}

//! Quartz-inspired native GPUI Grid color tokens.
//!
//! Values are independently chosen to deliver AG Grid-like density/contrast,
//! not copied from the AG Grid theme CSS or Enterprise packages.
//! These are the baseline for future configurable native GridTheme APIs.
#[derive(Clone, Copy, Debug)]
pub struct GridPalette {
    pub background: u32,
    pub header_background: u32,
    pub surface: u32,
    pub foreground: u32,
    pub muted: u32,
    pub border: u32,
    pub accent: u32,
    pub selected_background: u32,
    pub editor_background: u32,
    pub editor_border: u32,
}

pub const QUARTZ_LIGHT: GridPalette = GridPalette {
    background: 0xffffff,
    header_background: 0xf8fafb,
    surface: 0xffffff,
    foreground: 0x25313d,
    muted: 0x6b7b89,
    border: 0xe2e7eb,
    accent: 0x237fda,
    selected_background: 0xe0efff,
    editor_background: 0xf1f8ff,
    editor_border: 0x237fda,
};

pub const QUARTZ_DARK: GridPalette = GridPalette {
    background: 0x18252d,
    header_background: 0x24323b,
    surface: 0x203038,
    foreground: 0xe4eef1,
    muted: 0x9eafb8,
    border: 0x3b4b54,
    accent: 0x6db6f4,
    selected_background: 0x26445d,
    editor_background: 0x243f57,
    editor_border: 0x6db6f4,
};

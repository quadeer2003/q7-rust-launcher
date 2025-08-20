use eframe::egui::Color32;

#[derive(Clone, Copy)]
pub struct ThemePalette {
    pub bg: Color32,
    pub input_bg: Color32,
    pub selection_bg: Color32,
    pub fg: Color32,
    pub muted: Color32,
}

impl ThemePalette {
    const fn rgb(hex: u32) -> Color32 {
        Color32::from_rgb(
            ((hex >> 16) & 0xFF) as u8,
            ((hex >> 8) & 0xFF) as u8,
            (hex & 0xFF) as u8,
        )
    }

    pub fn dracula() -> Self {
        Self {
            bg: Self::rgb(0x282A36),
            input_bg: Self::rgb(0x3B3E4A),
            selection_bg: Self::rgb(0x44475A),
            fg: Self::rgb(0xF8F8F2),
            muted: Self::rgb(0xB9BBC5),
        }
    }

    pub fn solarized_dark() -> Self {
        Self {
            bg: Self::rgb(0x002B36), // base03
            input_bg: Self::rgb(0x073642),
            selection_bg: Self::rgb(0x073642),
            fg: Self::rgb(0x839496), // base0
            muted: Self::rgb(0x586e75),
        }
    }

    pub fn tokyonight() -> Self {
        Self {
            bg: Self::rgb(0x1A1B26),
            input_bg: Self::rgb(0x2A2E3F),
            selection_bg: Self::rgb(0x2F334D),
            fg: Self::rgb(0xC0CAF5),
            muted: Self::rgb(0x9AA5CE),
        }
    }

    pub fn catppuccin() -> Self {
        Self {
            bg: Self::rgb(0x1E1E2E), // Mocha base
            input_bg: Self::rgb(0x313244),
            selection_bg: Self::rgb(0x313244),
            fg: Self::rgb(0xCDD6F4),
            muted: Self::rgb(0xA6ADC8),
        }
    }

    pub fn gruvbox_dark() -> Self {
        Self {
            bg: Self::rgb(0x282828),
            input_bg: Self::rgb(0x3C3836),
            selection_bg: Self::rgb(0x3C3836),
            fg: Self::rgb(0xEBDBB2),
            muted: Self::rgb(0xBDAE93),
        }
    }

    pub fn iceberg_dark() -> Self {
        Self {
            bg: Self::rgb(0x161821),
            input_bg: Self::rgb(0x1F2230),
            selection_bg: Self::rgb(0x2E313F),
            fg: Self::rgb(0xC6C8D1),
            muted: Self::rgb(0xA7ADBA),
        }
    }

    pub fn bluloco_dark() -> Self {
        Self {
            bg: Self::rgb(0x1E1E1E),
            input_bg: Self::rgb(0x2C2C2C),
            selection_bg: Self::rgb(0x2F343F),
            fg: Self::rgb(0xE5E7EB),
            muted: Self::rgb(0x9AA0A6),
        }
    }

    pub fn nord() -> Self {
        Self {
            bg: Self::rgb(0x2E3440),
            input_bg: Self::rgb(0x3B4252),
            selection_bg: Self::rgb(0x434C5E),
            fg: Self::rgb(0xECEFF4),
            muted: Self::rgb(0xD8DEE9),
        }
    }

    pub fn one_dark() -> Self {
        Self {
            bg: Self::rgb(0x282C34),
            input_bg: Self::rgb(0x30343C),
            selection_bg: Self::rgb(0x3E4451),
            fg: Self::rgb(0xECEFF4),
            muted: Self::rgb(0x98A2B3),
        }
    }

    pub fn monokai_pro() -> Self {
        Self {
            bg: Self::rgb(0x2D2A2E),
            input_bg: Self::rgb(0x38353A),
            selection_bg: Self::rgb(0x403E43),
            fg: Self::rgb(0xFCFCFA),
            muted: Self::rgb(0xA59F85),
        }
    }

    pub fn horizon_dark() -> Self {
        Self {
            bg: Self::rgb(0x1C1E26),
            input_bg: Self::rgb(0x262833),
            selection_bg: Self::rgb(0x2E303E),
            fg: Self::rgb(0xE0E0E0),
            muted: Self::rgb(0x9CA3AF),
        }
    }

    pub fn night_owl() -> Self {
        Self {
            bg: Self::rgb(0x011627),
            input_bg: Self::rgb(0x0B2942),
            selection_bg: Self::rgb(0x103554),
            fg: Self::rgb(0xD6DEEB),
            muted: Self::rgb(0xA1B6E3),
        }
    }

    pub fn ayu_dark() -> Self {
        Self {
            bg: Self::rgb(0x0F1419),
            input_bg: Self::rgb(0x1A1F26),
            selection_bg: Self::rgb(0x1F2430),
            fg: Self::rgb(0xE6E1CF),
            muted: Self::rgb(0x9DA5B4),
        }
    }

    pub fn moonlight() -> Self {
        Self {
            bg: Self::rgb(0x1E2030),
            input_bg: Self::rgb(0x222436),
            selection_bg: Self::rgb(0x2F334D),
            fg: Self::rgb(0xC8D3F5),
            muted: Self::rgb(0xA9B8E8),
        }
    }

    pub fn material_dark() -> Self {
        Self {
            bg: Self::rgb(0x212121),
            input_bg: Self::rgb(0x2A2A2A),
            selection_bg: Self::rgb(0x373737),
            fg: Self::rgb(0xEEEEEE),
            muted: Self::rgb(0xBDBDBD),
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "dracula" => Some(Self::dracula()),
            "solarized dark" | "solarized-dark" | "solarized" => Some(Self::solarized_dark()),
            "tokyonight" => Some(Self::tokyonight()),
            "catppuccin" => Some(Self::catppuccin()),
            "gruvbox dark" | "gruvbox-dark" | "gruvbox" => Some(Self::gruvbox_dark()),
            "iceberg dark" | "iceberg" => Some(Self::iceberg_dark()),
            "bluloco dark" | "bluloco" => Some(Self::bluloco_dark()),
            "nord" => Some(Self::nord()),
            "one dark" | "one-dark" | "onedark" => Some(Self::one_dark()),
            "monokai pro" | "monokai-pro" | "monokaipro" | "monokai" => Some(Self::monokai_pro()),
            "horizon dark" | "horizon-dark" | "horizon" => Some(Self::horizon_dark()),
            "night owl" | "night-owl" | "nightowl" => Some(Self::night_owl()),
            "ayu dark" | "ayu-dark" | "ayu" => Some(Self::ayu_dark()),
            "moonlight" => Some(Self::moonlight()),
            "material dark" | "material-dark" | "material" => Some(Self::material_dark()),
            _ => None,
        }
    }

    pub fn names() -> &'static [&'static str] {
        &[
            "Dracula",
            "Solarized Dark",
            "Tokyonight",
            "Catppuccin",
            "Gruvbox Dark",
            "Iceberg Dark",
            "Bluloco Dark",
            "Nord",
            "One Dark",
            "Monokai Pro",
            "Horizon Dark",
            "Night Owl",
            "Ayu Dark",
            "Moonlight",
            "Material Dark",
        ]
    }
}
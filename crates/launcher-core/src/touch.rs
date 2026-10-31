//! Virtual on-screen touch controls engine for Horizon on Android.
//! Provides configurable on-screen buttons, analog joysticks, toggle states,
//! and low-end screen optimization without requiring JVM or Java code.

use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::error::{LauncherError, Result};

/// Virtual actions that can be triggered by touch inputs.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TouchAction {
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    Jump,
    Sneak,
    Sprint,
    AttackMine,
    UsePlace,
    Inventory,
    DropItem,
    SwapHand,
    Chat,
    PauseMenu,
    PerspectiveToggle,
    DebugScreen,
    PlayerListTab,
    Zoom,
    PickBlockMid,
    ToggleMouseMode,
    ScrollHotbarPrev,
    ScrollHotbarNext,
    HotbarSlot(u8),
}

/// Visual shape of a virtual touch button.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ButtonShape {
    Circle,
    Rectangle,
    Dpad,
    AnalogJoystick,
}

/// A single on-screen virtual button or control widget.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct VirtualButton {
    pub id: String,
    pub label: String,
    pub action: TouchAction,
    /// X coordinate normalized to screen width (0.0 to 1.0).
    pub x: f32,
    /// Y coordinate normalized to screen height (0.0 to 1.0).
    pub y: f32,
    /// Width normalized to screen width (0.0 to 1.0).
    pub width: f32,
    /// Height normalized to screen height (0.0 to 1.0).
    pub height: f32,
    pub shape: ButtonShape,
    /// Opacity from 0.1 (translucent) to 1.0 (fully opaque).
    pub opacity: f32,
    /// If true, tapping toggles the action active/inactive (e.g. for Sneak or Sprint lock).
    pub toggle: bool,
}

impl VirtualButton {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        action: TouchAction,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Result<Self> {
        let button = Self {
            id: id.into(),
            label: label.into(),
            action,
            x,
            y,
            width,
            height,
            shape: ButtonShape::Rectangle,
            opacity: 0.75,
            toggle: false,
        };
        button.validate()?;
        Ok(button)
    }

    pub fn validate(&self) -> Result<()> {
        if self.id.is_empty() {
            return Err(LauncherError::InvalidProfile(
                "touch button id cannot be empty".into(),
            ));
        }
        if !(0.0..=1.0).contains(&self.x) || !(0.0..=1.0).contains(&self.y) {
            return Err(LauncherError::InvalidProfile(format!(
                "touch button `{}` coordinates must be within [0.0, 1.0]",
                self.id
            )));
        }
        if self.width <= 0.0 || self.height <= 0.0 || self.width > 1.0 || self.height > 1.0 {
            return Err(LauncherError::InvalidProfile(format!(
                "touch button `{}` dimensions must be within (0.0, 1.0]",
                self.id
            )));
        }
        if !(0.1..=1.0).contains(&self.opacity) {
            return Err(LauncherError::InvalidProfile(format!(
                "touch button `{}` opacity must be between 0.1 and 1.0",
                self.id
            )));
        }
        Ok(())
    }

    /// Tests if a normalized touch coordinate `(tx, ty)` hits this button.
    pub fn contains_point(&self, tx: f32, ty: f32) -> bool {
        tx >= self.x && tx <= (self.x + self.width) && ty >= self.y && ty <= (self.y + self.height)
    }
}

/// Virtual Touchpad configuration (inspired by PojavLauncher and Mojo Launcher)
/// enabling laptop trackpad emulation and virtual mouse pointer control on touchscreen.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct VirtualTouchpad {
    pub enabled: bool,
    /// X coordinate normalized to screen width (0.0 to 1.0).
    pub x: f32,
    /// Y coordinate normalized to screen height (0.0 to 1.0).
    pub y: f32,
    /// Width normalized to screen width (0.0 to 1.0).
    pub width: f32,
    /// Height normalized to screen height (0.0 to 1.0).
    pub height: f32,
    /// Mouse tracking sensitivity multiplier (e.g. 1.0).
    pub sensitivity: f32,
    /// Single tap generates a Primary Click (Attack / Mine / Select).
    pub tap_to_click: bool,
    /// Two finger tap generates a Secondary Click (Use / Place).
    pub two_finger_right_click: bool,
    /// Pointer acceleration factor for smooth cursor tracking.
    pub pointer_acceleration: f32,
}

impl Default for VirtualTouchpad {
    fn default() -> Self {
        Self {
            enabled: true,
            x: 0.25,
            y: 0.15,
            width: 0.50,
            height: 0.70,
            sensitivity: 1.0,
            tap_to_click: true,
            two_finger_right_click: true,
            pointer_acceleration: 1.25,
        }
    }
}

impl VirtualTouchpad {
    pub fn contains_point(&self, tx: f32, ty: f32) -> bool {
        self.enabled
            && tx >= self.x
            && tx <= (self.x + self.width)
            && ty >= self.y
            && ty <= (self.y + self.height)
    }

    pub fn validate(&self) -> Result<()> {
        if !(0.0..=1.0).contains(&self.x) || !(0.0..=1.0).contains(&self.y) {
            return Err(LauncherError::InvalidProfile(
                "touchpad coordinates must be within [0.0, 1.0]".into(),
            ));
        }
        if self.width <= 0.0 || self.height <= 0.0 || self.width > 1.0 || self.height > 1.0 {
            return Err(LauncherError::InvalidProfile(
                "touchpad dimensions must be within (0.0, 1.0]".into(),
            ));
        }
        if self.sensitivity <= 0.0 || self.sensitivity > 10.0 {
            return Err(LauncherError::InvalidProfile(
                "touchpad sensitivity must be within (0.0, 10.0]".into(),
            ));
        }
        Ok(())
    }
}

/// A complete on-screen touch layout configuration.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TouchLayout {
    pub version: u32,
    pub name: String,
    pub scale: f32,
    pub haptic_feedback: bool,
    pub analog_deadzone: f32,
    pub buttons: Vec<VirtualButton>,
    #[serde(default)]
    pub touchpad: Option<VirtualTouchpad>,
}

impl Default for TouchLayout {
    fn default() -> Self {
        Self::default_phone()
    }
}

impl TouchLayout {
    /// Standard smartphone layout with D-Pad on bottom-left, action cluster on bottom-right,
    /// and menu/inventory at top/middle.
    pub fn default_phone() -> Self {
        let buttons = vec![
            // Movement cluster (bottom left)
            VirtualButton {
                id: "btn_forward".into(),
                label: "▲".into(),
                action: TouchAction::MoveForward,
                x: 0.08,
                y: 0.65,
                width: 0.08,
                height: 0.12,
                shape: ButtonShape::Rectangle,
                opacity: 0.70,
                toggle: false,
            },
            VirtualButton {
                id: "btn_backward".into(),
                label: "▼".into(),
                action: TouchAction::MoveBackward,
                x: 0.08,
                y: 0.82,
                width: 0.08,
                height: 0.12,
                shape: ButtonShape::Rectangle,
                opacity: 0.70,
                toggle: false,
            },
            VirtualButton {
                id: "btn_left".into(),
                label: "◀".into(),
                action: TouchAction::MoveLeft,
                x: 0.02,
                y: 0.74,
                width: 0.06,
                height: 0.12,
                shape: ButtonShape::Rectangle,
                opacity: 0.70,
                toggle: false,
            },
            VirtualButton {
                id: "btn_right".into(),
                label: "▶".into(),
                action: TouchAction::MoveRight,
                x: 0.16,
                y: 0.74,
                width: 0.06,
                height: 0.12,
                shape: ButtonShape::Rectangle,
                opacity: 0.70,
                toggle: false,
            },
            VirtualButton {
                id: "btn_jump_center".into(),
                label: "◆".into(),
                action: TouchAction::Jump,
                x: 0.085,
                y: 0.745,
                width: 0.07,
                height: 0.09,
                shape: ButtonShape::Circle,
                opacity: 0.60,
                toggle: false,
            },
            // Actions (bottom right)
            VirtualButton {
                id: "btn_attack".into(),
                label: "PRI".into(),
                action: TouchAction::AttackMine,
                x: 0.85,
                y: 0.58,
                width: 0.09,
                height: 0.14,
                shape: ButtonShape::Circle,
                opacity: 0.80,
                toggle: false,
            },
            VirtualButton {
                id: "btn_use".into(),
                label: "SEC".into(),
                action: TouchAction::UsePlace,
                x: 0.74,
                y: 0.58,
                width: 0.09,
                height: 0.14,
                shape: ButtonShape::Circle,
                opacity: 0.80,
                toggle: false,
            },
            VirtualButton {
                id: "btn_jump".into(),
                label: "JUMP".into(),
                action: TouchAction::Jump,
                x: 0.85,
                y: 0.76,
                width: 0.10,
                height: 0.16,
                shape: ButtonShape::Circle,
                opacity: 0.80,
                toggle: false,
            },
            VirtualButton {
                id: "btn_sneak".into(),
                label: "SNK".into(),
                action: TouchAction::Sneak,
                x: 0.74,
                y: 0.76,
                width: 0.08,
                height: 0.12,
                shape: ButtonShape::Rectangle,
                opacity: 0.75,
                toggle: true,
            },
            // System and menu bars (top)
            VirtualButton {
                id: "btn_pause".into(),
                label: "ESC".into(),
                action: TouchAction::PauseMenu,
                x: 0.02,
                y: 0.03,
                width: 0.07,
                height: 0.08,
                shape: ButtonShape::Rectangle,
                opacity: 0.65,
                toggle: false,
            },
            VirtualButton {
                id: "btn_chat".into(),
                label: "CHAT".into(),
                action: TouchAction::Chat,
                x: 0.55,
                y: 0.03,
                width: 0.07,
                height: 0.08,
                shape: ButtonShape::Rectangle,
                opacity: 0.65,
                toggle: false,
            },
            VirtualButton {
                id: "btn_inventory".into(),
                label: "INV".into(),
                action: TouchAction::Inventory,
                x: 0.45,
                y: 0.88,
                width: 0.10,
                height: 0.09,
                shape: ButtonShape::Rectangle,
                opacity: 0.85,
                toggle: false,
            },
            VirtualButton {
                id: "btn_f5".into(),
                label: "F5".into(),
                action: TouchAction::PerspectiveToggle,
                x: 0.44,
                y: 0.03,
                width: 0.06,
                height: 0.08,
                shape: ButtonShape::Rectangle,
                opacity: 0.65,
                toggle: false,
            },
            VirtualButton {
                id: "btn_f3".into(),
                label: "F3".into(),
                action: TouchAction::DebugScreen,
                x: 0.35,
                y: 0.03,
                width: 0.06,
                height: 0.08,
                shape: ButtonShape::Rectangle,
                opacity: 0.65,
                toggle: false,
            },
        ];

        Self {
            version: 1,
            name: "Default Phone".into(),
            scale: 1.0,
            haptic_feedback: true,
            analog_deadzone: 0.15,
            buttons,
            touchpad: None,
        }
    }

    /// Compact layout specifically tuned for low-end, smaller screen devices.
    /// Uses fewer buttons with higher contrast and larger touch areas.
    pub fn compact_low_end() -> Self {
        let buttons = vec![
            VirtualButton {
                id: "stick_move".into(),
                label: "MOVE".into(),
                action: TouchAction::MoveForward,
                x: 0.04,
                y: 0.60,
                width: 0.22,
                height: 0.35,
                shape: ButtonShape::AnalogJoystick,
                opacity: 0.50,
                toggle: false,
            },
            VirtualButton {
                id: "btn_action_pri".into(),
                label: "MINE".into(),
                action: TouchAction::AttackMine,
                x: 0.80,
                y: 0.55,
                width: 0.16,
                height: 0.18,
                shape: ButtonShape::Circle,
                opacity: 0.70,
                toggle: false,
            },
            VirtualButton {
                id: "btn_action_jump".into(),
                label: "JUMP".into(),
                action: TouchAction::Jump,
                x: 0.80,
                y: 0.75,
                width: 0.16,
                height: 0.20,
                shape: ButtonShape::Circle,
                opacity: 0.70,
                toggle: false,
            },
            VirtualButton {
                id: "btn_inv".into(),
                label: "INV".into(),
                action: TouchAction::Inventory,
                x: 0.44,
                y: 0.88,
                width: 0.12,
                height: 0.10,
                shape: ButtonShape::Rectangle,
                opacity: 0.75,
                toggle: false,
            },
            VirtualButton {
                id: "btn_menu".into(),
                label: "MENU".into(),
                action: TouchAction::PauseMenu,
                x: 0.02,
                y: 0.02,
                width: 0.10,
                height: 0.08,
                shape: ButtonShape::Rectangle,
                opacity: 0.60,
                toggle: false,
            },
        ];

        Self {
            version: 1,
            name: "Compact Low-End".into(),
            scale: 0.9,
            haptic_feedback: false, // Disabling haptics saves battery on budget devices
            analog_deadzone: 0.20,
            buttons,
            touchpad: None,
        }
    }

    /// PojavLauncher / Mojo Launcher compatible layout with in-game virtual touchpad,
    /// mouse cursor emulation mode, dedicated PRI/SEC/MID buttons, and 1-9 hotbar buttons.
    pub fn pojav_touchpad() -> Self {
        let mut buttons = vec![
            // D-Pad / Movement (Bottom Left)
            VirtualButton {
                id: "btn_forward".into(),
                label: "W".into(),
                action: TouchAction::MoveForward,
                x: 0.07,
                y: 0.62,
                width: 0.07,
                height: 0.12,
                shape: ButtonShape::Rectangle,
                opacity: 0.70,
                toggle: false,
            },
            VirtualButton {
                id: "btn_backward".into(),
                label: "S".into(),
                action: TouchAction::MoveBackward,
                x: 0.07,
                y: 0.80,
                width: 0.07,
                height: 0.12,
                shape: ButtonShape::Rectangle,
                opacity: 0.70,
                toggle: false,
            },
            VirtualButton {
                id: "btn_left".into(),
                label: "A".into(),
                action: TouchAction::MoveLeft,
                x: 0.01,
                y: 0.71,
                width: 0.06,
                height: 0.12,
                shape: ButtonShape::Rectangle,
                opacity: 0.70,
                toggle: false,
            },
            VirtualButton {
                id: "btn_right".into(),
                label: "D".into(),
                action: TouchAction::MoveRight,
                x: 0.14,
                y: 0.71,
                width: 0.06,
                height: 0.12,
                shape: ButtonShape::Rectangle,
                opacity: 0.70,
                toggle: false,
            },
            VirtualButton {
                id: "btn_jump_center".into(),
                label: "JUMP".into(),
                action: TouchAction::Jump,
                x: 0.07,
                y: 0.74,
                width: 0.07,
                height: 0.06,
                shape: ButtonShape::Rectangle,
                opacity: 0.65,
                toggle: false,
            },
            // Pojav Mouse & Action Controls (Bottom Right)
            VirtualButton {
                id: "btn_pri".into(),
                label: "PRI".into(),
                action: TouchAction::AttackMine,
                x: 0.85,
                y: 0.54,
                width: 0.08,
                height: 0.13,
                shape: ButtonShape::Circle,
                opacity: 0.80,
                toggle: false,
            },
            VirtualButton {
                id: "btn_sec".into(),
                label: "SEC".into(),
                action: TouchAction::UsePlace,
                x: 0.75,
                y: 0.54,
                width: 0.08,
                height: 0.13,
                shape: ButtonShape::Circle,
                opacity: 0.80,
                toggle: false,
            },
            VirtualButton {
                id: "btn_mid".into(),
                label: "MID".into(),
                action: TouchAction::PickBlockMid,
                x: 0.80,
                y: 0.42,
                width: 0.07,
                height: 0.11,
                shape: ButtonShape::Circle,
                opacity: 0.70,
                toggle: false,
            },
            VirtualButton {
                id: "btn_mouse_toggle".into(),
                label: "MOUSE".into(),
                action: TouchAction::ToggleMouseMode,
                x: 0.91,
                y: 0.42,
                width: 0.07,
                height: 0.11,
                shape: ButtonShape::Rectangle,
                opacity: 0.75,
                toggle: true,
            },
            VirtualButton {
                id: "btn_jump_main".into(),
                label: "UP".into(),
                action: TouchAction::Jump,
                x: 0.85,
                y: 0.72,
                width: 0.09,
                height: 0.15,
                shape: ButtonShape::Circle,
                opacity: 0.85,
                toggle: false,
            },
            VirtualButton {
                id: "btn_sneak".into(),
                label: "SNK".into(),
                action: TouchAction::Sneak,
                x: 0.75,
                y: 0.72,
                width: 0.07,
                height: 0.11,
                shape: ButtonShape::Rectangle,
                opacity: 0.75,
                toggle: true,
            },
            // Top System Bar
            VirtualButton {
                id: "btn_esc".into(),
                label: "ESC".into(),
                action: TouchAction::PauseMenu,
                x: 0.01,
                y: 0.02,
                width: 0.06,
                height: 0.07,
                shape: ButtonShape::Rectangle,
                opacity: 0.65,
                toggle: false,
            },
            VirtualButton {
                id: "btn_tab".into(),
                label: "TAB".into(),
                action: TouchAction::PlayerListTab,
                x: 0.08,
                y: 0.02,
                width: 0.06,
                height: 0.07,
                shape: ButtonShape::Rectangle,
                opacity: 0.60,
                toggle: false,
            },
            VirtualButton {
                id: "btn_f3".into(),
                label: "F3".into(),
                action: TouchAction::DebugScreen,
                x: 0.38,
                y: 0.02,
                width: 0.05,
                height: 0.07,
                shape: ButtonShape::Rectangle,
                opacity: 0.60,
                toggle: false,
            },
            VirtualButton {
                id: "btn_f5".into(),
                label: "F5".into(),
                action: TouchAction::PerspectiveToggle,
                x: 0.44,
                y: 0.02,
                width: 0.05,
                height: 0.07,
                shape: ButtonShape::Rectangle,
                opacity: 0.60,
                toggle: false,
            },
            VirtualButton {
                id: "btn_chat".into(),
                label: "CHAT".into(),
                action: TouchAction::Chat,
                x: 0.50,
                y: 0.02,
                width: 0.07,
                height: 0.07,
                shape: ButtonShape::Rectangle,
                opacity: 0.65,
                toggle: false,
            },
            VirtualButton {
                id: "btn_zoom".into(),
                label: "ZOOM".into(),
                action: TouchAction::Zoom,
                x: 0.58,
                y: 0.02,
                width: 0.06,
                height: 0.07,
                shape: ButtonShape::Rectangle,
                opacity: 0.60,
                toggle: false,
            },
            VirtualButton {
                id: "btn_inv".into(),
                label: "INV".into(),
                action: TouchAction::Inventory,
                x: 0.46,
                y: 0.79,
                width: 0.08,
                height: 0.08,
                shape: ButtonShape::Rectangle,
                opacity: 0.80,
                toggle: false,
            },
        ];

        // Bottom 1-9 Hotbar slots (PojavLauncher style)
        for slot in 1..=9 {
            let start_x = 0.25;
            let slot_w = 0.055;
            let x = start_x + ((slot - 1) as f32 * slot_w);
            buttons.push(VirtualButton {
                id: format!("slot_{slot}"),
                label: format!("{slot}"),
                action: TouchAction::HotbarSlot(slot),
                x,
                y: 0.89,
                width: slot_w * 0.95,
                height: 0.09,
                shape: ButtonShape::Rectangle,
                opacity: 0.70,
                toggle: false,
            });
        }

        Self {
            version: 1,
            name: "Pojav Virtual Touchpad".into(),
            scale: 1.0,
            haptic_feedback: true,
            analog_deadzone: 0.15,
            buttons,
            touchpad: Some(VirtualTouchpad::default()),
        }
    }

    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_vec_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_file(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(LauncherError::MissingFile(path.to_owned()));
        }
        let data = fs::read(path)?;
        let layout: Self = serde_json::from_slice(&data)?;
        for btn in &layout.buttons {
            btn.validate()?;
        }
        if let Some(ref tp) = layout.touchpad {
            tp.validate()?;
        }
        Ok(layout)
    }
}

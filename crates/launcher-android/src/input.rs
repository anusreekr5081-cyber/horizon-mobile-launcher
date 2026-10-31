//! Touch input mapper: translates Android screen coordinates and multi-touch pointers
//! into game actions configured by the active `TouchLayout`.

use std::collections::{HashMap, HashSet};

use horizon_mobile_core::touch::{TouchAction, TouchLayout};

#[derive(Clone, Debug)]
pub struct ActivePointer {
    pub id: i32,
    pub x: f32,
    pub y: f32,
    pub assigned_button_id: Option<String>,
}

#[derive(Debug)]
pub struct TouchInputManager {
    layout: TouchLayout,
    pointers: HashMap<i32, ActivePointer>,
    active_actions: HashSet<TouchAction>,
    toggled_buttons: HashSet<String>,
    mouse_mode: bool,
    cursor_x: f32,
    cursor_y: f32,
    accumulated_dx: f32,
    accumulated_dy: f32,
    touchpad_pointer_id: Option<i32>,
    last_touchpad_x: f32,
    last_touchpad_y: f32,
}

impl TouchInputManager {
    pub fn new(layout: TouchLayout) -> Self {
        Self {
            layout,
            pointers: HashMap::new(),
            active_actions: HashSet::new(),
            toggled_buttons: HashSet::new(),
            mouse_mode: false,
            cursor_x: 0.5,
            cursor_y: 0.5,
            accumulated_dx: 0.0,
            accumulated_dy: 0.0,
            touchpad_pointer_id: None,
            last_touchpad_x: 0.0,
            last_touchpad_y: 0.0,
        }
    }

    pub fn layout(&self) -> &TouchLayout {
        &self.layout
    }

    pub fn set_layout(&mut self, layout: TouchLayout) {
        self.layout = layout;
        self.clear();
    }

    pub fn is_mouse_mode(&self) -> bool {
        self.mouse_mode
    }

    pub fn set_mouse_mode(&mut self, enabled: bool) {
        self.mouse_mode = enabled;
    }

    pub fn cursor_pos(&self) -> (f32, f32) {
        (self.cursor_x, self.cursor_y)
    }

    pub fn set_cursor_pos(&mut self, x: f32, y: f32) {
        self.cursor_x = x.clamp(0.0, 1.0);
        self.cursor_y = y.clamp(0.0, 1.0);
    }

    pub fn take_accumulated_delta(&mut self) -> (f32, f32) {
        let delta = (self.accumulated_dx, self.accumulated_dy);
        self.accumulated_dx = 0.0;
        self.accumulated_dy = 0.0;
        delta
    }

    pub fn is_touchpad_active(&self) -> bool {
        self.touchpad_pointer_id.is_some()
    }

    pub fn is_action_pressed(&self, action: TouchAction) -> bool {
        self.active_actions.contains(&action)
    }

    pub fn active_actions(&self) -> &HashSet<TouchAction> {
        &self.active_actions
    }

    /// Handles a finger pressing down on the screen at normalized coordinates `(norm_x, norm_y)`.
    pub fn on_pointer_down(&mut self, id: i32, norm_x: f32, norm_y: f32) -> Option<TouchAction> {
        let mut hit_action = None;
        let mut hit_button_id = None;

        for button in &self.layout.buttons {
            if button.contains_point(norm_x, norm_y) {
                hit_button_id = Some(button.id.clone());
                if button.action == TouchAction::ToggleMouseMode {
                    self.mouse_mode = !self.mouse_mode;
                }
                if button.toggle {
                    if self.toggled_buttons.contains(&button.id) {
                        self.toggled_buttons.remove(&button.id);
                        self.active_actions.remove(&button.action);
                    } else {
                        self.toggled_buttons.insert(button.id.clone());
                        self.active_actions.insert(button.action);
                        hit_action = Some(button.action);
                    }
                } else {
                    self.active_actions.insert(button.action);
                    hit_action = Some(button.action);
                }
                break;
            }
        }

        // If no button was hit, check if point falls inside virtual touchpad
        if hit_button_id.is_none() {
            if let Some(ref tp) = self.layout.touchpad {
                if tp.contains_point(norm_x, norm_y) {
                    self.touchpad_pointer_id = Some(id);
                    self.last_touchpad_x = norm_x;
                    self.last_touchpad_y = norm_y;
                }
            }
        }

        self.pointers.insert(
            id,
            ActivePointer {
                id,
                x: norm_x,
                y: norm_y,
                assigned_button_id: hit_button_id,
            },
        );
        hit_action
    }

    /// Handles finger dragging across the screen.
    pub fn on_pointer_move(&mut self, id: i32, norm_x: f32, norm_y: f32) {
        if let Some(pointer) = self.pointers.get_mut(&id) {
            pointer.x = norm_x;
            pointer.y = norm_y;
        }

        if Some(id) == self.touchpad_pointer_id {
            if let Some(ref tp) = self.layout.touchpad {
                let dx = (norm_x - self.last_touchpad_x) * tp.sensitivity * tp.pointer_acceleration;
                let dy = (norm_y - self.last_touchpad_y) * tp.sensitivity * tp.pointer_acceleration;
                self.accumulated_dx += dx;
                self.accumulated_dy += dy;
                self.cursor_x = (self.cursor_x + dx).clamp(0.0, 1.0);
                self.cursor_y = (self.cursor_y + dy).clamp(0.0, 1.0);
                self.last_touchpad_x = norm_x;
                self.last_touchpad_y = norm_y;
            }
        }
    }

    /// Handles finger lift off screen.
    pub fn on_pointer_up(&mut self, id: i32) -> Option<TouchAction> {
        if Some(id) == self.touchpad_pointer_id {
            self.touchpad_pointer_id = None;
        }

        let removed = self.pointers.remove(&id);
        if let Some(pointer) = removed {
            if let Some(btn_id) = pointer.assigned_button_id {
                let button = self.layout.buttons.iter().find(|b| b.id == btn_id);
                if let Some(button) = button {
                    if !button.toggle {
                        self.active_actions.remove(&button.action);
                        return Some(button.action);
                    }
                }
            }
        }
        None
    }

    pub fn clear(&mut self) {
        self.pointers.clear();
        self.active_actions.clear();
        self.toggled_buttons.clear();
        self.touchpad_pointer_id = None;
        self.accumulated_dx = 0.0;
        self.accumulated_dy = 0.0;
    }
}

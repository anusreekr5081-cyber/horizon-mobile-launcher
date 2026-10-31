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
}

impl TouchInputManager {
    pub fn new(layout: TouchLayout) -> Self {
        Self {
            layout,
            pointers: HashMap::new(),
            active_actions: HashSet::new(),
            toggled_buttons: HashSet::new(),
        }
    }

    pub fn layout(&self) -> &TouchLayout {
        &self.layout
    }

    pub fn set_layout(&mut self, layout: TouchLayout) {
        self.layout = layout;
        self.clear();
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
    }

    /// Handles finger lift off screen.
    pub fn on_pointer_up(&mut self, id: i32) -> Option<TouchAction> {
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
    }
}

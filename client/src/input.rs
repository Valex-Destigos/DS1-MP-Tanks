//! Client input sampling and wire-protocol conversion.

use crate::camera::ViewportCamera;
use macroquad::prelude::*;
use protocol::InputFrame;

/// Sampled player input for a single frame.
#[derive(Clone, Copy, Debug)]
pub struct PlayerCommand {
    pub frame: InputFrame,
    pub aim_world: Vec2,
    pub quit_requested: bool,
}

/// Polls hardware inputs and tracks the monotonically increasing input sequence number.
#[derive(Default)]
pub struct InputSampler {
    next_seq: u32,
}

impl InputSampler {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Reads keyboard and mouse state for the current frame.
    pub fn sample(&mut self, camera: &ViewportCamera) -> PlayerCommand {
        let mut move_dir = Vec2::ZERO;

        // Standard 2D top-down coordinates: -Y is Up (North), +Y is Down (South).
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            move_dir.y -= 1.0;
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            move_dir.y += 1.0;
        }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            move_dir.x -= 1.0;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            move_dir.x += 1.0;
        }

        let move_dir = move_dir.normalize_or_zero();
        let shoot = is_mouse_button_down(MouseButton::Left) || is_key_down(KeyCode::Space);
        let aim_world = camera.screen_to_world(mouse_position().into());

        let seq = self.next_seq;
        self.next_seq = self.next_seq.wrapping_add(1);

        PlayerCommand {
            frame: InputFrame {
                seq,
                move_dir,
                shoot,
            },
            aim_world,
            quit_requested: is_key_pressed(KeyCode::Escape),
        }
    }
}

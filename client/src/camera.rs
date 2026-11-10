//! Viewport camera controller with smooth target tracking and frustum culling.

use crate::config::{arena, view};
use macroquad::prelude::*;

/// Manages the 2D world-space camera and viewport bounds.
pub struct ViewportCamera {
    center: Vec2,
    view_rect: Rect,
    camera: Camera2D,
}

impl ViewportCamera {
    #[must_use]
    pub fn new(initial_target: Vec2) -> Self {
        let mut cam = Self {
            center: initial_target,
            view_rect: Rect::default(),
            camera: Camera2D::default(),
        };
        cam.rebuild_projection();
        cam
    }

    /// Smoothly interpolates the camera toward `target` and clamps to arena borders.
    pub fn follow(&mut self, target: Vec2, dt: f32) {
        let alpha = 1.0 - (-view::FOLLOW_SHARPNESS * dt).exp();
        self.center = self.center.lerp(target, alpha);
        self.rebuild_projection();
    }

    /// Activates the world-space camera for subsequent draw calls.
    pub fn apply(&self) {
        set_camera(&self.camera);
    }

    /// Converts a screen-space pixel coordinate (e.g. mouse cursor) into world space.
    #[must_use]
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        self.camera.screen_to_world(screen_pos)
    }

    /// World-space rectangle currently visible on screen.
    #[must_use]
    pub const fn visible_rect(&self) -> Rect {
        self.view_rect
    }

    /// Returns `true` if a circle of `radius` at `pos` intersects the viewport.
    #[must_use]
    pub fn is_circle_visible(&self, pos: Vec2, radius: f32) -> bool {
        pos.x + radius >= self.view_rect.x
            && pos.x - radius <= self.view_rect.right()
            && pos.y + radius >= self.view_rect.y
            && pos.y - radius <= self.view_rect.bottom()
    }

    fn rebuild_projection(&mut self) {
        let screen = vec2(screen_width().max(1.0), screen_height().max(1.0));
        let half = screen * 0.5;

        for axis in 0..2 {
            self.center[axis] = if screen[axis] >= arena::SIZE[axis] {
                arena::SIZE[axis] * 0.5
            } else {
                self.center[axis].clamp(half[axis], arena::SIZE[axis] - half[axis])
            };
        }

        self.view_rect = Rect::new(
            self.center.x - half.x,
            self.center.y - half.y,
            screen.x,
            screen.y,
        );

        // Note: Macroquad's `Camera2D::matrix()` automatically negates `zoom.y`
        // when `render_target` is `None`. Using positive `2.0 / screen.y` keeps
        // +X pointing right and +Y pointing down (matching screen & Rect space).
        self.camera = Camera2D {
            target: self.center,
            zoom: vec2(2.0 / screen.x, 2.0 / screen.y),
            ..Default::default()
        };
    }
}

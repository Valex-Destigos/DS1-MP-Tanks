//! Centralized tuning constants and visual palette for the client.

use macroquad::prelude::{Color, Vec2};

/// Arena dimensions and layout constants.
pub mod arena {
    use super::*;

    pub const SIZE: Vec2 = Vec2::new(3000.0, 2000.0);
    pub const BORDER_THICKNESS: f32 = 40.0;
    pub const GRID_SPACING: f32 = 100.0;
    pub const SPAWN_POS: Vec2 = Vec2::new(250.0, 250.0);
    pub const SPAWN_SAFE_RADIUS: f32 = 200.0;
    pub const ROCK_COUNT: usize = 45;
    pub const ROCK_RADIUS_RANGE: (f32, f32) = (18.0, 45.0);
    pub const RNG_SEED: u64 = 42;
}

/// Tank movement, geometry, and tread constants.
pub mod tank {
    use super::*;

    pub const COLLISION_RADIUS: f32 = 22.0;
    pub const MAX_FORWARD_SPEED: f32 = 240.0;
    pub const MAX_REVERSE_SPEED: f32 = 180.0;
    pub const ACCELERATION: f32 = 1100.0;
    pub const DECELERATION: f32 = 1500.0;
    pub const HULL_TURN_SPEED: f32 = 6.5; // rad / s
    pub const GEAR_HYSTERESIS: f32 = 0.15;

    pub const HULL_SIZE: Vec2 = Vec2::new(42.0, 26.0);
    pub const TREAD_SIZE: Vec2 = Vec2::new(48.0, 10.0);
    pub const TREAD_LATERAL_OFFSET: f32 = 15.0;
    pub const TREAD_CLEAT_SPACING: f32 = 8.0;
    pub const TURRET_RADIUS: f32 = 11.0;
    pub const BARREL_LENGTH: f32 = 34.0;
    pub const BARREL_WIDTH: f32 = 7.0;
}

/// Projectile and impact effect constants.
pub mod bullet {
    pub const RADIUS: f32 = 5.0;
    pub const SPEED: f32 = 750.0;
    pub const LIFETIME: f32 = 2.0;
    pub const FIRE_COOLDOWN: f32 = 0.2;
    pub const IMPACT_LIFETIME: f32 = 0.25;
}

/// Camera and simulation step constants.
pub mod view {
    pub const WINDOW_WIDTH: i32 = 1280;
    pub const WINDOW_HEIGHT: i32 = 720;
    pub const FOLLOW_SHARPNESS: f32 = 8.0;
    pub const MAX_DT: f32 = 1.0 / 20.0;
}

/// Color palette for world and HUD rendering.
pub mod palette {
    use super::*;

    pub const BACKGROUND: Color = Color::from_rgba(28, 32, 38, 255);
    pub const GRID_LINE: Color = Color::from_rgba(255, 255, 255, 18);

    pub const WALL_FILL: Color = Color::from_rgba(90, 95, 110, 255);
    pub const WALL_OUTLINE: Color = Color::from_rgba(130, 135, 150, 255);

    pub const ROCK_FILL: Color = Color::from_rgba(110, 100, 85, 255);
    pub const ROCK_OUTLINE: Color = Color::from_rgba(70, 62, 52, 255);

    pub const TREAD_BASE: Color = Color::from_rgba(30, 38, 30, 255);
    pub const TREAD_CLEAT: Color = Color::from_rgba(58, 70, 58, 255);
    pub const HULL_FILL: Color = Color::from_rgba(70, 120, 60, 255);
    pub const HULL_FRONT_STRIPE: Color = Color::from_rgba(115, 175, 95, 255);
    pub const BARREL: Color = Color::from_rgba(40, 70, 35, 255);
    pub const TURRET: Color = Color::from_rgba(95, 150, 80, 255);

    pub const BULLET: Color = Color::from_rgba(255, 220, 120, 255);
    pub const CROSSHAIR: Color = Color::from_rgba(255, 255, 255, 180);
    pub const HUD_BG: Color = Color::from_rgba(0, 0, 0, 140);
}

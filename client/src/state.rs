//! Game entities, world layout, and local simulation state.

use crate::config::{arena, bullet, tank};
use crate::input::PlayerCommand;
use crate::physics::{resolve_circle_circle, resolve_circle_rect, shortest_angle_diff};
use macroquad::prelude::*;
use std::f32::consts::TAU;

/// Circular static obstacle.
#[derive(Clone, Copy, Debug)]
pub struct Rock {
    pub pos: Vec2,
    pub radius: f32,
}

/// Static arena geometry (walls and rocks).
pub struct WorldMap {
    pub walls: Vec<Rect>,
    pub rocks: Vec<Rock>,
}

impl WorldMap {
    /// Generates the deterministic arena layout around `spawn`.
    #[must_use]
    pub fn generate(spawn: Vec2) -> Self {
        let (w, h, t) = (arena::SIZE.x, arena::SIZE.y, arena::BORDER_THICKNESS);

        let walls = vec![
            // Outer arena boundary
            Rect::new(0.0, 0.0, w, t),
            Rect::new(0.0, h - t, w, t),
            Rect::new(0.0, 0.0, t, h),
            Rect::new(w - t, 0.0, t, h),
            // Interior cover walls
            Rect::new(500.0, 300.0, 400.0, 30.0),
            Rect::new(500.0, 300.0, 30.0, 350.0),
            Rect::new(1200.0, 700.0, 30.0, 600.0),
            Rect::new(1400.0, 400.0, 500.0, 30.0),
            Rect::new(2100.0, 900.0, 30.0, 500.0),
            Rect::new(2100.0, 1400.0, 450.0, 30.0),
            Rect::new(700.0, 1500.0, 600.0, 30.0),
            Rect::new(2400.0, 300.0, 30.0, 350.0),
            // Center sector divider wall with pass-through gap
            Rect::new(1485.0, 1100.0, 30.0, 860.0),
        ];

        let mut map = Self {
            walls,
            rocks: Vec::with_capacity(arena::ROCK_COUNT),
        };

        rand::srand(arena::RNG_SEED);
        let (min_r, max_r) = arena::ROCK_RADIUS_RANGE;
        let clearance = tank::COLLISION_RADIUS * 2.0;

        for _ in 0..2000 {
            if map.rocks.len() >= arena::ROCK_COUNT {
                break;
            }
            let radius = rand::gen_range(min_r, max_r);
            let pos = vec2(
                rand::gen_range(t + radius, w - t - radius),
                rand::gen_range(t + radius, h - t - radius),
            );

            let near_spawn = pos.distance(spawn) < arena::SPAWN_SAFE_RADIUS;
            if !near_spawn && !map.intersects_circle(pos, radius + clearance) {
                map.rocks.push(Rock { pos, radius });
            }
        }

        map
    }

    /// Returns `true` if a circle at `pos` with `radius` overlaps any wall or rock.
    #[must_use]
    pub fn intersects_circle(&self, pos: Vec2, radius: f32) -> bool {
        let mut probe = pos;
        self.walls
            .iter()
            .any(|w| resolve_circle_rect(&mut probe, radius, w))
            || self
                .rocks
                .iter()
                .any(|r| pos.distance_squared(r.pos) < (radius + r.radius).powi(2))
    }

    /// Resolves collisions for a dynamic circle against all static obstacles.
    pub fn resolve_circle_collisions(&self, pos: &mut Vec2, radius: f32) {
        for _ in 0..3 {
            let mut hit = false;
            for wall in &self.walls {
                hit |= resolve_circle_rect(pos, radius, wall);
            }
            for rock in &self.rocks {
                hit |= resolve_circle_circle(pos, radius, rock.pos, rock.radius);
            }
            if !hit {
                break;
            }
        }
    }

    #[must_use]
    pub fn total_obstacles(&self) -> usize {
        self.walls.len() + self.rocks.len()
    }
}

/// Player tank using non-holonomic bidirectional tread kinematics.
///
/// Treads never scrub sideways: linear velocity is always strictly aligned with
/// `hull_angle`, and opposing inputs engage reverse gear rather than spinning 180°.
#[derive(Clone, Debug)]
pub struct Tank {
    pub pos: Vec2,
    pub hull_angle: f32,
    pub aim_angle: f32,
    pub linear_speed: f32,
    pub reversing: bool,
    pub left_tread_phase: f32,
    pub right_tread_phase: f32,
    fire_cooldown: f32,
}

impl Tank {
    #[must_use]
    pub fn new(spawn: Vec2) -> Self {
        Self {
            pos: spawn,
            hull_angle: 0.0,
            aim_angle: 0.0,
            linear_speed: 0.0,
            reversing: false,
            left_tread_phase: 0.0,
            right_tread_phase: 0.0,
            fire_cooldown: 0.0,
        }
    }

    /// Updates tank steering, tread-aligned movement, collisions, and turret aim.
    pub fn update(&mut self, move_dir: Vec2, aim_world: Vec2, map: &WorldMap, dt: f32) {
        let mut turn_step = 0.0;

        let target_speed = if move_dir.length_squared() > 1e-4 {
            let dir = move_dir.normalize();
            let current_fwd = Vec2::from_angle(self.hull_angle);
            let forward_dot = current_fwd.dot(dir);

            // Hysteresis prevents rapid gear toggling when steering near 90 degrees.
            let reverse_threshold = if self.reversing {
                tank::GEAR_HYSTERESIS
            } else {
                -tank::GEAR_HYSTERESIS
            };
            self.reversing = forward_dot < reverse_threshold;

            // Steer the active end (front in forward gear, rear in reverse gear) toward `dir`.
            let desired_hull_heading = if self.reversing {
                (-dir).to_angle()
            } else {
                dir.to_angle()
            };
            let turn_error = shortest_angle_diff(self.hull_angle, desired_hull_heading);

            let max_turn = tank::HULL_TURN_SPEED * dt;
            turn_step = turn_error.clamp(-max_turn, max_turn);
            self.hull_angle = (self.hull_angle + turn_step).rem_euclid(TAU);

            // Scale linear speed by cos^2(remaining_error) so the tank pivots on its
            // treads during sharp turns instead of drifting sideways.
            let remaining_error = shortest_angle_diff(self.hull_angle, desired_hull_heading);
            let alignment = remaining_error.cos().max(0.0).powi(2);

            if self.reversing {
                -tank::MAX_REVERSE_SPEED * alignment
            } else {
                tank::MAX_FORWARD_SPEED * alignment
            }
        } else {
            0.0
        };

        // Smooth acceleration / braking along the treads.
        let rate = if target_speed.abs() > self.linear_speed.abs() {
            tank::ACCELERATION
        } else {
            tank::DECELERATION
        };
        let speed_diff = target_speed - self.linear_speed;
        self.linear_speed += speed_diff.clamp(-rate * dt, rate * dt);

        // Displace strictly along the hull's forward vector (zero lateral slip).
        let forward = Vec2::from_angle(self.hull_angle);
        let prev_pos = self.pos;
        self.pos += forward * self.linear_speed * dt;
        map.resolve_circle_collisions(&mut self.pos, tank::COLLISION_RADIUS);

        // Advance left and right tread cleats from actual forward travel + differential rotation.
        let actual_forward_travel = (self.pos - prev_pos).dot(forward);
        let rotational_arc = turn_step * tank::TREAD_LATERAL_OFFSET;
        self.left_tread_phase = (self.left_tread_phase + actual_forward_travel + rotational_arc)
            .rem_euclid(tank::TREAD_CLEAT_SPACING);
        self.right_tread_phase = (self.right_tread_phase + actual_forward_travel - rotational_arc)
            .rem_euclid(tank::TREAD_CLEAT_SPACING);

        // Turret aiming in world coordinates.
        let to_aim = aim_world - self.pos;
        if to_aim.length_squared() > 1.0 {
            self.aim_angle = to_aim.to_angle();
        }

        self.fire_cooldown = (self.fire_cooldown - dt).max(0.0);
    }

    /// Attempts to spawn a projectile if the cannon cooldown has elapsed.
    pub fn try_fire(&mut self) -> Option<Bullet> {
        if self.fire_cooldown > 0.0 {
            return None;
        }
        self.fire_cooldown = bullet::FIRE_COOLDOWN;
        let dir = Vec2::from_angle(self.aim_angle);
        Some(Bullet {
            pos: self.pos + dir * (tank::BARREL_LENGTH + bullet::RADIUS),
            vel: dir * bullet::SPEED,
            ttl: bullet::LIFETIME,
        })
    }
}

/// Active projectile in flight.
#[derive(Clone, Copy, Debug)]
pub struct Bullet {
    pub pos: Vec2,
    pub vel: Vec2,
    pub ttl: f32,
}

/// Short-lived visual flash when a projectile strikes an obstacle.
#[derive(Clone, Copy, Debug)]
pub struct Impact {
    pub pos: Vec2,
    pub ttl: f32,
}

/// Complete client-side game state.
pub struct GameState {
    pub map: WorldMap,
    pub player: Tank,
    pub bullets: Vec<Bullet>,
    pub impacts: Vec<Impact>,
}

impl GameState {
    #[must_use]
    pub fn new() -> Self {
        let spawn = arena::SPAWN_POS;
        Self {
            map: WorldMap::generate(spawn),
            player: Tank::new(spawn),
            bullets: Vec::new(),
            impacts: Vec::new(),
        }
    }

    /// Advances the local simulation by `dt` seconds.
    pub fn step(&mut self, cmd: &PlayerCommand, dt: f32) {
        self.player
            .update(cmd.frame.move_dir, cmd.aim_world, &self.map, dt);

        if cmd.frame.shoot
            && let Some(proj) = self.player.try_fire()
        {
            self.bullets.push(proj);
        }

        self.step_bullets(dt);
    }

    fn step_bullets(&mut self, dt: f32) {
        let map = &self.map;
        let impacts = &mut self.impacts;

        self.bullets.retain_mut(|b| {
            b.ttl -= dt;
            if b.ttl <= 0.0 {
                return false;
            }

            // Sub-step high-speed projectiles so they cannot tunnel through thin walls.
            let travel = b.vel * dt;
            let steps = (travel.length() / bullet::RADIUS).ceil().max(1.0) as usize;
            let step_vec = travel / steps as f32;

            for _ in 0..steps {
                b.pos += step_vec;
                if map.intersects_circle(b.pos, bullet::RADIUS) {
                    impacts.push(Impact {
                        pos: b.pos,
                        ttl: bullet::IMPACT_LIFETIME,
                    });
                    return false;
                }
            }
            true
        });

        self.impacts.retain_mut(|i| {
            i.ttl -= dt;
            i.ttl > 0.0
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_map() -> WorldMap {
        WorldMap {
            walls: Vec::new(),
            rocks: Vec::new(),
        }
    }

    #[test]
    fn opposite_input_reverses_without_180_hull_spin() {
        let map = empty_map();
        let mut tank = Tank::new(vec2(500.0, 500.0));
        tank.hull_angle = 0.0; // Facing East (+X)

        // Press West (-X): tank should shift into reverse and keep hull facing ~East (0 rad).
        for _ in 0..10 {
            tank.update(vec2(-1.0, 0.0), vec2(600.0, 500.0), &map, 1.0 / 60.0);
        }

        assert!(tank.reversing);
        assert!(tank.pos.x < 500.0, "tank should move West in reverse");
        assert!(
            shortest_angle_diff(tank.hull_angle, 0.0).abs() < 1e-4,
            "hull should not spin 180 degrees when reversing straight back"
        );
    }

    #[test]
    fn perpendicular_input_never_scrubs_sideways() {
        let map = empty_map();
        let mut tank = Tank::new(vec2(500.0, 500.0));
        tank.hull_angle = 0.0; // Facing East (+X)

        let prev_pos = tank.pos;
        // Input straight South (+Y) for a single small step:
        tank.update(vec2(0.0, 1.0), vec2(600.0, 500.0), &map, 1.0 / 60.0);

        let displacement = tank.pos - prev_pos;
        let lateral_axis = Vec2::from_angle(tank.hull_angle).perp();
        let lateral_slip = displacement.dot(lateral_axis).abs();

        assert!(
            lateral_slip < 1e-4,
            "displacement must be strictly aligned with tread direction (lateral slip = {lateral_slip})"
        );
    }
}

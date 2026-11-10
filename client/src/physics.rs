//! Pure 2D geometric and collision helpers.
//!
//! Kept free of rendering and I/O so it can be moved directly into the
//! `simulation` crate in Issue #4.

use macroquad::prelude::{Rect, Vec2, vec2};
use std::f32::consts::{PI, TAU};

/// Shortest signed angular difference from `from` to `to`, wrapped to `(-PI, PI]`.
#[must_use]
pub fn shortest_angle_diff(from: f32, to: f32) -> f32 {
    (to - from + PI).rem_euclid(TAU) - PI
}

/// Pushes a circle (`pos`, `radius`) out of an axis-aligned `rect`.
/// Returns `true` if they overlapped.
pub fn resolve_circle_rect(pos: &mut Vec2, radius: f32, rect: &Rect) -> bool {
    let closest = vec2(
        pos.x.clamp(rect.x, rect.right()),
        pos.y.clamp(rect.y, rect.bottom()),
    );
    let delta = *pos - closest;
    let dist_sq = delta.length_squared();
    if dist_sq >= radius * radius {
        return false;
    }

    if dist_sq > 1e-6 {
        // Center is outside the rect: push along the contact normal for smooth wall sliding.
        *pos = closest + delta / dist_sq.sqrt() * radius;
    } else {
        // Center is inside the rect: push out along the axis of minimum penetration.
        let left = pos.x - rect.x;
        let right = rect.right() - pos.x;
        let top = pos.y - rect.y;
        let bottom = rect.bottom() - pos.y;
        let min = left.min(right).min(top).min(bottom);

        if min == left {
            pos.x = rect.x - radius;
        } else if min == right {
            pos.x = rect.right() + radius;
        } else if min == top {
            pos.y = rect.y - radius;
        } else {
            pos.y = rect.bottom() + radius;
        }
    }
    true
}

/// Pushes a circle (`pos`, `radius`) out of a static circle (`other_pos`, `other_radius`).
/// Returns `true` if they overlapped.
pub fn resolve_circle_circle(
    pos: &mut Vec2,
    radius: f32,
    other_pos: Vec2,
    other_radius: f32,
) -> bool {
    let delta = *pos - other_pos;
    let min_dist = radius + other_radius;
    let dist_sq = delta.length_squared();
    if dist_sq >= min_dist * min_dist {
        return false;
    }

    let normal = if dist_sq > 1e-6 {
        delta / dist_sq.sqrt()
    } else {
        Vec2::X
    };
    *pos = other_pos + normal * min_dist;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angle_diff_wraps_shortest_way() {
        let diff = shortest_angle_diff(0.1, TAU - 0.1);
        assert!((diff - (-0.2)).abs() < 1e-5);

        let diff_rev = shortest_angle_diff(TAU - 0.1, 0.1);
        assert!((diff_rev - 0.2).abs() < 1e-5);
    }

    #[test]
    fn circle_rect_pushes_out_of_wall() {
        let wall = Rect::new(100.0, 100.0, 50.0, 50.0);
        let mut pos = vec2(90.0, 125.0);
        assert!(resolve_circle_rect(&mut pos, 20.0, &wall));
        assert!((pos.x - 80.0).abs() < 1e-4);
    }

    #[test]
    fn circle_circle_separates_overlap() {
        let mut pos = vec2(10.0, 0.0);
        assert!(resolve_circle_circle(&mut pos, 10.0, Vec2::ZERO, 10.0));
        assert!((pos.x - 20.0).abs() < 1e-4);
    }
}

//! Stateless rendering functions for world entities and HUD overlay.

use crate::camera::ViewportCamera;
use crate::config::{arena, bullet, palette, tank};
use crate::state::{GameState, Tank, WorldMap};
use macroquad::prelude::*;

/// Renders the entire frame (world space followed by screen-space HUD).
pub fn render_frame(state: &GameState, camera: &ViewportCamera, aim_world: Vec2) {
    clear_background(palette::BACKGROUND);

    // --- World Space ---
    camera.apply();
    let view = camera.visible_rect();

    draw_grid(&view);
    let drawn_obstacles = draw_obstacles(&state.map, camera);
    draw_bullets_and_impacts(state, camera);
    draw_tank(&state.player);
    draw_crosshair(aim_world);

    // --- Screen Space (HUD) ---
    set_default_camera();
    draw_hud(state, drawn_obstacles);
}

fn draw_grid(view: &Rect) {
    let spacing = arena::GRID_SPACING;
    let x_start = (view.x / spacing).floor() * spacing;
    let y_start = (view.y / spacing).floor() * spacing;

    let mut x = x_start;
    while x <= view.right() {
        draw_line(x, view.y, x, view.bottom(), 1.0, palette::GRID_LINE);
        x += spacing;
    }

    let mut y = y_start;
    while y <= view.bottom() {
        draw_line(view.x, y, view.right(), y, 1.0, palette::GRID_LINE);
        y += spacing;
    }
}

fn draw_obstacles(map: &WorldMap, camera: &ViewportCamera) -> usize {
    let view = camera.visible_rect();
    let mut drawn = 0;

    for wall in &map.walls {
        if wall.overlaps(&view) {
            draw_rectangle(wall.x, wall.y, wall.w, wall.h, palette::WALL_FILL);
            draw_rectangle_lines(wall.x, wall.y, wall.w, wall.h, 2.0, palette::WALL_OUTLINE);
            drawn += 1;
        }
    }

    for rock in &map.rocks {
        if camera.is_circle_visible(rock.pos, rock.radius) {
            draw_circle(rock.pos.x, rock.pos.y, rock.radius, palette::ROCK_FILL);
            draw_circle_lines(
                rock.pos.x,
                rock.pos.y,
                rock.radius,
                2.0,
                palette::ROCK_OUTLINE,
            );
            drawn += 1;
        }
    }

    drawn
}

fn draw_bullets_and_impacts(state: &GameState, camera: &ViewportCamera) {
    for b in &state.bullets {
        if camera.is_circle_visible(b.pos, bullet::RADIUS) {
            draw_circle(b.pos.x, b.pos.y, bullet::RADIUS, palette::BULLET);
        }
    }

    for impact in &state.impacts {
        let life_ratio = impact.ttl / bullet::IMPACT_LIFETIME;
        let radius = 6.0 + (1.0 - life_ratio) * 14.0;
        let color = Color::new(1.0, 0.6, 0.2, life_ratio * 0.8);
        draw_circle(impact.pos.x, impact.pos.y, radius, color);
    }
}

fn draw_tank(tank_state: &Tank) {
    let forward = Vec2::from_angle(tank_state.hull_angle);
    let right = forward.perp();

    // Left and right tracks with scrolling tread cleats
    let phases = [
        (-1.0, tank_state.left_tread_phase),
        (1.0, tank_state.right_tread_phase),
    ];
    let half_tread_len = tank::TREAD_SIZE.x * 0.5;
    let half_tread_width = tank::TREAD_SIZE.y * 0.5 - 1.0;

    for (side_sign, phase) in phases {
        let track_center = tank_state.pos + right * (tank::TREAD_LATERAL_OFFSET * side_sign);
        draw_rectangle_ex(
            track_center.x,
            track_center.y,
            tank::TREAD_SIZE.x,
            tank::TREAD_SIZE.y,
            DrawRectangleParams {
                offset: vec2(0.5, 0.5),
                rotation: tank_state.hull_angle,
                color: palette::TREAD_BASE,
            },
        );

        // Draw animated cleats along the length of the track
        let mut offset = -half_tread_len + phase;
        while offset <= half_tread_len {
            let cleat_center = track_center + forward * offset;
            let p1 = cleat_center - right * half_tread_width;
            let p2 = cleat_center + right * half_tread_width;
            draw_line(p1.x, p1.y, p2.x, p2.y, 2.0, palette::TREAD_CLEAT);
            offset += tank::TREAD_CLEAT_SPACING;
        }
    }

    // Main armored hull
    draw_rectangle_ex(
        tank_state.pos.x,
        tank_state.pos.y,
        tank::HULL_SIZE.x,
        tank::HULL_SIZE.y,
        DrawRectangleParams {
            offset: vec2(0.5, 0.5),
            rotation: tank_state.hull_angle,
            color: palette::HULL_FILL,
        },
    );

    // Front glacis stripe so the hull's front orientation is unmistakable
    let front_marker = tank_state.pos + forward * (tank::HULL_SIZE.x * 0.36);
    draw_rectangle_ex(
        front_marker.x,
        front_marker.y,
        5.0,
        tank::HULL_SIZE.y - 6.0,
        DrawRectangleParams {
            offset: vec2(0.5, 0.5),
            rotation: tank_state.hull_angle,
            color: palette::HULL_FRONT_STRIPE,
        },
    );

    // Turret barrel & dome
    let barrel_end = tank_state.pos + Vec2::from_angle(tank_state.aim_angle) * tank::BARREL_LENGTH;
    draw_line(
        tank_state.pos.x,
        tank_state.pos.y,
        barrel_end.x,
        barrel_end.y,
        tank::BARREL_WIDTH,
        palette::BARREL,
    );
    draw_circle(
        tank_state.pos.x,
        tank_state.pos.y,
        tank::TURRET_RADIUS,
        palette::TURRET,
    );
}

fn draw_crosshair(pos: Vec2) {
    draw_circle_lines(pos.x, pos.y, 10.0, 1.5, palette::CROSSHAIR);
    draw_line(
        pos.x - 15.0,
        pos.y,
        pos.x - 5.0,
        pos.y,
        1.5,
        palette::CROSSHAIR,
    );
    draw_line(
        pos.x + 5.0,
        pos.y,
        pos.x + 15.0,
        pos.y,
        1.5,
        palette::CROSSHAIR,
    );
    draw_line(
        pos.x,
        pos.y - 15.0,
        pos.x,
        pos.y - 5.0,
        1.5,
        palette::CROSSHAIR,
    );
    draw_line(
        pos.x,
        pos.y + 5.0,
        pos.x,
        pos.y + 15.0,
        1.5,
        palette::CROSSHAIR,
    );
}

fn draw_hud(state: &GameState, drawn_obstacles: usize) {
    let gear = if state.player.reversing { "REV" } else { "FWD" };
    let lines = [
        format!("FPS: {}", get_fps()),
        format!(
            "Tank: ({:.0}, {:.0}) | Speed: {:.0} ({gear})",
            state.player.pos.x,
            state.player.pos.y,
            state.player.linear_speed.abs()
        ),
        format!("Bullets alive: {}", state.bullets.len()),
        format!(
            "Obstacles in viewport: {drawn_obstacles} / {}",
            state.map.total_obstacles()
        ),
        "WASD/Arrows: drive | Mouse: aim | LMB/Space: fire | Esc: quit".to_owned(),
    ];

    let box_height = 22.0 * lines.len() as f32 + 10.0;
    draw_rectangle(8.0, 8.0, 600.0, box_height, palette::HUD_BG);

    for (i, line) in lines.iter().enumerate() {
        draw_text(line, 16.0, 30.0 + 22.0 * i as f32, 22.0, WHITE);
    }
}

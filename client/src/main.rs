//! MP Tanks client entry point.

mod camera;
mod config;
mod input;
mod physics;
mod render;
mod state;

use crate::camera::ViewportCamera;
use crate::config::{arena, view};
use crate::input::InputSampler;
use crate::state::GameState;
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "MP Tanks".to_owned(),
        window_width: view::WINDOW_WIDTH,
        window_height: view::WINDOW_HEIGHT,
        window_resizable: true,
        sample_count: 4,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut state = GameState::new();
    let mut camera = ViewportCamera::new(arena::SPAWN_POS);
    let mut input = InputSampler::new();

    loop {
        let dt = get_frame_time().min(view::MAX_DT);

        let cmd = input.sample(&camera);
        if cmd.quit_requested {
            break;
        }

        // In Issue #3, `cmd.frame` will also be sent over UDP and `state` will
        // reconcile against incoming `protocol::ServerSnapshot`s.
        state.step(&cmd, dt);
        camera.follow(state.player.pos, dt);

        render::render_frame(&state, &camera, cmd.aim_world);

        next_frame().await;
    }
}

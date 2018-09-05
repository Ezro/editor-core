//! TODO: Rewrite for new renderer.

extern crate amethyst;
extern crate env_logger;
extern crate amethyst_editor_sync;
#[macro_use]
extern crate serde;

mod audio;
mod bundle;
mod pong;
mod systems;

use amethyst::core::Transform;
use amethyst::audio::AudioBundle;
use amethyst::core::frame_limiter::FrameRateLimitStrategy;
use amethyst::core::transform::TransformBundle;
use amethyst::ecs::prelude::{Component, DenseVecStorage};
use amethyst::input::InputBundle;
use amethyst::prelude::*;
use amethyst::renderer::{DisplayConfig, DrawSprite, Pipeline, RenderBundle, Stage};
use amethyst::ui::{DrawUi, UiBundle};

use audio::Music;
use bundle::PongBundle;
use std::str;
use std::time::Duration;
use amethyst_editor_sync::*;

const ARENA_HEIGHT: f32 = 100.0;
const ARENA_WIDTH: f32 = 100.0;
const PADDLE_HEIGHT: f32 = 16.0;
const PADDLE_WIDTH: f32 = 4.0;
const PADDLE_VELOCITY: f32 = 75.0;

const BALL_VELOCITY_X: f32 = 75.0;
const BALL_VELOCITY_Y: f32 = 50.0;
const BALL_RADIUS: f32 = 2.0;

const SPRITESHEET_SIZE: (f32, f32) = (8.0, 16.0);

const AUDIO_MUSIC: &'static [&'static str] = &[
    "audio/Computer_Music_All-Stars_-_Wheres_My_Jetpack.ogg",
    "audio/Computer_Music_All-Stars_-_Albatross_v2.ogg",
];
const AUDIO_BOUNCE: &'static str = "audio/bounce.ogg";
const AUDIO_SCORE: &'static str = "audio/score.ogg";

fn main() -> amethyst::Result<()> {
    env_logger::init();

    use pong::Pong;

    let display_config_path = format!(
        "{}/examples/pong/resources/display.ron",
        env!("CARGO_MANIFEST_DIR")
    );
    let config = DisplayConfig::load(&display_config_path);

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawSprite::new())
            .with_pass(DrawUi::new()),
    );

    let key_bindings_path = {
        if cfg!(feature = "sdl_controller") {
            format!(
                "{}/examples/pong/resources/input_controller.ron",
                env!("CARGO_MANIFEST_DIR")
            )
        } else {
            format!(
                "{}/examples/pong/resources/input.ron",
                env!("CARGO_MANIFEST_DIR")
            )
        }
    };

    let assets_dir = format!("{}/examples/assets/", env!("CARGO_MANIFEST_DIR"));

    let editor_system = SyncEditorSystem::new();

    let game_data = GameDataBuilder::default()
        .with_bundle(
            InputBundle::<String, String>::new().with_bindings_from_file(&key_bindings_path)?,
        )?
        .with_bundle(PongBundle)?
        .with_bundle(RenderBundle::new(pipe, Some(config)).with_sprite_sheet_processor())?
        .with_bundle(TransformBundle::new().with_dep(&["ball_system", "paddle_system"]))?
        .with_bundle(AudioBundle::new(|music: &mut Music| music.music.next()))?
        .with_bundle(UiBundle::<String, String>::new())?
        .with_barrier()
        .with(SyncComponentSystem::<Transform>::new("Transform", &editor_system), "editor_transform", &[])
        .with(SyncComponentSystem::<Ball>::new("Ball", &editor_system), "editor_ball", &[])
        .with(SyncComponentSystem::<Paddle>::new("Paddle", &editor_system), "editor_paddle", &[])
        .with(SyncResourceSystem::<ScoreBoard>::new("ScoreBoard", &editor_system), "editor_score_board", &[])
        .with_thread_local(editor_system);
    let mut game = Application::build(assets_dir, Pong)?
        .with_frame_limit(
            FrameRateLimitStrategy::SleepAndYield(Duration::from_millis(2)),
            144,
        )
        .build(game_data)?;
    game.run();
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ball {
    pub velocity: [f32; 2],
    pub radius: f32,
}

impl Component for Ball {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Left,
    Right,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Paddle {
    pub velocity: f32,
    pub side: Side,
    pub width: f32,
    pub height: f32,
}

impl Paddle {
    pub fn new(side: Side) -> Paddle {
        Paddle {
            velocity: 1.0,
            side: side,
            width: 1.0,
            height: 1.0,
        }
    }
}

impl Component for Paddle {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ScoreBoard {
    score_left: i32,
    score_right: i32,
}

impl ScoreBoard {
    pub fn new() -> ScoreBoard {
        ScoreBoard {
            score_left: 0,
            score_right: 0,
        }
    }
}
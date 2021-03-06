use ggez::event;
use ggez::graphics;
use ggez::input::keyboard;
use ggez::timer;

use crate::components;
use crate::entities;
use crate::resources;
use crate::systems;

pub const MAP_WIDTH: u8 = 9;
pub const MAP_HEIGHT: u8 = 9;

pub const TILE_WIDTH: f32 = 48.0;
pub const TILE_HEIGHT: f32 = 48.0;

pub const ARENA_WIDTH: f32 = 720.0;
pub const ARENA_HEIGHT: f32 = MAP_HEIGHT as f32 * TILE_HEIGHT;

const FPS: u32 = 60;

const SOUNDS: &[&str] = &[
    "/sounds/wall.wav",
    "/sounds/correct.wav",
    "/sounds/incorrect.wav",
];

const IMAGES: &[&str] = &[
    "/images/box_blue_1.png",
    "/images/box_blue_2.png",
    "/images/box_red_1.png",
    "/images/box_red_2.png",
    "/images/box_spot_blue.png",
    "/images/box_spot_red.png",
    "/images/floor.png",
    "/images/player_1.png",
    "/images/player_2.png",
    "/images/player_3.png",
    "/images/wall.png",
];

pub struct Game {
    world: legion::World,
    resources: legion::Resources,
    schedule: legion::Schedule,
}

impl Game {
    pub fn new(ctx: &mut ggez::Context, map_str: &str) -> ggez::GameResult<Self> {
        // Load game's images into memory.
        let mut audio_store = resources::AudioStore::default();
        load_sounds(ctx, &mut audio_store, SOUNDS)?;

        // Load game's sound effects into memory.
        let mut drawable_store = resources::DrawableStore::default();
        load_images(ctx, &mut drawable_store, IMAGES)?;

        // Load game's map and create the entity as specified by the map.
        let mut world = legion::World::default();
        let map = parse_map(map_str);
        entities::create_entities_from_map(&mut world, map)?;

        // Initialize shared resources.
        let mut resources = legion::Resources::default();
        resources.insert(resources::Time::default());
        resources.insert(resources::GamePlay::default());
        resources.insert(resources::KeyPressedEventQueue::default());
        resources.insert(resources::GamePlayEventQueue::default());
        resources.insert(audio_store);
        resources.insert(drawable_store);

        let schedule = legion::Schedule::builder()
            .add_system(systems::input_handling_system())
            .add_system(systems::game_objective_system())
            .add_system(systems::consume_gameplay_events_system())
            .build();

        Ok(Self {
            world,
            resources,
            schedule,
        })
    }
}

impl event::EventHandler for Game {
    fn update(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        while timer::check_update_time(ctx, FPS) {
            if let Some(mut time) = self.resources.get_mut::<resources::Time>() {
                time.alive += timer::delta(ctx);
            }
            self.schedule.execute(&mut self.world, &mut self.resources);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        graphics::clear(ctx, graphics::WHITE);
        systems::render_entities(ctx, &self.world, &self.resources)?;
        systems::render_gameplay_data(ctx, &self.resources)?;
        graphics::present(ctx)
    }

    fn key_down_event(
        &mut self,
        ctx: &mut ggez::Context,
        keycode: keyboard::KeyCode,
        _keymods: keyboard::KeyMods,
        _repeat: bool,
    ) {
        if keycode == keyboard::KeyCode::Escape {
            event::quit(ctx);
        }

        let key_pressed_events = self.resources.get_mut::<resources::KeyPressedEventQueue>();
        if let Some(mut key_pressed_events) = key_pressed_events {
            key_pressed_events.queue.push(keycode);
        };
    }
}

fn load_sounds(
    ctx: &mut ggez::Context,
    audio_store: &mut resources::AudioStore,
    sounds: &[&str],
) -> ggez::GameResult {
    for sound_path in sounds.iter() {
        audio_store.add_sound(ctx, sound_path)?;
    }
    Ok(())
}

fn load_images(
    ctx: &mut ggez::Context,
    drawable_store: &mut resources::DrawableStore,
    images: &[&str],
) -> ggez::GameResult {
    for image_path in images.iter() {
        drawable_store.add_image(ctx, image_path, graphics::FilterMode::Nearest)?;
    }
    Ok(())
}

fn parse_map(map_str: &str) -> Vec<(components::Position, &str)> {
    map_str
        .split('\n')
        .enumerate()
        .flat_map(|(y, row)| {
            row.trim().split(' ').enumerate().map(move |(x, val)| {
                (
                    components::Position {
                        x: x as u8,
                        y: y as u8,
                        z: 0,
                    },
                    val,
                )
            })
        })
        .collect()
}

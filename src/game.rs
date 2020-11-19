use ggez::event;
use ggez::graphics::{self, Drawable};
use ggez::input::keyboard;
use ggez::mint;
use legion::query::IntoQuery;

use crate::components;
use crate::entities;
use crate::resources;
use crate::systems;

pub const TILE_WIDTH: f32 = 48.0;
pub const TILE_HEIGHT: f32 = 48.0;

/// This structure holds access to the game's `World` and implements `EventHandler` to updates and
/// render entities on each game tick
pub struct Game {
    world: legion::World,
    resources: legion::Resources,
    update_schedule: legion::Schedule,
}

impl Game {
    /// Create a new game state based on the given context and initializes entities based on the
    /// given map represented in string
    pub fn new(ctx: &mut ggez::Context, map_str: &str) -> ggez::GameResult<Self> {
        let mut world = legion::World::default();
        load_map(ctx, &mut world, map_str)?;

        let mut resources = legion::Resources::default();
        resources.insert(resources::KeyBoardEventQueue::default());

        let update_schedule = legion::Schedule::builder()
            .add_system(systems::player_control_system())
            .build();

        Ok(Self {
            world,
            resources,
            update_schedule,
        })
    }
}

impl event::EventHandler for Game {
    /// This method is run on each game tick to update the world's data
    fn update(&mut self, _ctx: &mut ggez::Context) -> ggez::GameResult {
        self.update_schedule
            .execute(&mut self.world, &mut self.resources);
        Ok(())
    }

    /// This method is run on each game tick to render the entities to screen
    /// based on the world's data
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        graphics::clear(ctx, graphics::WHITE);

        // Go through the entities that can be rendered to screen and get their data
        let mut renderables_query = <(&components::Position, &components::Renderable)>::query();
        let mut renderables_data = renderables_query
            .iter(&self.world)
            .collect::<Vec<(&components::Position, &components::Renderable)>>();
        renderables_data.sort_by_key(|&k| k.0.z);

        for (position, renderable) in renderables_data {
            let screen_dest = mint::Point2 {
                x: position.x as f32 * TILE_WIDTH,
                y: position.y as f32 * TILE_HEIGHT,
            };

            let mut draw_params = graphics::DrawParam::default().dest(screen_dest);
            if let Some(renderable_dims) = renderable.dimensions(ctx) {
                draw_params = draw_params.scale(mint::Vector2 {
                    x: TILE_WIDTH / renderable_dims.w,
                    y: TILE_HEIGHT / renderable_dims.h,
                });
            }

            graphics::draw(ctx, renderable, draw_params)?;
        }

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

        if let Some(mut keyboard_events) = self.resources.get_mut::<resources::KeyBoardEventQueue>()
        {
            keyboard_events.keys_pressed.push(keycode);
        };
    }
}

fn load_map(ctx: &mut ggez::Context, world: &mut legion::World, map_str: &str) -> ggez::GameResult {
    let rows: Vec<&str> = map_str.split('\n').map(|x| x.trim()).collect();
    for (y, row) in rows.iter().enumerate() {
        let cols: Vec<&str> = row.split(' ').collect();
        for (x, col) in cols.iter().enumerate() {
            let position = components::Position {
                x: x as u8,
                y: y as u8,
                z: 0, // this will be modified when the entity is created
            };

            match *col {
                "P" => {
                    let _ = entities::create_floor(ctx, world, position)?;
                    let _ = entities::create_player(ctx, world, position)?;
                }
                "B" => {
                    let _ = entities::create_floor(ctx, world, position)?;
                    let _ = entities::create_box(ctx, world, position)?;
                }
                "W" => {
                    let _ = entities::create_wall(ctx, world, position)?;
                }
                "S" => {
                    let _ = entities::create_floor(ctx, world, position)?;
                    let _ = entities::create_box_spot(ctx, world, position)?;
                }
                "." => {
                    let _ = entities::create_floor(ctx, world, position)?;
                }
                "N" => {}
                c => panic!("Invalid map item {}", c),
            }
        }
    }
    Ok(())
}
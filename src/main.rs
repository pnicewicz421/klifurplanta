use bevy::prelude::*;

mod components;
mod systems;
mod resources;
mod states;
mod levels;

use components::*;
use systems::*;
use resources::*;
use states::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
                .add_systems(Startup, (setup, load_terrain_from_level))
        .add_systems(
            Update,
            (
                // Phase 2+ systems with health & stamina
                player_movement_system,
                stamina_regeneration_system,
                health_stamina_display_system,
                camera_follow_system,
                update_time,
            ).run_if(in_state(GameState::Climbing)),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Spawn camera
    commands.spawn(Camera2dBundle::default());
    
    // Initialize basic game resources
    commands.insert_resource(GameTime::new());
    
    // Spawn player for Phase 2 with Health & Stamina
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.2, 0.7, 0.2), // Green climber
                custom_size: Some(Vec2::new(30.0, 50.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
            ..default()
        },
        Player { id: 1 },
        Health { current: 100.0, max: 100.0 },
        MovementStats { 
            speed: 200.0,
            climbing_skill: 1.0,
            stamina: 100.0,
            max_stamina: 100.0,
        },
    ));
    
    // Start in climbing state for Phase 1
    next_state.set(GameState::Climbing);
    
    info!("🎮 Health & Stamina Active! Watch console: Move=drain stamina, Rest=regenerate, Ice/Snow=damage!");
    info!("📊 Health/Stamina display updates every 2 seconds. Move around to see effects!");
}

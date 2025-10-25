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
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                // Core systems that run in multiple states
                time_system,
                input_system,
            ).run_if(not(in_state(GameState::Menu))),
        )
        .add_systems(
            Update,
            (
                shop_system,
                inventory_ui_system,
            ).run_if(in_state(GameState::Shop)),
        )
        .add_systems(
            Update,
            (
                climbing_movement_system,
                terrain_interaction_system,
                weather_system,
                wildlife_system,
                health_system,
            ).run_if(in_state(GameState::Climbing)),
        )
        .add_systems(
            Update,
            conversation_system.run_if(in_state(GameState::Conversation)),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Spawn camera
    commands.spawn(Camera2dBundle::default());
    
    // Initialize game resources
    commands.insert_resource(GameTime::new());
    commands.insert_resource(PlayerInventory::new(150.0, 20.0)); // $150, 20kg capacity
    commands.insert_resource(ShopInventory::default());
    commands.insert_resource(WeatherSystem::default());
    commands.insert_resource(Party::new(4));
    
    // Spawn initial player
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.2, 0.7, 0.2),
                custom_size: Some(Vec2::new(30.0, 50.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
            ..default()
        },
        Player { id: 1 },
        Health { current: 100.0, max: 100.0 },
        Hunger { current: 100.0, max: 100.0 },
        Thirst { current: 100.0, max: 100.0 },
        Morale { current: 100.0, max: 100.0 },
        MovementStats {
            speed: 100.0,
            climbing_skill: 1.0,
            stamina: 100.0,
            max_stamina: 100.0,
        },
        Position { x: 0.0, y: 0.0, z: 0.0 },
        Velocity { x: 0.0, y: 0.0, z: 0.0 },
        Climbing {
            is_climbing: false,
            anchor_point: None,
            rope_length: 0.0,
        },
        SelectedCharacter,
    ));
    
    // Start in shop state
    next_state.set(GameState::Shop);
    
    // Create sample levels
    if let Err(e) = levels::save_sample_levels() {
        warn!("Failed to save sample levels: {}", e);
    } else {
        info!("Sample levels created successfully!");
    }
    
    info!("Game initialized! Press 1-2 to buy items in shop, Enter to start climbing");
    info!("Controls: WASD/Arrow keys to move, I for inventory, ESC for menu");
}

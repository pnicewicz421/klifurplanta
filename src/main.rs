use bevy::prelude::*;

mod components;
mod levels;
mod resources;
mod states;
mod systems;

use components::*;
use resources::*;
use states::*;
use systems::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .add_event::<TerrainBrokenEvent>()
        .add_systems(Startup, (setup, setup_ui, load_terrain_from_level))
        .add_systems(PostStartup, setup_starting_equipment)
        .add_systems(
            Update,
            (
                // Phase 2+ systems with health & stamina
                player_movement_system,  // Consolidated movement and stamina system
                health_stamina_display_system,
                update_health_stamina_ui,
                camera_follow_system,
                update_time,
                // Equipment systems
                inventory_input_system,
                apply_equipment_bonuses,
                // Ice axe terrain interaction systems
                ice_axe_interaction_system,
                terrain_broken_handler_system,
            )
                .run_if(in_state(GameState::Climbing)),
        )
        .add_systems(OnEnter(GameState::Inventory), setup_inventory_ui)
        .add_systems(OnExit(GameState::Inventory), cleanup_inventory_ui)
        .add_systems(
            Update,
            (update_inventory_ui, close_button_system).run_if(in_state(GameState::Inventory)),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut next_state: ResMut<NextState<GameState>>) {
    // Generate the new procedural level files
    if let Err(e) = crate::levels::save_sample_levels() {
        error!("Failed to generate sample levels: {}", e);
    } else {
        info!("Generated new procedural level files");
    }

    // Spawn camera
    commands.spawn(Camera2dBundle::default());

    // Initialize basic game resources
    commands.insert_resource(GameTime::new());

    // Initialize item images resource and load ice axe image
    let mut item_images = ItemImages::new();
    item_images.load_item_image(&asset_server, "ice_axe_01", "images/items/ice_axe.png");
    commands.insert_resource(item_images);

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
        Health {
            current: 100.0,
            max: 100.0,
        },
        MovementStats {
            speed: 200.0,
            climbing_skill: 1.0,
            stamina: 100.0,
            max_stamina: 100.0,
        },
        // Add inventory and equipment components
        Inventory {
            items: Vec::new(),
            capacity: 20,
            weight_limit: 50.0,
            current_weight: 0.0,
        },
        EquippedItems::new(),
    ));

    // Start in climbing state for Phase 1
    next_state.set(GameState::Climbing);

    info!("🎮 Health & Stamina Active! Watch console: Move=drain stamina, Rest=regenerate, Ice/Snow=damage!");
    info!("📊 Health/Stamina display updates every 2 seconds. Move around to see effects!");
}

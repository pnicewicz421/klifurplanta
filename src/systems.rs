use crate::components::*;
use crate::resources::*;
use crate::states::*;
use bevy::prelude::*;

// Type aliases to fix clippy::type_complexity warnings
type CloseButtonQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static mut BackgroundColor),
    (Changed<Interaction>, With<CloseButton>),
>;

/// Unified player movement system that handles both movement and stamina/health effects
pub fn player_movement_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_query: Query<(&mut Transform, &mut MovementStats, &mut Health), With<Player>>,
    mut last_movement_log: Local<f32>,
    mut last_regen_log: Local<f32>,
) {
    for (mut transform, mut stats, mut health) in player_query.iter_mut() {
        let movement = get_movement_input(&keyboard_input);
        let is_moving = movement.length() > 0.0;
        
        if is_moving {
            handle_player_movement(
                &mut transform,
                &mut stats,
                movement,
                &time,
                &mut last_movement_log,
            );
        } else {
            handle_player_rest(
                &mut stats,
                &mut health,
                &time,
                &mut last_regen_log,
            );
        }

        check_player_death(&health);
    }
}

fn get_movement_input(keyboard_input: &Res<ButtonInput<KeyCode>>) -> Vec3 {
    let mut movement = Vec3::ZERO;
    
    if keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA) {
        movement.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD) {
        movement.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW) {
        movement.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) || keyboard_input.pressed(KeyCode::KeyS) {
        movement.y -= 1.0;
    }
    
    movement
}

fn handle_player_movement(
    transform: &mut Transform,
    stats: &mut MovementStats,
    movement: Vec3,
    time: &Res<Time>,
    last_movement_log: &mut f32,
) {
    let normalized_movement = movement.normalize_or_zero();
    transform.translation += normalized_movement * stats.speed * time.delta_seconds();
    
    let old_stamina = stats.stamina;
    let stamina_drain_rate = calculate_stamina_drain_rate(movement);
    
    // Apply stamina drain
    stats.stamina = (stats.stamina - stamina_drain_rate * time.delta_seconds()).max(0.0);

    // Prevent upward movement if out of stamina
    if stats.stamina <= 0.0 && movement.y > 0.0 {
        transform.translation.y -= normalized_movement.y * stats.speed * time.delta_seconds();
    }

    log_movement_effects(stats.stamina, old_stamina, time, last_movement_log);
}

fn calculate_stamina_drain_rate(movement: Vec3) -> f32 {
    if movement.y > 0.0 {
        15.0 // Climbing up is more exhausting
    } else {
        5.0 // Moving horizontally or downward is less exhausting
    }
}

fn log_movement_effects(current_stamina: f32, old_stamina: f32, time: &Res<Time>, last_movement_log: &mut f32) {
    *last_movement_log += time.delta_seconds();
    if *last_movement_log >= 0.5 {
        *last_movement_log = 0.0;
        let stamina_lost = old_stamina - current_stamina;
        if stamina_lost > 0.0 {
            info!(
                "🏃 Moving! Stamina: {:.1}/100 (-{:.1})",
                current_stamina, stamina_lost
            );
        }
    }
}

fn handle_player_rest(
    stats: &mut MovementStats,
    health: &mut Health,
    time: &Res<Time>,
    last_regen_log: &mut f32,
) {
    let old_stamina = stats.stamina;
    let old_health = health.current;

    // Regenerate stamina when not moving
    let stamina_regen_rate = 15.0;
    stats.stamina = (stats.stamina + stamina_regen_rate * time.delta_seconds()).min(stats.max_stamina);

    // Slow health regeneration when resting
    let health_regen_rate = 2.0;
    health.current = (health.current + health_regen_rate * time.delta_seconds()).min(health.max);

    log_regeneration_effects(stats, health, old_stamina, old_health, time, last_regen_log);
}

fn log_regeneration_effects(
    stats: &MovementStats,
    health: &Health,
    old_stamina: f32,
    old_health: f32,
    time: &Res<Time>,
    last_regen_log: &mut f32,
) {
    *last_regen_log += time.delta_seconds();
    if *last_regen_log >= 3.0 {
        *last_regen_log = 0.0;
        if stats.stamina < stats.max_stamina || health.current < health.max {
            info!(
                "💚 Resting... Stamina: {:.1}/100 (+{:.1}), Health: {:.1}/100 (+{:.1})",
                stats.stamina,
                stats.stamina - old_stamina,
                health.current,
                health.current - old_health
            );
        }
    }
}

fn check_player_death(health: &Health) {
    if health.current <= 0.0 {
        error!("💀 Player has died! Health reached zero.");
    }
}

// ===== PHASE 2: TERRAIN LOADING FROM FILES =====

/// System to load and spawn terrain from level files
pub fn load_terrain_from_level(mut commands: Commands) {
    let level_path = "levels/large_mountain_01.ron";

    match crate::levels::LevelDefinition::load_from_file(level_path) {
        Ok(level) => {
            spawn_level_terrain(&mut commands, &level);
            log_level_loading_success(level_path, &level);
        }
        Err(e) => {
            error!("Failed to load level {}: {}", level_path, e);
            spawn_simple_fallback_terrain(&mut commands);
        }
    }
}

fn spawn_level_terrain(commands: &mut Commands, level: &crate::levels::LevelDefinition) {
    info!("Loading level: {}", level.name);

    for (row_idx, row) in level.terrain.iter().enumerate() {
        for (col_idx, terrain_data) in row.iter().enumerate() {
            spawn_terrain_tile(commands, level, row_idx, col_idx, terrain_data);
        }
    }
}

fn spawn_terrain_tile(
    commands: &mut Commands,
    level: &crate::levels::LevelDefinition,
    row_idx: usize,
    col_idx: usize,
    terrain_data: &crate::levels::TerrainData,
) {
    let color = get_terrain_color(&terrain_data.terrain_type);
    let climbable = determine_climbability(&terrain_data.terrain_type, terrain_data.climbable);

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::new(32.0, 32.0)),
                ..default()
            },
            transform: Transform::from_translation(calculate_tile_position(
                level, row_idx, col_idx,
            )),
            ..default()
        },
        TerrainTile {
            terrain_type: terrain_data.terrain_type.clone(),
            slope: terrain_data.slope,
            stability: terrain_data.stability,
            climbable,
        },
    ));
}

fn determine_climbability(terrain_type: &TerrainType, file_climbable: bool) -> bool {
    match terrain_type {
        TerrainType::Soil => true, // Always allow movement through soil/brown terrain
        _ => file_climbable,       // Use level file setting for other terrain
    }
}

fn calculate_tile_position(
    level: &crate::levels::LevelDefinition,
    row_idx: usize,
    col_idx: usize,
) -> Vec3 {
    Vec3::new(
        (col_idx as f32 - level.width as f32 / 2.0) * 32.0,
        (level.height as f32 / 2.0 - row_idx as f32) * 32.0,
        0.0,
    )
}

fn log_level_loading_success(level_path: &str, level: &crate::levels::LevelDefinition) {
    info!(
        "Terrain loaded from {}: {}x{} tiles",
        level_path, level.width, level.height
    );
    info!("Terrain types: Brown=soil, Gray=rock, Blue=ice, Green=grass, White=snow");
}

/// Helper function to get color for terrain type
fn get_terrain_color(terrain_type: &TerrainType) -> Color {
    match terrain_type {
        TerrainType::Soil => Color::srgb(0.6, 0.4, 0.2),     // Brown
        TerrainType::Rock => Color::srgb(0.5, 0.5, 0.5),     // Gray
        TerrainType::Ice => Color::srgb(0.6, 0.8, 1.0),      // Light blue
        TerrainType::Snow => Color::srgb(0.9, 0.9, 0.9),     // White
        TerrainType::Grass => Color::srgb(0.2, 0.7, 0.2),    // Green
        TerrainType::Glacier => Color::srgb(0.8, 0.95, 1.0), // Bright icy blue
        TerrainType::Lava => Color::srgb(0.2, 0.1, 0.1),     // Dark reddish-black
        TerrainType::Coast => Color::srgb(0.8, 0.7, 0.5),    // Sandy beige
    }
}

/// Fallback terrain spawning if level loading fails
fn spawn_simple_fallback_terrain(commands: &mut Commands) {
    warn!("Using fallback terrain generation");

    // Create a simple 10x8 grid
    for x in -5..5 {
        for y in -4..4 {
            let terrain_type = if y < -2 {
                TerrainType::Soil
            } else if y < 0 {
                TerrainType::Rock
            } else {
                TerrainType::Grass
            };

            let color = get_terrain_color(&terrain_type);

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color,
                        custom_size: Some(Vec2::new(32.0, 32.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(
                        x as f32 * 32.0,
                        y as f32 * 32.0,
                        0.0,
                    )),
                    ..default()
                },
                TerrainTile {
                    terrain_type: terrain_type.clone(),
                    slope: 0.0,
                    stability: 1.0,
                    climbable: true, // All terrain should be climbable by default
                },
            ));
        }
    }
}

/// Camera follow system - smoothly follows the player
pub fn camera_follow_system(
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    player_query: Query<&Transform, (With<Player>, Without<Camera>)>,
    time: Res<Time>,
) {
    if let (Ok(mut camera_transform), Ok(player_transform)) =
        (camera_query.get_single_mut(), player_query.get_single())
    {
        let target_position = Vec3::new(
            player_transform.translation.x,
            player_transform.translation.y,
            camera_transform.translation.z, // Keep camera's Z position
        );

        // Smooth camera following with lerp
        let follow_speed = 2.0;
        camera_transform.translation = camera_transform
            .translation
            .lerp(target_position, follow_speed * time.delta_seconds());
    }
}

// ===== PHASE 1: BASIC PLAYER MOVEMENT =====

/// Health and stamina status display system - shows current values periodically
pub fn health_stamina_display_system(
    time: Res<Time>,
    player_query: Query<(&MovementStats, &Health), With<Player>>,
    mut last_update: Local<f32>,
) {
    // Update display every 2 seconds
    *last_update += time.delta_seconds();
    if *last_update >= 2.0 {
        *last_update = 0.0;

        for (stats, health) in player_query.iter() {
            info!(
                "📊 Health: {:.1}/{:.1} | Stamina: {:.1}/{:.1}",
                health.current, health.max, stats.stamina, stats.max_stamina
            );
        }
    }
}

/// Simple time update system (renamed from time_system)
pub fn update_time(time: Res<Time>, mut game_time: ResMut<GameTime>) {
    game_time.update(time.delta_seconds());
}

// ===== CORE SYSTEMS =====

pub fn time_system(time: Res<Time>, mut game_time: ResMut<GameTime>) {
    game_time.update(time.delta_seconds());
}

pub fn input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    // Global state transitions
    if keys.just_pressed(KeyCode::Escape) {
        match current_state.get() {
            GameState::Menu => {}
            _ => next_state.set(GameState::Menu),
        }
    }

    if keys.just_pressed(KeyCode::KeyI) {
        match current_state.get() {
            GameState::Inventory => next_state.set(GameState::Climbing),
            GameState::Climbing => next_state.set(GameState::Inventory),
            _ => {}
        }
    }
}

// ===== SHOP SYSTEMS =====

pub fn shop_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut inventory: ResMut<PlayerInventory>,
    shop: Res<ShopInventory>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Handle shop interactions
    if keys.just_pressed(KeyCode::Enter) {
        next_state.set(GameState::Climbing);
        info!("Leaving shop, starting climb!");
        return;
    }

    // Handle item purchases
    match () {
        _ if keys.just_pressed(KeyCode::Digit1) => {
            try_purchase_item(&mut inventory, &shop, "rope");
        }
        _ if keys.just_pressed(KeyCode::Digit2) => {
            try_purchase_item(&mut inventory, &shop, "tent");
        }
        _ => {}
    }
}

fn try_purchase_item(inventory: &mut PlayerInventory, shop: &ShopInventory, item_id: &str) {
    if let Some(shop_item) = shop.items.get(item_id) {
        if inventory.money >= shop_item.price && inventory.can_add_item(&shop_item.item) {
            inventory.money -= shop_item.price;
            inventory.add_item(shop_item.item.clone());
            info!("Bought {} for {}", item_id, shop_item.price);
        } else {
            info!("Cannot buy {} - not enough money or space", item_id);
        }
    }
}

pub fn inventory_ui_system(inventory: Res<PlayerInventory>, game_time: Res<GameTime>) {
    // In a real game, this would update UI elements
    // For now, just log every few seconds
    if game_time.real_seconds_elapsed % 3.0 < 0.1 {
        info!(
            "Money: ${:.2}, Weight: {:.1}/{:.1} kg",
            inventory.money, inventory.current_weight, inventory.max_weight
        );
        info!("Items: {}", inventory.items.len());
    }
}

// ===== CLIMBING SYSTEMS =====

pub fn terrain_interaction_system(
    player_query: Query<&Transform, With<Player>>,
    _terrain_query: Query<&TerrainTile>,
    mut health_query: Query<&mut Health, With<Player>>,
) {
    // Basic terrain interaction - would be expanded with proper collision detection
    for player_transform in player_query.iter() {
        for mut health in health_query.iter_mut() {
            // Simple hazard check based on position
            let player_y = player_transform.translation.y;

            // High altitude effects
            if player_y > 500.0 {
                health.current -= 1.0 * 0.016; // Lose health at high altitude
            }
        }
    }
}

pub fn weather_system(
    mut weather: ResMut<WeatherSystem>,
    time: Res<Time>,
    game_time: Res<GameTime>,
    mut health_query: Query<&mut Health, With<Player>>,
) {
    weather.weather_change_timer += time.delta_seconds();

    // Change weather every 2-5 minutes of real time
    if weather.weather_change_timer > 120.0 {
        weather.weather_change_timer = 0.0;

        // Simple weather changes based on time of day
        if game_time.is_night() {
            weather.temperature -= 5.0;
            weather.current_weather = Weather::Clear; // Simplified
        } else {
            weather.temperature += 3.0;
        }

        weather.temperature = weather.temperature.clamp(-20.0, 25.0);
    }

    // Apply weather effects to players
    for mut health in health_query.iter_mut() {
        if weather.temperature < -10.0 {
            health.current -= 0.5 * time.delta_seconds(); // Cold damage
        }

        match weather.current_weather {
            Weather::Storm | Weather::Blizzard => {
                health.current -= 0.2 * time.delta_seconds();
            }
            _ => {}
        }
    }
}

pub fn wildlife_system(
    mut wildlife_query: Query<(&mut Transform, &Wildlife)>,
    player_query: Query<&Transform, (With<Player>, Without<Wildlife>)>,
    time: Res<Time>,
) {
    // Basic wildlife behavior - animals move randomly or flee from players
    for (mut animal_transform, wildlife) in wildlife_query.iter_mut() {
        for player_transform in player_query.iter() {
            let distance = animal_transform
                .translation
                .distance(player_transform.translation);

            if distance < wildlife.flee_distance {
                // Animal flees from player
                let flee_direction =
                    (animal_transform.translation - player_transform.translation).normalize();
                animal_transform.translation += flee_direction * 50.0 * time.delta_seconds();
            }
        }
    }
}

pub fn health_system(
    mut query: Query<(&mut Health, &Hunger, &Thirst), With<Player>>,
    mut game_over: ResMut<NextState<GameState>>,
    time: Res<Time>,
) {
    for (mut health, hunger, thirst) in query.iter_mut() {
        // Health loss from hunger/thirst
        if hunger.current <= 0.0 {
            health.current -= 2.0 * time.delta_seconds();
        }
        if thirst.current <= 0.0 {
            health.current -= 5.0 * time.delta_seconds();
        }

        // Clamp health
        health.current = health.current.clamp(0.0, health.max);

        // Check for game over
        if health.current <= 0.0 {
            game_over.set(GameState::GameOver);
            warn!("Player died!");
        }
    }
}

// ===== CONVERSATION SYSTEM =====

pub fn conversation_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    npc_query: Query<&Npc>,
) {
    // Simple conversation system
    if keys.just_pressed(KeyCode::Space) {
        // End conversation
        next_state.set(GameState::Climbing);
        info!("Conversation ended");
    }

    // In a real implementation, this would handle dialogue trees
    for npc in npc_query.iter() {
        if keys.just_pressed(KeyCode::Digit1) {
            info!("Talking to {}", npc.name);
        }
    }
}

// ===== UI SYSTEMS =====

pub fn setup_ui(mut commands: Commands) {
    // Create UI root node
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            create_health_bar(parent);
            create_health_label(parent);
            create_stamina_bar(parent);
            create_stamina_label(parent);
        });
}

fn create_health_bar(parent: &mut ChildBuilder) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(300.0),
                height: Val::Px(30.0),
                margin: UiRect::bottom(Val::Px(10.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            background_color: Color::srgb(0.2, 0.2, 0.2).into(), // Dark background
            border_color: Color::srgb(0.6, 0.6, 0.6).into(),
            ..default()
        })
        .with_children(|parent| {
            // Health Bar Fill
            parent.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0), // Will be updated based on health
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    background_color: Color::srgb(0.8, 0.2, 0.2).into(), // Red health bar
                    ..default()
                },
                HealthBarFill,
            ));
        })
        .insert(HealthBar);
}

fn create_health_label(parent: &mut ChildBuilder) {
    parent.spawn(TextBundle::from_section(
        "Health: 100/100",
        TextStyle {
            font_size: 18.0,
            color: Color::WHITE,
            ..default()
        },
    ));
}

fn create_stamina_bar(parent: &mut ChildBuilder) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(300.0),
                height: Val::Px(30.0),
                margin: UiRect::top(Val::Px(20.0)).with_bottom(Val::Px(10.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            background_color: Color::srgb(0.2, 0.2, 0.2).into(), // Dark background
            border_color: Color::srgb(0.6, 0.6, 0.6).into(),
            ..default()
        })
        .with_children(|parent| {
            // Stamina Bar Fill
            parent.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0), // Will be updated based on stamina
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    background_color: Color::srgb(0.2, 0.8, 0.2).into(), // Green stamina bar
                    ..default()
                },
                StaminaBarFill,
            ));
        })
        .insert(StaminaBar);
}

fn create_stamina_label(parent: &mut ChildBuilder) {
    parent.spawn(TextBundle::from_section(
        "Stamina: 100/100",
        TextStyle {
            font_size: 18.0,
            color: Color::WHITE,
            ..default()
        },
    ));
}

pub fn update_health_stamina_ui(
    player_query: Query<(&Health, &MovementStats), With<Player>>,
    mut health_bar_query: Query<&mut Style, (With<HealthBarFill>, Without<StaminaBarFill>)>,
    mut stamina_bar_query: Query<&mut Style, (With<StaminaBarFill>, Without<HealthBarFill>)>,
    mut text_query: Query<&mut Text>,
) {
    if let Ok((health, movement_stats)) = player_query.get_single() {
        // Update health bar width
        for mut style in health_bar_query.iter_mut() {
            let health_percentage = (health.current / health.max) * 100.0;
            style.width = Val::Percent(health_percentage);
        }

        // Update stamina bar width
        for mut style in stamina_bar_query.iter_mut() {
            let stamina_percentage = (movement_stats.stamina / movement_stats.max_stamina) * 100.0;
            style.width = Val::Percent(stamina_percentage);
        }

        // Update text labels
        let mut texts = text_query.iter_mut().collect::<Vec<_>>();
        if texts.len() >= 2 {
            // Health text
            if let Some(text_sections) = texts[0].sections.first_mut() {
                text_sections.value = format!("Health: {:.0}/{:.0}", health.current, health.max);
            }
            // Stamina text
            if let Some(text_sections) = texts[1].sections.first_mut() {
                text_sections.value = format!(
                    "Stamina: {:.0}/{:.0}",
                    movement_stats.stamina, movement_stats.max_stamina
                );
            }
        }
    }
}

// ===== INVENTORY & EQUIPMENT SYSTEMS =====

pub fn setup_starting_equipment(
    mut player_query: Query<(&mut Inventory, &mut EquippedItems), With<Player>>,
) {
    if let Ok((mut inventory, mut equipped)) = player_query.get_single_mut() {
        info!("🎒 Setting up starting equipment for player...");

        let starting_items = create_starting_items();
        equip_starting_items(&mut inventory, &mut equipped, starting_items);
        
        info!("🎒 Starting equipment loaded: Ice Axe (+15% climb), Heavy Boots (+10% climb, +20 warmth), Wool Jacket (+30 warmth)");
    } else {
        warn!("⚠️ Could not find player entity to add starting equipment!");
    }
}

fn create_starting_items() -> (Item, Item, Item) {
    let ice_axe = create_ice_axe();
    let heavy_boots = create_heavy_boots();
    let wool_jacket = create_wool_jacket();
    
    (ice_axe, heavy_boots, wool_jacket)
}

fn create_ice_axe() -> Item {
    Item {
        id: "ice_axe_01".to_string(),
        name: "Ice Axe".to_string(),
        weight: 1.5,
        item_type: ItemType::ClimbingGear,
        durability: Some(100.0),
        properties: ItemProperties {
            strength: Some(15.0),
            warmth: None,
            magic_power: None,
            nutrition: None,
            water: None,
            protection: Some(5.0),
        },
    }
}

fn create_heavy_boots() -> Item {
    Item {
        id: "heavy_boots_01".to_string(),
        name: "Heavy Climbing Boots".to_string(),
        weight: 3.0,
        item_type: ItemType::Clothing,
        durability: Some(100.0),
        properties: ItemProperties {
            strength: Some(10.0),
            warmth: Some(20.0),
            magic_power: None,
            nutrition: None,
            water: None,
            protection: Some(15.0),
        },
    }
}

fn create_wool_jacket() -> Item {
    Item {
        id: "wool_jacket_01".to_string(),
        name: "Wool Jacket".to_string(),
        weight: 2.0,
        item_type: ItemType::Clothing,
        durability: Some(100.0),
        properties: ItemProperties {
            strength: None,
            warmth: Some(30.0),
            magic_power: None,
            nutrition: None,
            water: None,
            protection: Some(10.0),
        },
    }
}

fn equip_starting_items(
    inventory: &mut Inventory,
    equipped: &mut EquippedItems,
    (ice_axe, heavy_boots, wool_jacket): (Item, Item, Item),
) {
    // Update inventory with starting items
    inventory.items = vec![ice_axe.clone(), heavy_boots.clone(), wool_jacket.clone()];
    inventory.current_weight = ice_axe.weight + heavy_boots.weight + wool_jacket.weight;

    // Equip items
    equipped.axe = Some(ice_axe);
    equipped.boots = Some(heavy_boots);
    equipped.jacket = Some(wool_jacket);
}

pub fn inventory_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    // Handle I key toggle
    if keyboard_input.just_pressed(KeyCode::KeyI) {
        match current_state.get() {
            GameState::Climbing => {
                next_state.set(GameState::Inventory);
                info!("📦 Opening inventory...");
            }
            GameState::Inventory => {
                next_state.set(GameState::Climbing);
                info!("📦 Closing inventory...");
            }
            _ => {}
        }
    }

    // Handle Escape key (only closes inventory)
    if keyboard_input.just_pressed(KeyCode::Escape) && current_state.get() == &GameState::Inventory
    {
        next_state.set(GameState::Climbing);
        info!("📦 Closing inventory with Escape...");
    }
}

pub fn close_button_system(
    mut interaction_query: CloseButtonQuery,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                next_state.set(GameState::Climbing);
                info!("📦 Closing inventory with button click...");
            }
            Interaction::Hovered => {
                *color = Color::srgb(0.8, 0.3, 0.3).into(); // Lighter red on hover
            }
            Interaction::None => {
                *color = Color::srgb(0.6, 0.2, 0.2).into(); // Original red
            }
        }
    }
}

pub fn setup_inventory_ui(mut commands: Commands) {
    // Main inventory container
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(80.0),
                    height: Val::Percent(70.0),
                    position_type: PositionType::Absolute,
                    left: Val::Percent(10.0),
                    top: Val::Percent(15.0),
                    flex_direction: FlexDirection::Column,
                    border: UiRect::all(Val::Px(3.0)),
                    padding: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
                background_color: Color::srgba(0.1, 0.1, 0.1, 0.9).into(),
                border_color: Color::srgb(0.6, 0.6, 0.6).into(),
                ..default()
            },
            InventoryUI,
        ))
        .with_children(|parent| {
            create_inventory_title_bar(parent);
            create_inventory_main_content(parent);
        });
}

fn create_inventory_title_bar(parent: &mut ChildBuilder) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(40.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // Title
            parent.spawn(TextBundle::from_section(
                "INVENTORY & EQUIPMENT",
                TextStyle {
                    font_size: 28.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));

            create_close_button(parent);
        });
}

fn create_close_button(parent: &mut ChildBuilder) {
    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(80.0),
                    height: Val::Px(35.0),
                    border: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::srgb(0.6, 0.2, 0.2).into(),
                border_color: Color::srgb(0.8, 0.4, 0.4).into(),
                ..default()
            },
            CloseButton,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "CLOSE",
                TextStyle {
                    font_size: 16.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
        });
}

fn create_inventory_main_content(parent: &mut ChildBuilder) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            create_equipment_panel(parent);
            create_inventory_panel(parent);
        });
}

fn create_equipment_panel(parent: &mut ChildBuilder) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(40.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                margin: UiRect::right(Val::Px(20.0)),
                ..default()
            },
            background_color: Color::srgba(0.2, 0.2, 0.3, 0.8).into(),
            ..default()
        })
        .with_children(|parent| {
            // Equipment title
            parent.spawn(TextBundle::from_section(
                "EQUIPPED",
                TextStyle {
                    font_size: 24.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));

            create_equipment_slots(parent);
        });
}

fn create_equipment_slots(parent: &mut ChildBuilder) {
    let equipment_slots = vec![
        ("🪓 Axe", EquipmentSlotType::Axe),
        ("👢 Boots", EquipmentSlotType::Boots),
        ("🧥 Jacket", EquipmentSlotType::Jacket),
        ("🧤 Gloves", EquipmentSlotType::Gloves),
        ("🎒 Backpack", EquipmentSlotType::Backpack),
    ];

    for (label, slot_type) in equipment_slots {
        create_single_equipment_slot(parent, label, slot_type);
    }
}

fn create_single_equipment_slot(parent: &mut ChildBuilder, label: &str, slot_type: EquipmentSlotType) {
    parent
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Px(60.0),
                    margin: UiRect::all(Val::Px(5.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    padding: UiRect::all(Val::Px(10.0)),
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::srgba(0.3, 0.3, 0.4, 0.8).into(),
                border_color: Color::srgb(0.5, 0.5, 0.5).into(),
                ..default()
            },
            EquipmentSlot { slot_type },
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                format!("{}: Empty", label),
                TextStyle {
                    font_size: 16.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
        });
}

fn create_inventory_panel(parent: &mut ChildBuilder) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(60.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: Color::srgba(0.2, 0.3, 0.2, 0.8).into(),
            ..default()
        })
        .with_children(|parent| {
            // Inventory title
            parent.spawn(TextBundle::from_section(
                "INVENTORY",
                TextStyle {
                    font_size: 24.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));

            create_inventory_grid(parent);
            create_stats_panel(parent);
        });
}

fn create_inventory_grid(parent: &mut ChildBuilder) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(80.0),
                flex_direction: FlexDirection::Column,
                flex_wrap: FlexWrap::Wrap,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // Create inventory slots
            for i in 0..20 {
                create_single_inventory_slot(parent, i);
            }
        });
}

fn create_single_inventory_slot(parent: &mut ChildBuilder, slot_index: usize) {
    parent
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Px(80.0),
                    height: Val::Px(80.0),
                    margin: UiRect::all(Val::Px(2.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    padding: UiRect::all(Val::Px(5.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: Color::srgba(0.4, 0.4, 0.4, 0.6).into(),
                border_color: Color::srgb(0.6, 0.6, 0.6).into(),
                ..default()
            },
            InventorySlot { slot_index },
        ))
        .with_children(|parent| {
            // Add image placeholder
            parent.spawn((
                ImageBundle {
                    style: Style {
                        width: Val::Px(60.0),
                        height: Val::Px(60.0),
                        ..default()
                    },
                    image: UiImage::default(),
                    visibility: Visibility::Hidden, // Hidden by default
                    ..default()
                },
                InventorySlotImage { slot_index },
            ));
            
            // Add text label
            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size: 10.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                InventorySlotText { slot_index },
            ));
        });
}

fn create_stats_panel(parent: &mut ChildBuilder) {
    parent.spawn(TextBundle::from_section(
        "Weight: 0/25 kg",
        TextStyle {
            font_size: 16.0,
            color: Color::srgb(1.0, 1.0, 0.0), // Yellow
            ..default()
        },
    ));
}

pub fn update_inventory_ui(
    player_query: Query<(&Inventory, &EquippedItems), With<Player>>,
    mut image_slots_query: Query<(&mut UiImage, &mut Visibility, &InventorySlotImage)>,
    mut text_slots_query: Query<(&mut Text, &InventorySlotText)>,
    equipment_slots_query: Query<Entity, With<EquipmentSlot>>,
    children_query: Query<&Children>,
    mut equipment_text_query: Query<&mut Text, (Without<InventorySlotText>, Without<InventorySlotImage>)>,
    item_images: Res<ItemImages>,
) {
    if let Ok((inventory, equipped)) = player_query.get_single() {
        // Update equipment display
        update_equipment_display(
            &equipment_slots_query,
            &mut equipment_text_query,
            &children_query,
            equipped,
        );
        
        // Update inventory slots with items and images
        update_inventory_slots(
            inventory,
            &mut image_slots_query,
            &mut text_slots_query,
            &item_images,
        );
        
        // Update weight display (using equipment_text_query for non-slot text)
        update_weight_display(&mut equipment_text_query, inventory);
    }
}

fn update_inventory_slots(
    inventory: &Inventory,
    image_slots_query: &mut Query<(&mut UiImage, &mut Visibility, &InventorySlotImage)>,
    text_slots_query: &mut Query<(&mut Text, &InventorySlotText)>,
    item_images: &Res<ItemImages>,
) {
    reset_all_inventory_slots(image_slots_query, text_slots_query);
    populate_inventory_slots(inventory, image_slots_query, text_slots_query, item_images);
}

fn reset_all_inventory_slots(
    image_slots_query: &mut Query<(&mut UiImage, &mut Visibility, &InventorySlotImage)>,
    text_slots_query: &mut Query<(&mut Text, &InventorySlotText)>,
) {
    for (mut image, mut visibility, _) in image_slots_query.iter_mut() {
        *visibility = Visibility::Hidden;
        *image = UiImage::default();
    }
    
    for (mut text, _) in text_slots_query.iter_mut() {
        text.sections[0].value = "".to_string();
    }
}

fn populate_inventory_slots(
    inventory: &Inventory,
    image_slots_query: &mut Query<(&mut UiImage, &mut Visibility, &InventorySlotImage)>,
    text_slots_query: &mut Query<(&mut Text, &InventorySlotText)>,
    item_images: &Res<ItemImages>,
) {
    for (index, item) in inventory.items.iter().enumerate() {
        if index >= 20 { break; } // Only 20 slots available
        
        update_slot_image(index, item, image_slots_query, item_images);
        update_slot_text_for_item(index, item, text_slots_query);
    }
}

fn update_slot_image(
    index: usize,
    item: &Item,
    image_slots_query: &mut Query<(&mut UiImage, &mut Visibility, &InventorySlotImage)>,
    item_images: &Res<ItemImages>,
) {
    for (mut image, mut visibility, slot_image) in image_slots_query.iter_mut() {
        if slot_image.slot_index == index {
            if let Some(item_image) = item_images.get_image(&item.id) {
                image.texture = item_image.clone();
                *visibility = Visibility::Visible;
            }
            break;
        }
    }
}

fn update_slot_text_for_item(
    index: usize,
    item: &Item,
    text_slots_query: &mut Query<(&mut Text, &InventorySlotText)>,
) {
    for (mut text, slot_text) in text_slots_query.iter_mut() {
        if slot_text.slot_index == index {
            text.sections[0].value = item.name.clone();
            break;
        }
    }
}

fn update_equipment_display(
    equipment_slots_query: &Query<Entity, With<EquipmentSlot>>,
    text_query: &mut Query<&mut Text, (Without<InventorySlotText>, Without<InventorySlotImage>)>,
    children_query: &Query<&Children>,
    equipped: &EquippedItems,
) {
    for equipment_entity in equipment_slots_query.iter() {
        if let Ok(children) = children_query.get(equipment_entity) {
            for child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(*child) {
                    update_slot_text(&mut text, equipped);
                    break;
                }
            }
        }
    }
}

fn update_slot_text(text: &mut Text, equipped: &EquippedItems) {
    let current_text = &text.sections[0].value;
    text.sections[0].value = get_equipment_slot_text(current_text, equipped);
}

fn get_equipment_slot_text(current_text: &str, equipped: &EquippedItems) -> String {
    match current_text {
        text if text.contains("🪓") || text.contains("Axe") => {
            format_equipment_slot("🪓", "Axe", &equipped.axe)
        }
        text if text.contains("👢") || text.contains("Boots") => {
            format_equipment_slot("👢", "Boots", &equipped.boots)
        }
        text if text.contains("🧥") || text.contains("Jacket") => {
            format_equipment_slot("🧥", "Jacket", &equipped.jacket)
        }
        text if text.contains("🧤") || text.contains("Gloves") => {
            format_equipment_slot("🧤", "Gloves", &equipped.gloves)
        }
        text if text.contains("🎒") || text.contains("Backpack") => {
            format_equipment_slot("🎒", "Backpack", &equipped.backpack)
        }
        _ => current_text.to_string(),
    }
}

fn format_equipment_slot(icon: &str, slot_name: &str, item: &Option<Item>) -> String {
    if let Some(equipment) = item {
        format!("{} {}", icon, equipment.name)
    } else {
        format!("{} {}: Empty", icon, slot_name)
    }
}

fn update_weight_display(text_query: &mut Query<&mut Text, (Without<InventorySlotText>, Without<InventorySlotImage>)>, inventory: &Inventory) {
    for mut text in text_query.iter_mut() {
        if !text.sections.is_empty() && text.sections[0].value.contains("Weight:") {
            text.sections[0].value = format!(
                "Weight: {:.1}/{:.0} kg",
                inventory.current_weight, inventory.weight_limit
            );
            break;
        }
    }
}

pub fn cleanup_inventory_ui(
    mut commands: Commands,
    inventory_ui_query: Query<Entity, With<InventoryUI>>,
) {
    for entity in inventory_ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn apply_equipment_bonuses(
    mut player_query: Query<(&mut MovementStats, &EquippedItems), With<Player>>,
) {
    for (mut movement_stats, equipped) in player_query.iter_mut() {
        // Base climbing skill
        let base_skill = 1.0;

        // Apply equipment bonuses
        let equipment_bonus = equipped.get_climbing_bonus() / 100.0; // Convert percentage to decimal

        // Update climbing skill with equipment bonus
        movement_stats.climbing_skill = base_skill + equipment_bonus;

        // You could also modify movement speed based on boots, etc.
        // movement_stats.speed = base_speed * boot_modifier;
    }
}

// ===== ICE AXE TERRAIN INTERACTION SYSTEM =====

/// System for ice axe terrain breaking interaction
pub fn ice_axe_interaction_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    _mouse_input: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut player_query: Query<(&Transform, &Inventory, &EquippedItems), With<Player>>,
    mut terrain_query: Query<(Entity, &Transform, &mut TerrainTile, Option<&mut Breakable>)>,
    mut terrain_broken_events: EventWriter<TerrainBrokenEvent>,
) {
    if !keyboard_input.just_pressed(KeyCode::KeyX) {
        return;
    }

    for (player_transform, inventory, _equipped) in player_query.iter_mut() {
        if has_ice_axe(inventory) {
            attempt_terrain_break(
                player_transform,
                &mut commands,
                &mut terrain_query,
                &mut terrain_broken_events,
            );
        } else {
            warn!("❌ No ice axe available! Check your inventory or equipped items.");
        }
    }
}

fn has_ice_axe(inventory: &Inventory) -> bool {
    get_ice_axe_from_inventory(inventory).is_some()
}

fn attempt_terrain_break(
    player_transform: &Transform,
    commands: &mut Commands,
    terrain_query: &mut Query<(Entity, &Transform, &mut TerrainTile, Option<&mut Breakable>)>,
    terrain_broken_events: &mut EventWriter<TerrainBrokenEvent>,
) {
    let reach_distance = 40.0;
    
    for (terrain_entity, terrain_transform, mut terrain_tile, breakable) in terrain_query.iter_mut() {
        if is_breakable_terrain_in_reach(player_transform, terrain_transform, &terrain_tile, reach_distance) {
            process_terrain_break(
                commands,
                terrain_entity,
                &mut terrain_tile,
                breakable,
                terrain_transform.translation,
                terrain_broken_events,
            );
            break; // Only break one terrain tile at a time
        }
    }
}

fn is_breakable_terrain_in_reach(
    player_transform: &Transform,
    terrain_transform: &Transform,
    terrain_tile: &TerrainTile,
    reach_distance: f32,
) -> bool {
    let distance = player_transform.translation.distance(terrain_transform.translation);
    let is_ice_terrain = matches!(terrain_tile.terrain_type, TerrainType::Ice | TerrainType::Glacier);
    
    distance <= reach_distance && is_ice_terrain
}

fn process_terrain_break(
    commands: &mut Commands,
    terrain_entity: Entity,
    terrain_tile: &mut TerrainTile,
    breakable: Option<Mut<Breakable>>,
    position: Vec3,
    terrain_broken_events: &mut EventWriter<TerrainBrokenEvent>,
) {
    match breakable {
        Some(mut breakable_comp) => {
            apply_axe_damage(
                commands,
                terrain_entity,
                terrain_tile,
                &mut breakable_comp,
                position,
                terrain_broken_events,
            );
        }
        None => {
            add_breakable_component(commands, terrain_entity);
        }
    }
}

fn add_breakable_component(commands: &mut Commands, terrain_entity: Entity) {
    commands.entity(terrain_entity).insert(Breakable {
        tool_required: ToolType::IceAxe,
        durability: 50.0,
        max_durability: 50.0,
    });
}

/// Helper function to get ice axe from inventory or equipped items
fn get_ice_axe_from_inventory(inventory: &Inventory) -> Option<&Item> {
    // Check inventory for ice axe
    inventory.items.iter().find(|item| {
        item.name.to_lowercase().contains("ice axe") || item.name.to_lowercase().contains("axe")
    })
}

/// Break ice terrain with ice axe
fn apply_axe_damage(
    commands: &mut Commands,
    terrain_entity: Entity,
    terrain_tile: &mut TerrainTile,
    breakable: &mut Breakable,
    position: Vec3,
    terrain_broken_events: &mut EventWriter<TerrainBrokenEvent>,
) {
    let original_terrain_type = terrain_tile.terrain_type.clone();
    let damage = 25.0; // Damage per axe hit
    
    reduce_terrain_durability(breakable, damage, &original_terrain_type);
    
    if is_terrain_broken(breakable) {
        complete_terrain_break(
            commands,
            terrain_entity,
            terrain_tile,
            position,
            terrain_broken_events,
            original_terrain_type,
        );
    }
}

fn reduce_terrain_durability(breakable: &mut Breakable, damage: f32, terrain_type: &TerrainType) {
    breakable.durability = (breakable.durability - damage).max(0.0);
    
    info!(
        "🪓 Breaking {:?} terrain! Durability: {:.1}/{:.1}",
        terrain_type, breakable.durability, breakable.max_durability
    );
}

fn is_terrain_broken(breakable: &Breakable) -> bool {
    breakable.durability <= 0.0
}

fn complete_terrain_break(
    commands: &mut Commands,
    terrain_entity: Entity,
    terrain_tile: &mut TerrainTile,
    position: Vec3,
    terrain_broken_events: &mut EventWriter<TerrainBrokenEvent>,
    original_terrain_type: TerrainType,
) {
    // Convert to passable terrain
    terrain_tile.terrain_type = TerrainType::Soil;
    terrain_tile.climbable = true;

    // Send terrain broken event
    terrain_broken_events.send(TerrainBrokenEvent {
        position,
        terrain_type: original_terrain_type.clone(),
        tool_used: ToolType::IceAxe,
    });

    // Remove Breakable component as terrain is now broken
    commands.entity(terrain_entity).remove::<Breakable>();

    info!("✅ {:?} terrain broken! Path is now clear.", original_terrain_type);
}

/// System to handle terrain broken events and update visuals
pub fn terrain_broken_handler_system(
    mut terrain_broken_events: EventReader<TerrainBrokenEvent>,
    mut terrain_query: Query<(&Transform, &mut Sprite, &TerrainTile)>,
) {
    for event in terrain_broken_events.read() {
        // Update terrain visual to reflect the change
        for (transform, mut sprite, terrain_tile) in terrain_query.iter_mut() {
            if transform.translation.distance(event.position) < 5.0 {
                // Update sprite color to match new terrain type
                sprite.color = terrain_tile.terrain_type.color();
                break;
            }
        }

        info!(
            "🎉 Terrain broken at {:?} using {:?}",
            event.position, event.tool_used
        );
    }
}

// ===== DIALOGUE SYSTEM =====

/// System to detect when player is near NPCs and show interaction prompt
pub fn npc_proximity_system(
    player_query: Query<&Transform, With<Player>>,
    npc_query: Query<(Entity, &Transform, &Npc, &ConversationRange), Without<Player>>,
    mut commands: Commands,
    existing_prompts: Query<Entity, With<InteractionPrompt>>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    clear_existing_prompts(&mut commands, &existing_prompts);
    check_npc_proximity(player_transform, &npc_query, &mut commands);
}

fn clear_existing_prompts(
    commands: &mut Commands,
    existing_prompts: &Query<Entity, With<InteractionPrompt>>,
) {
    for entity in existing_prompts.iter() {
        commands.entity(entity).despawn();
    }
}

fn check_npc_proximity(
    player_transform: &Transform,
    npc_query: &Query<(Entity, &Transform, &Npc, &ConversationRange), Without<Player>>,
    commands: &mut Commands,
) {
    for (npc_entity, npc_transform, npc, range) in npc_query.iter() {
        let distance = player_transform.translation.distance(npc_transform.translation);
        
        if distance <= range.distance {
            spawn_interaction_prompt(commands, npc_entity, &npc.name);
            break; // Only show one prompt at a time
        }
    }
}

fn spawn_interaction_prompt(commands: &mut Commands, npc_entity: Entity, npc_name: &str) {
    commands.spawn((
        TextBundle::from_section(
            format!("Press E to talk to {}", npc_name),
            TextStyle {
                font_size: 16.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(150.0),
            left: Val::Percent(50.0),
            ..default()
        }),
        InteractionPrompt { target_npc: npc_entity },
    ));
}

/// System to handle starting conversations with NPCs
pub fn conversation_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
    prompt_query: Query<&InteractionPrompt>,
    npc_query: Query<&DialogueTree>,
) {
    if !keyboard_input.just_pressed(KeyCode::KeyE) {
        return;
    }

    let Ok(player_entity) = player_query.get_single() else {
        return;
    };

    if let Some(prompt) = prompt_query.iter().next() {
        start_conversation(&mut commands, player_entity, prompt.target_npc, &npc_query);
    }
}

fn start_conversation(
    commands: &mut Commands,
    player_entity: Entity,
    npc_entity: Entity,
    npc_query: &Query<&DialogueTree>,
) {
    if let Ok(dialogue_tree) = npc_query.get(npc_entity) {
        commands.entity(player_entity).insert(InConversation {
            with_npc: npc_entity,
            current_node: dialogue_tree.current_node.clone(),
        });
        spawn_dialogue_ui(commands, dialogue_tree, &dialogue_tree.current_node);
        info!("💬 Started conversation with NPC");
    }
}

/// System to update dialogue UI when conversation state changes
pub fn dialogue_ui_system(
    mut commands: Commands,
    conversation_query: Query<&InConversation, (With<Player>, Changed<InConversation>)>,
    npc_query: Query<&DialogueTree>,
    existing_ui: Query<Entity, With<DialogueUI>>,
) {
    // Clean up existing dialogue UI
    for entity in existing_ui.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Show new dialogue UI if in conversation
    if let Ok(conversation) = conversation_query.get_single() {
        if let Ok(dialogue_tree) = npc_query.get(conversation.with_npc) {
            spawn_dialogue_ui(&mut commands, dialogue_tree, &conversation.current_node);
        }
    }
}

fn spawn_dialogue_ui(
    commands: &mut Commands,
    dialogue_tree: &DialogueTree,
    current_node: &str,
) {
    if let Some(node) = dialogue_tree.nodes.get(current_node) {
        // Main dialogue container
        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(80.0),
                        height: Val::Percent(60.0),
                        position_type: PositionType::Absolute,
                        left: Val::Percent(10.0),
                        top: Val::Percent(20.0),
                        flex_direction: FlexDirection::Column,
                        border: UiRect::all(Val::Px(3.0)),
                        padding: UiRect::all(Val::Px(20.0)),
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    background_color: Color::srgba(0.1, 0.1, 0.2, 0.95).into(),
                    border_color: Color::srgb(0.6, 0.6, 0.8).into(),
                    ..default()
                },
                DialogueUI,
            ))
            .with_children(|parent| {
                // Header with speaker name and close button
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Px(40.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            margin: UiRect::bottom(Val::Px(15.0)),
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        // Speaker name
                        parent.spawn(TextBundle::from_section(
                            format!("💬 {}", node.speaker),
                            TextStyle {
                                font_size: 24.0,
                                color: Color::srgb(0.9, 0.9, 1.0),
                                ..default()
                            },
                        ));
                        
                        // Close button
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: Style {
                                        width: Val::Px(30.0),
                                        height: Val::Px(30.0),
                                        border: UiRect::all(Val::Px(2.0)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    background_color: Color::srgb(0.8, 0.3, 0.3).into(),
                                    border_color: Color::srgb(0.9, 0.5, 0.5).into(),
                                    ..default()
                                },
                                DialogueCloseButton,
                            ))
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section(
                                    "×",
                                    TextStyle {
                                        font_size: 20.0,
                                        color: Color::WHITE,
                                        ..default()
                                    },
                                ));
                            });
                    });

                // Main dialogue text area
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            min_height: Val::Px(80.0),
                            padding: UiRect::all(Val::Px(15.0)),
                            margin: UiRect::bottom(Val::Px(20.0)),
                            ..default()
                        },
                        background_color: Color::srgba(0.0, 0.0, 0.1, 0.5).into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn((
                            TextBundle::from_section(
                                &node.text,
                                TextStyle {
                                    font_size: 18.0,
                                    color: Color::srgb(0.95, 0.95, 1.0),
                                    ..default()
                                },
                            ),
                            DialogueText,
                        ));
                    });

                // Dialogue options area
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(8.0),
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        for (index, option) in node.options.iter().enumerate() {
                            let option_text = format!("{}. {}", index + 1, get_option_action(&option.text));
                            let button_color = get_option_color(index, &option.text);
                            
                            parent
                                .spawn((
                                    ButtonBundle {
                                        style: Style {
                                            width: Val::Percent(100.0),
                                            height: Val::Px(45.0),
                                            border: UiRect::all(Val::Px(2.0)),
                                            justify_content: JustifyContent::FlexStart,
                                            align_items: AlignItems::Center,
                                            padding: UiRect::all(Val::Px(10.0)),
                                            ..default()
                                        },
                                        background_color: button_color.into(),
                                        border_color: Color::srgb(0.7, 0.7, 0.8).into(),
                                        ..default()
                                    },
                                    DialogueOptionButton { option_index: index },
                                ))
                                .with_children(|parent| {
                                    parent.spawn(TextBundle::from_section(
                                        option_text,
                                        TextStyle {
                                            font_size: 16.0,
                                            color: Color::WHITE,
                                            ..default()
                                        },
                                    ));
                                });
                        }
                        
                        // Instructions
                        parent.spawn(TextBundle::from_section(
                            "Press 1-4 to choose, Esc to exit, or click the × button",
                            TextStyle {
                                font_size: 14.0,
                                color: Color::srgb(0.7, 0.7, 0.8),
                                ..default()
                            },
                        ));
                    });
            });
    }
}

fn get_option_action(option_text: &str) -> String {
    let text_lower = option_text.to_lowercase();
    
    let icon = if text_lower.contains("invite") || text_lower.contains("join") || text_lower.contains("party") {
        "🤝"
    } else if text_lower.contains("buy") || text_lower.contains("sell") || text_lower.contains("trade") {
        "💰"
    } else if text_lower.contains("guidance") || text_lower.contains("advice") || text_lower.contains("help") || text_lower.contains("question") {
        "❓"
    } else if text_lower.contains("goodbye") || text_lower.contains("leave") || text_lower.contains("passing") {
        "👋"
    } else {
        "💭"
    };
    
    format!("{} {}", icon, option_text)
}

fn get_option_color(index: usize, option_text: &str) -> Color {
    let text_lower = option_text.to_lowercase();
    
    if text_lower.contains("invite") || text_lower.contains("join") {
        Color::srgb(0.2, 0.7, 0.3) // Green for party invites
    } else if text_lower.contains("buy") || text_lower.contains("sell") || text_lower.contains("trade") {
        Color::srgb(0.7, 0.6, 0.2) // Gold for trading
    } else if text_lower.contains("guidance") || text_lower.contains("advice") || text_lower.contains("help") {
        Color::srgb(0.3, 0.5, 0.8) // Blue for information
    } else if text_lower.contains("goodbye") || text_lower.contains("leave") {
        Color::srgb(0.6, 0.4, 0.4) // Muted red for goodbye
    } else {
        match index {
            0 => Color::srgb(0.4, 0.6, 0.7), // Default blues/grays
            1 => Color::srgb(0.5, 0.5, 0.7),
            2 => Color::srgb(0.6, 0.5, 0.6),
            _ => Color::srgb(0.5, 0.5, 0.6),
        }
    }
}

/// System to handle dialogue progression and choices
pub fn dialogue_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut conversation_query: Query<(Entity, &mut InConversation), With<Player>>,
    npc_query: Query<&DialogueTree>,
    mut button_query: Query<&Interaction, (Changed<Interaction>, With<DialogueCloseButton>)>,
    mut option_button_query: Query<(&Interaction, &DialogueOptionButton), Changed<Interaction>>,
) {
    let Ok((player_entity, mut conversation)) = conversation_query.get_single_mut() else {
        return;
    };

    // Check for close button click
    for interaction in button_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            end_conversation(&mut commands, player_entity);
            return;
        }
    }

    // Check for option button clicks
    for (interaction, option_button) in option_button_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            process_dialogue_choice(&mut commands, player_entity, &mut conversation, &npc_query, option_button.option_index);
            return;
        }
    }

    handle_dialogue_input(
        &keyboard_input,
        &mut commands,
        player_entity,
        &mut conversation,
        &npc_query,
    );
}

fn handle_dialogue_input(
    keyboard_input: &Res<ButtonInput<KeyCode>>,
    commands: &mut Commands,
    player_entity: Entity,
    conversation: &mut InConversation,
    npc_query: &Query<&DialogueTree>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        end_conversation(commands, player_entity);
        return;
    }

    // Handle numbered choices (1-4)
    let choice = get_dialogue_choice_input(keyboard_input);
    if let Some(choice_num) = choice {
        process_dialogue_choice(commands, player_entity, conversation, npc_query, choice_num);
    }
}

fn get_dialogue_choice_input(keyboard_input: &Res<ButtonInput<KeyCode>>) -> Option<usize> {
    if keyboard_input.just_pressed(KeyCode::Digit1) { Some(0) }
    else if keyboard_input.just_pressed(KeyCode::Digit2) { Some(1) }
    else if keyboard_input.just_pressed(KeyCode::Digit3) { Some(2) }
    else if keyboard_input.just_pressed(KeyCode::Digit4) { Some(3) }
    else { None }
}

fn process_dialogue_choice(
    commands: &mut Commands,
    player_entity: Entity,
    conversation: &mut InConversation,
    npc_query: &Query<&DialogueTree>,
    choice_index: usize,
) {
    if let Ok(dialogue_tree) = npc_query.get(conversation.with_npc) {
        if let Some(node) = dialogue_tree.nodes.get(&conversation.current_node) {
            if let Some(option) = node.options.get(choice_index) {
                conversation.current_node = option.next_node.clone();
                
                // Check if this ends the conversation
                if option.next_node == "end" {
                    end_conversation(commands, player_entity);
                }
            }
        }
    }
}

/// System to clean up dialogue UI when conversation ends
pub fn cleanup_dialogue_ui_system(
    mut commands: Commands,
    player_query: Query<Entity, (With<Player>, Without<InConversation>)>,
    dialogue_ui_query: Query<Entity, With<DialogueUI>>,
    mut last_had_conversation: Local<bool>,
) {
    let player_has_conversation = player_query.is_empty();
    
    // If player had conversation last frame but doesn't now, clean up UI
    if *last_had_conversation && !player_has_conversation {
        for entity in dialogue_ui_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
    
    *last_had_conversation = player_has_conversation;
}

fn end_conversation(commands: &mut Commands, player_entity: Entity) {
    commands.entity(player_entity).remove::<InConversation>();
    info!("💬 Ended conversation");
}

// ===== PARTY INVITATION SYSTEM =====

/// System to handle party invitations with acceptance/rejection mechanics
pub fn party_invitation_system(
    mut commands: Commands,
    mut invitation_events: EventReader<PartyInvitationEvent>,
    npc_query: Query<&Npc>,
    player_query: Query<&Transform, With<Player>>,
) {
    for event in invitation_events.read() {
        process_party_invitation(&mut commands, event, &npc_query, &player_query);
    }
}

fn process_party_invitation(
    commands: &mut Commands,
    event: &PartyInvitationEvent,
    npc_query: &Query<&Npc>,
    _player_query: &Query<&Transform, With<Player>>,
) {
    if let Ok(npc) = npc_query.get(event.npc_entity) {
        let acceptance_chance = calculate_invitation_acceptance(npc, &event.player_reputation);
        
        if roll_invitation_success(acceptance_chance) {
            accept_party_invitation(commands, event, npc);
        } else {
            reject_party_invitation(npc);
        }
    }
}

fn calculate_invitation_acceptance(npc: &Npc, player_reputation: &f32) -> f32 {
    let base_chance = npc.join_probability;
    let reputation_bonus = (player_reputation * 0.2).clamp(-0.3, 0.3);
    let mood_bonus = (npc.current_mood - 0.5) * 0.2;
    
    (base_chance + reputation_bonus + mood_bonus).clamp(0.0, 1.0)
}

fn roll_invitation_success(acceptance_chance: f32) -> bool {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen::<f32>() < acceptance_chance
}

fn accept_party_invitation(commands: &mut Commands, event: &PartyInvitationEvent, npc: &Npc) {
    commands.entity(event.npc_entity).insert(PartyMember {
        leader: event.player_entity,
        follow_distance: 50.0,
    });
    
    info!("🎉 {} accepted your party invitation!", npc.name);
}

fn reject_party_invitation(npc: &Npc) {
    info!("😔 {} declined your party invitation.", npc.name);
}

// ===== NPC AI BEHAVIOR =====

/// System to handle basic NPC AI behaviors
pub fn npc_behavior_system(
    time: Res<Time>,
    mut npc_query: Query<(&mut Transform, &mut NpcBehavior), (With<Npc>, Without<Player>)>,
) {
    for (mut transform, mut behavior) in npc_query.iter_mut() {
        update_npc_behavior(&time, &mut transform, &mut behavior);
    }
}

fn update_npc_behavior(
    time: &Res<Time>,
    transform: &mut Transform,
    behavior: &mut NpcBehavior,
) {
    behavior.last_action_time += time.delta_seconds();
    
    if behavior.last_action_time >= behavior.action_cooldown {
        execute_npc_behavior(transform, behavior);
        behavior.last_action_time = 0.0;
    }
}

fn execute_npc_behavior(transform: &mut Transform, behavior: &mut NpcBehavior) {
    match behavior.behavior_type {
        NpcBehaviorType::Wandering => execute_wandering_behavior(transform, behavior),
        NpcBehaviorType::Stationary => {}, // Do nothing
        NpcBehaviorType::Following => {}, // Would follow party leader
        NpcBehaviorType::Resting => {}, // Maybe play rest animation
    }
}

fn execute_wandering_behavior(transform: &mut Transform, behavior: &NpcBehavior) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    let distance = rng.gen_range(10.0..30.0);
    
    let new_x = behavior.home_position.x + angle.cos() * distance;
    let new_y = behavior.home_position.y + angle.sin() * distance;
    
    // Simple movement toward new position
    let target = Vec3::new(new_x, new_y, transform.translation.z);
    let direction = (target - transform.translation).normalize_or_zero();
    
    transform.translation += direction * 20.0; // Move slowly
}

// ===== NPC SPAWNING =====

/// System to spawn NPCs in the world during level loading
pub fn spawn_npcs_system(mut commands: Commands) {
    spawn_mountaineering_npcs(&mut commands);
}

fn spawn_mountaineering_npcs(commands: &mut Commands) {
    spawn_experienced_guide(commands);
    spawn_fellow_climber(commands);
    spawn_mountain_hermit(commands);
}

fn spawn_experienced_guide(commands: &mut Commands) {
    let guide_dialogue = create_guide_dialogue();
    let spawn_position = Vec3::new(100.0, 200.0, 1.0);
    
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.3, 0.6, 0.9), // Blue for guide
                custom_size: Some(Vec2::new(24.0, 32.0)),
                ..default()
            },
            transform: Transform::from_translation(spawn_position),
            ..default()
        },
        Npc {
            name: "Erik the Guide".to_string(),
            npc_type: NPCType::Guide,
            dialogue_tree: "guide_basic".to_string(),
            join_probability: 0.7,
            reputation_modifier: 0.0,
            current_mood: 0.8,
        },
        DialogueTree {
            current_node: "greeting".to_string(),
            nodes: guide_dialogue,
        },
        ConversationRange { distance: 60.0 },
        NpcBehavior {
            behavior_type: NpcBehaviorType::Stationary,
            last_action_time: 0.0,
            action_cooldown: 5.0,
            wander_radius: 50.0,
            home_position: spawn_position,
        },
    ));
}

fn spawn_fellow_climber(commands: &mut Commands) {
    let climber_dialogue = create_climber_dialogue();
    let spawn_position = Vec3::new(-150.0, 150.0, 1.0);
    
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.8, 0.4, 0.2), // Orange for climber
                custom_size: Some(Vec2::new(24.0, 32.0)),
                ..default()
            },
            transform: Transform::from_translation(spawn_position),
            ..default()
        },
        Npc {
            name: "Astrid".to_string(),
            npc_type: NPCType::Climber,
            dialogue_tree: "climber_basic".to_string(),
            join_probability: 0.5,
            reputation_modifier: 0.1,
            current_mood: 0.6,
        },
        DialogueTree {
            current_node: "greeting".to_string(),
            nodes: climber_dialogue,
        },
        ConversationRange { distance: 60.0 },
        NpcBehavior {
            behavior_type: NpcBehaviorType::Wandering,
            last_action_time: 0.0,
            action_cooldown: 8.0,
            wander_radius: 80.0,
            home_position: spawn_position,
        },
    ));
}

fn spawn_mountain_hermit(commands: &mut Commands) {
    let hermit_dialogue = create_hermit_dialogue();
    let spawn_position = Vec3::new(200.0, -100.0, 1.0);
    
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.5, 0.3, 0.6), // Purple for hermit
                custom_size: Some(Vec2::new(24.0, 32.0)),
                ..default()
            },
            transform: Transform::from_translation(spawn_position),
            ..default()
        },
        Npc {
            name: "Old Magnus".to_string(),
            npc_type: NPCType::Hermit,
            dialogue_tree: "hermit_basic".to_string(),
            join_probability: 0.2,
            reputation_modifier: -0.1,
            current_mood: 0.4,
        },
        DialogueTree {
            current_node: "greeting".to_string(),
            nodes: hermit_dialogue,
        },
        ConversationRange { distance: 50.0 },
        NpcBehavior {
            behavior_type: NpcBehaviorType::Stationary,
            last_action_time: 0.0,
            action_cooldown: 10.0,
            wander_radius: 20.0,
            home_position: spawn_position,
        },
    ));
}

// ===== DIALOGUE CONTENT CREATION =====

fn create_guide_dialogue() -> std::collections::HashMap<String, DialogueNode> {
    let mut nodes = std::collections::HashMap::new();
    
    nodes.insert("greeting".to_string(), DialogueNode {
        text: "Greetings, fellow climber! I'm Erik, been guiding these mountains for 20 years.".to_string(),
        speaker: "Erik the Guide".to_string(),
        options: vec![
            DialogueOption {
                text: "I could use some guidance on these peaks.".to_string(),
                next_node: "offer_help".to_string(),
                requirements: vec![],
            },
            DialogueOption {
                text: "Want to join my climbing party?".to_string(),
                next_node: "party_invite".to_string(),
                requirements: vec![],
            },
            DialogueOption {
                text: "Just passing through.".to_string(),
                next_node: "end".to_string(),
                requirements: vec![],
            },
        ],
        effects: vec![],
    });
    
    nodes.insert("offer_help".to_string(), DialogueNode {
        text: "The weather's been harsh lately. Ice axes are essential for the glacier sections.".to_string(),
        speaker: "Erik the Guide".to_string(),
        options: vec![
            DialogueOption {
                text: "Thanks for the advice!".to_string(),
                next_node: "end".to_string(),
                requirements: vec![],
            },
        ],
        effects: vec![DialogueEffect::ChangeReputation(0.1)],
    });
    
    nodes.insert("party_invite".to_string(), DialogueNode {
        text: "I'd be honored to join your expedition! These mountains are safer with company.".to_string(),
        speaker: "Erik the Guide".to_string(),
        options: vec![
            DialogueOption {
                text: "Welcome to the team!".to_string(),
                next_node: "end".to_string(),
                requirements: vec![],
            },
        ],
        effects: vec![DialogueEffect::InviteToParty],
    });
    
    nodes
}

fn create_climber_dialogue() -> std::collections::HashMap<String, DialogueNode> {
    let mut nodes = std::collections::HashMap::new();
    
    nodes.insert("greeting".to_string(), DialogueNode {
        text: "Hey there! I'm Astrid. Been climbing solo, but these peaks are challenging.".to_string(),
        speaker: "Astrid".to_string(),
        options: vec![
            DialogueOption {
                text: "How's the climb been?".to_string(),
                next_node: "climbing_talk".to_string(),
                requirements: vec![],
            },
            DialogueOption {
                text: "Want to team up?".to_string(),
                next_node: "party_invite".to_string(),
                requirements: vec![],
            },
        ],
        effects: vec![],
    });
    
    nodes.insert("climbing_talk".to_string(), DialogueNode {
        text: "Tough but rewarding! The ice sections require good technique.".to_string(),
        speaker: "Astrid".to_string(),
        options: vec![
            DialogueOption {
                text: "Good luck with your climb!".to_string(),
                next_node: "end".to_string(),
                requirements: vec![],
            },
        ],
        effects: vec![],
    });
    
    nodes.insert("party_invite".to_string(), DialogueNode {
        text: "That sounds great! Safety in numbers, right?".to_string(),
        speaker: "Astrid".to_string(),
        options: vec![
            DialogueOption {
                text: "Exactly! Let's climb together.".to_string(),
                next_node: "end".to_string(),
                requirements: vec![],
            },
        ],
        effects: vec![DialogueEffect::InviteToParty],
    });
    
    nodes
}

fn create_hermit_dialogue() -> std::collections::HashMap<String, DialogueNode> {
    let mut nodes = std::collections::HashMap::new();
    
    nodes.insert("greeting".to_string(), DialogueNode {
        text: "Hmph. Another climber disturbing my solitude. I'm Magnus.".to_string(),
        speaker: "Old Magnus".to_string(),
        options: vec![
            DialogueOption {
                text: "Sorry to bother you.".to_string(),
                next_node: "respectful".to_string(),
                requirements: vec![],
            },
            DialogueOption {
                text: "Join my party?".to_string(),
                next_node: "party_invite".to_string(),
                requirements: vec![],
            },
        ],
        effects: vec![],
    });
    
    nodes.insert("respectful".to_string(), DialogueNode {
        text: "Hmm, at least you have manners. These mountains teach respect.".to_string(),
        speaker: "Old Magnus".to_string(),
        options: vec![
            DialogueOption {
                text: "I'll leave you in peace.".to_string(),
                next_node: "end".to_string(),
                requirements: vec![],
            },
        ],
        effects: vec![DialogueEffect::ChangeReputation(0.05)],
    });
    
    nodes.insert("party_invite".to_string(), DialogueNode {
        text: "Bah! I climb alone. Too old for your foolishness.".to_string(),
        speaker: "Old Magnus".to_string(),
        options: vec![
            DialogueOption {
                text: "Understood.".to_string(),
                next_node: "end".to_string(),
                requirements: vec![],
            },
        ],
        effects: vec![DialogueEffect::ChangeReputation(-0.1)],
    });
    
    nodes
}

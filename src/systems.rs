use crate::components::*;
use crate::resources::*;
use crate::states::*;
use bevy::prelude::*;

// Type aliases to fix clippy::type_complexity warnings
type TerrainQuery<'w, 's> =
    Query<'w, 's, (&'static Transform, &'static TerrainTile), (With<TerrainTile>, Without<Player>)>;
type CloseButtonQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static mut BackgroundColor),
    (Changed<Interaction>, With<CloseButton>),
>;

/// Stamina and health regeneration system with feedback
pub fn stamina_regeneration_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_query: Query<(&mut MovementStats, &mut Health), With<Player>>,
    mut last_regen_log: Local<f32>,
) {
    for (mut stats, mut health) in player_query.iter_mut() {
        // Check if player is moving (any arrow key pressed)
        let is_moving = keyboard_input.pressed(KeyCode::ArrowUp)
            || keyboard_input.pressed(KeyCode::ArrowDown)
            || keyboard_input.pressed(KeyCode::ArrowLeft)
            || keyboard_input.pressed(KeyCode::ArrowRight);

        if !is_moving {
            let old_stamina = stats.stamina;
            let old_health = health.current;

            // Regenerate stamina when not moving
            let stamina_regen_rate = 15.0; // Stamina per second when resting
            stats.stamina =
                (stats.stamina + stamina_regen_rate * time.delta_seconds()).min(stats.max_stamina);

            // Slow health regeneration when resting (if not in harsh conditions)
            let health_regen_rate = 2.0; // Health per second when resting
            health.current =
                (health.current + health_regen_rate * time.delta_seconds()).min(health.max);

            // Log regeneration periodically (every 3 seconds)
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

        // Death check
        if health.current <= 0.0 {
            error!("💀 Player has died! Health reached zero.");
            // In a real game, you'd transition to a death/game over state here
        }
    }
}

// ===== PHASE 2: TERRAIN LOADING FROM FILES =====

/// System to load and spawn terrain from level files
pub fn load_terrain_from_level(mut commands: Commands) {
    // Load the tutorial level
    let level_path = "levels/tutorial_01.ron";

    match crate::levels::LevelDefinition::load_from_file(level_path) {
        Ok(level) => {
            info!("Loading level: {}", level.name);

            // Spawn terrain tiles from level data
            for (row_idx, row) in level.terrain.iter().enumerate() {
                for (col_idx, terrain_data) in row.iter().enumerate() {
                    let color = get_terrain_color(&terrain_data.terrain_type);

                    // Override climbable for soil terrain - should always be passable
                    let climbable = match terrain_data.terrain_type {
                        TerrainType::Soil => true,  // Always allow movement through soil/brown terrain
                        _ => terrain_data.climbable, // Use level file setting for other terrain
                    };

                    commands.spawn((
                        SpriteBundle {
                            sprite: Sprite {
                                color,
                                custom_size: Some(Vec2::new(32.0, 32.0)),
                                ..default()
                            },
                            transform: Transform::from_translation(Vec3::new(
                                (col_idx as f32 - level.width as f32 / 2.0) * 32.0,
                                (level.height as f32 / 2.0 - row_idx as f32) * 32.0,
                                0.0,
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
            }

            info!(
                "Terrain loaded from {}: {}x{} tiles",
                level_path, level.width, level.height
            );
            info!("Terrain types: Brown=soil, Gray=rock, Blue=ice, Green=grass, White=snow");
        }
        Err(e) => {
            error!("Failed to load level {}: {}", level_path, e);
            // Fallback to simple terrain for testing
            spawn_simple_fallback_terrain(&mut commands);
        }
    }
}

/// Helper function to get color for terrain type
fn get_terrain_color(terrain_type: &TerrainType) -> Color {
    match terrain_type {
        TerrainType::Soil => Color::srgb(0.6, 0.4, 0.2), // Brown
        TerrainType::Rock => Color::srgb(0.5, 0.5, 0.5), // Gray
        TerrainType::Ice => Color::srgb(0.6, 0.8, 1.0),  // Light blue
        TerrainType::Snow => Color::srgb(0.9, 0.9, 0.9), // White
        TerrainType::Grass => Color::srgb(0.2, 0.7, 0.2), // Green
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

/// Enhanced movement system with better stamina feedback
pub fn player_movement_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_query: Query<(&mut Transform, &mut MovementStats, &mut Health), With<Player>>,
    terrain_query: TerrainQuery,
) {
    for (mut player_transform, mut stats, mut health) in player_query.iter_mut() {
        let direction = get_movement_direction(&keyboard_input);
        
        if direction.length() > 0.0 {
            handle_player_movement(
                &mut player_transform,
                &mut stats,
                direction,
                &terrain_query,
                &time,
            );
        }

        // Health effects from environmental hazards
        apply_environmental_effects(
            &mut health,
            player_transform.translation,
            &terrain_query,
            time.delta_seconds(),
        );
    }
}

fn get_movement_direction(keyboard_input: &Res<ButtonInput<KeyCode>>) -> Vec3 {
    let mut direction = Vec3::ZERO;

    // Arrow key controls
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    direction
}

fn handle_player_movement(
    player_transform: &mut Transform,
    stats: &mut MovementStats,
    direction: Vec3,
    terrain_query: &TerrainQuery,
    time: &Res<Time>,
) {
    // Check if player has enough stamina to move
    if stats.stamina <= 5.0 {
        warn!(
            "❌ Too exhausted to move! Stamina: {:.1}/100 - Rest to recover!",
            stats.stamina
        );
        return;
    }

    let normalized_direction = direction.normalize();
    let terrain_modifier = get_terrain_modifier_at_position(player_transform.translation, terrain_query);
    
    // Calculate and consume stamina
    let stamina_cost = calculate_movement_stamina_cost(
        player_transform.translation,
        terrain_query,
        time.delta_seconds(),
    );
    
    stats.stamina = (stats.stamina - stamina_cost).max(0.0);
    
    // Log movement feedback
    log_movement_feedback(stats.stamina, stamina_cost);
    
    // Calculate and apply movement
    let movement = normalized_direction * stats.speed * terrain_modifier * time.delta_seconds();
    let new_position = player_transform.translation + movement;

    // Check collision and move if valid
    if can_move_to_position(new_position, terrain_query) {
        player_transform.translation = new_position;
    }
}

fn calculate_movement_stamina_cost(
    position: Vec3,
    terrain_query: &TerrainQuery,
    delta_time: f32,
) -> f32 {
    let stamina_cost = get_stamina_cost_for_terrain(position, terrain_query, delta_time);
    stamina_cost
}

fn log_movement_feedback(stamina: f32, stamina_cost: f32) {
    if stamina_cost > 0.0 {
        info!("🏃 Moving! Stamina: {:.1}/100 (-{:.1})", stamina, stamina_cost);
    }

    if stamina < 20.0 && stamina > 0.0 {
        warn!("⚠️ Stamina low: {:.1}/100", stamina);
    }
}

/// Check if the player can move to a specific position (collision detection)
fn can_move_to_position(position: Vec3, terrain_query: &TerrainQuery) -> bool {
    let player_size = 16.0; // Half the player size for collision
    let tile_size = 16.0; // Half the tile size

    for (terrain_transform, terrain_tile) in terrain_query.iter() {
        // Only check collision with climbable terrain (non-climbable = solid walls)
        if !terrain_tile.climbable {
            // Simple AABB collision detection
            let distance = position.distance(terrain_transform.translation);
            if distance < (player_size + tile_size) {
                return false; // Collision detected
            }
        }
    }
    true // No collision
}

/// Calculate stamina cost based on terrain difficulty
fn get_stamina_cost_for_terrain(
    position: Vec3,
    terrain_query: &TerrainQuery,
    delta_time: f32,
) -> f32 {
    let detection_range = 20.0;
    let base_stamina_cost = 8.0; // Stamina per second while moving

    for (terrain_transform, terrain_tile) in terrain_query.iter() {
        let distance = position.distance(terrain_transform.translation);
        if distance < detection_range {
            // Different terrain types cost different amounts of stamina
            let terrain_multiplier = match terrain_tile.terrain_type {
                TerrainType::Grass => 0.8, // Easy terrain
                TerrainType::Soil => 1.0,  // Normal stamina cost
                TerrainType::Rock => 1.8,  // Very exhausting
                TerrainType::Ice => 1.2,   // Slippery but requires concentration
                TerrainType::Snow => 2.0,  // Most exhausting
            };

            return base_stamina_cost * terrain_multiplier * delta_time;
        }
    }

    base_stamina_cost * delta_time // Default cost
}

/// Apply environmental effects to health with detailed feedback
fn apply_environmental_effects(
    health: &mut Health,
    position: Vec3,
    terrain_query: &TerrainQuery,
    delta_time: f32,
) {
    let detection_range = 20.0;

    for (terrain_transform, terrain_tile) in terrain_query.iter() {
        let distance = position.distance(terrain_transform.translation);
        if distance < detection_range {
            // Apply terrain-specific health effects
            match terrain_tile.terrain_type {
                TerrainType::Ice => {
                    // Ice can cause minor health loss from cold
                    let cold_damage = 1.5 * delta_time;
                    let old_health = health.current;
                    health.current = (health.current - cold_damage).max(0.0);
                    if health.current < old_health {
                        warn!(
                            "🧊 Taking cold damage on ice! Health: {:.1}/100 (-{:.1})",
                            health.current, cold_damage
                        );
                    }
                }
                TerrainType::Snow => {
                    // Snow is even colder
                    let cold_damage = 3.0 * delta_time;
                    let old_health = health.current;
                    health.current = (health.current - cold_damage).max(0.0);
                    if health.current < old_health {
                        error!(
                            "❄️ Freezing in snow! Health: {:.1}/100 (-{:.1})",
                            health.current, cold_damage
                        );
                    }
                }
                TerrainType::Grass => {
                    // Grass is healing
                    let healing = 0.5 * delta_time;
                    let old_health = health.current;
                    health.current = (health.current + healing).min(health.max);
                    if health.current > old_health && health.current < health.max {
                        info!(
                            "🌱 Grass is healing you! Health: {:.1}/100 (+{:.1})",
                            health.current, healing
                        );
                    }
                }
                TerrainType::Soil | TerrainType::Rock => {
                    // Other terrain types don't affect health
                }
            }
            break;
        }
    }
}

/// Get movement speed modifier based on terrain at current position
fn get_terrain_modifier_at_position(position: Vec3, terrain_query: &TerrainQuery) -> f32 {
    let detection_range = 20.0; // How close to terrain center to apply modifier

    for (terrain_transform, terrain_tile) in terrain_query.iter() {
        let distance = position.distance(terrain_transform.translation);
        if distance < detection_range {
            // Apply terrain-specific movement modifiers
            return match terrain_tile.terrain_type {
                TerrainType::Soil => 1.0,  // Normal speed
                TerrainType::Grass => 1.1, // Slightly faster on grass
                TerrainType::Rock => 0.7,  // Slower on rock
                TerrainType::Ice => 1.4,   // Faster/slippery on ice
                TerrainType::Snow => 0.6,  // Much slower in snow
            };
        }
    }
    1.0 // Default speed if not on any specific terrain
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

pub fn climbing_movement_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut MovementStats), With<Player>>,
    time: Res<Time>,
) {
    for (mut transform, mut stats) in query.iter_mut() {
        let movement = get_climbing_movement(&keys);
        
        if movement.length() > 0.0 {
            handle_climbing_movement(&mut transform, &mut stats, movement, &time);
        } else {
            handle_climbing_rest(&mut stats, &time);
        }

        // Prevent upward movement if out of stamina
        enforce_stamina_limits(&mut stats, movement);
    }
}

fn get_climbing_movement(keys: &Res<ButtonInput<KeyCode>>) -> Vec3 {
    let mut movement = Vec3::ZERO;

    if keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA) {
        movement.x -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD) {
        movement.x += 1.0;
    }
    if keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyW) {
        movement.y += 1.0;
    }
    if keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyS) {
        movement.y -= 1.0;
    }

    movement
}

fn handle_climbing_movement(
    transform: &mut Transform,
    stats: &mut MovementStats,
    movement: Vec3,
    time: &Res<Time>,
) {
    let normalized_movement = movement.normalize();
    transform.translation += normalized_movement * stats.speed * time.delta_seconds();

    // Climbing up uses stamina
    if movement.y > 0.0 {
        stats.stamina -= 20.0 * time.delta_seconds();
    }

    // Regenerate stamina when not climbing up
    if movement.y <= 0.0 {
        stats.stamina = (stats.stamina + 10.0 * time.delta_seconds()).min(stats.max_stamina);
    }
}

fn handle_climbing_rest(stats: &mut MovementStats, time: &Res<Time>) {
    // Faster stamina regen when not moving
    stats.stamina = (stats.stamina + 15.0 * time.delta_seconds()).min(stats.max_stamina);
}

fn enforce_stamina_limits(stats: &mut MovementStats, movement: Vec3) {
    // Prevent movement if out of stamina (this would need to be applied to transform in real system)
    if stats.stamina <= 0.0 && movement.y > 0.0 {
        // In a real implementation, we'd prevent the upward movement here
        // For now, just ensure stamina doesn't go negative
        stats.stamina = 0.0;
    }
}

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

        // Create starting items
        let ice_axe = Item {
            id: "ice_axe_01".to_string(),
            name: "Ice Axe".to_string(),
            weight: 1.5,
            item_type: ItemType::ClimbingGear,
            durability: Some(100.0),
            properties: ItemProperties {
                strength: Some(15.0), // +15% climbing ability
                warmth: None,
                magic_power: None,
                nutrition: None,
                water: None,
                protection: Some(5.0),
            },
        };

        let heavy_boots = Item {
            id: "heavy_boots_01".to_string(),
            name: "Heavy Climbing Boots".to_string(),
            weight: 3.0,
            item_type: ItemType::Clothing,
            durability: Some(100.0),
            properties: ItemProperties {
                strength: Some(10.0), // +10% climbing ability
                warmth: Some(20.0),   // Cold protection
                magic_power: None,
                nutrition: None,
                water: None,
                protection: Some(15.0),
            },
        };

        let wool_jacket = Item {
            id: "wool_jacket_01".to_string(),
            name: "Wool Jacket".to_string(),
            weight: 2.0,
            item_type: ItemType::Clothing,
            durability: Some(100.0),
            properties: ItemProperties {
                strength: None,
                warmth: Some(30.0), // Good cold protection
                magic_power: None,
                nutrition: None,
                water: None,
                protection: Some(10.0),
            },
        };

        // Update inventory with starting items
        inventory.items = vec![ice_axe.clone(), heavy_boots.clone(), wool_jacket.clone()];
        inventory.current_weight = ice_axe.weight + heavy_boots.weight + wool_jacket.weight;

        // Equip items
        equipped.axe = Some(ice_axe);
        equipped.boots = Some(heavy_boots);
        equipped.jacket = Some(wool_jacket);

        info!("🎒 Starting equipment loaded: Ice Axe (+15% climb), Heavy Boots (+10% climb, +20 warmth), Wool Jacket (+30 warmth)");
    } else {
        warn!("⚠️ Could not find player entity to add starting equipment!");
    }
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
        handle_ice_axe_usage(
            player_transform,
            inventory,
            &mut commands,
            &mut terrain_query,
            &mut terrain_broken_events,
        );
    }
}

fn handle_ice_axe_usage(
    player_transform: &Transform,
    inventory: &Inventory,
    commands: &mut Commands,
    terrain_query: &mut Query<(Entity, &Transform, &mut TerrainTile, Option<&mut Breakable>)>,
    terrain_broken_events: &mut EventWriter<TerrainBrokenEvent>,
) {
    // Check if player has ice axe equipped or in inventory
    if get_ice_axe_from_inventory(inventory).is_none() {
        warn!("❌ No ice axe available! Check your inventory or equipped items.");
        return;
    }

    // Find ice terrain within reach and attempt to break it
    let reach_distance = 40.0;
    for (terrain_entity, terrain_transform, mut terrain_tile, breakable) in terrain_query.iter_mut() {
        if should_break_terrain(player_transform, terrain_transform, &terrain_tile, reach_distance) {
            break_terrain_with_axe(
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

fn should_break_terrain(
    player_transform: &Transform,
    terrain_transform: &Transform,
    terrain_tile: &TerrainTile,
    reach_distance: f32,
) -> bool {
    let distance = player_transform.translation.distance(terrain_transform.translation);
    distance <= reach_distance && terrain_tile.terrain_type == TerrainType::Ice
}

fn break_terrain_with_axe(
    commands: &mut Commands,
    terrain_entity: Entity,
    terrain_tile: &mut TerrainTile,
    breakable: Option<Mut<Breakable>>,
    position: Vec3,
    terrain_broken_events: &mut EventWriter<TerrainBrokenEvent>,
) {
    if let Some(mut breakable_comp) = breakable {
        break_ice_terrain(
            commands,
            terrain_entity,
            terrain_tile,
            &mut breakable_comp,
            position,
            terrain_broken_events,
        );
    } else {
        // Add Breakable component to ice terrain
        commands.entity(terrain_entity).insert(Breakable {
            tool_required: ToolType::IceAxe,
            durability: 50.0,
            max_durability: 50.0,
        });
    }
}

/// Helper function to get ice axe from inventory or equipped items
fn get_ice_axe_from_inventory(inventory: &Inventory) -> Option<&Item> {
    // Check inventory for ice axe
    inventory.items.iter().find(|item| {
        item.name.to_lowercase().contains("ice axe") || item.name.to_lowercase().contains("axe")
    })
}

/// Break ice terrain with ice axe
fn break_ice_terrain(
    commands: &mut Commands,
    terrain_entity: Entity,
    terrain_tile: &mut TerrainTile,
    breakable: &mut Breakable,
    position: Vec3,
    terrain_broken_events: &mut EventWriter<TerrainBrokenEvent>,
) {
    // Reduce terrain durability
    let damage = 25.0; // Damage per axe hit
    breakable.durability = (breakable.durability - damage).max(0.0);

    info!(
        "🪓 Breaking ice terrain! Durability: {:.1}/{:.1}",
        breakable.durability, breakable.max_durability
    );

    // If terrain is broken, change it to passable
    if breakable.durability <= 0.0 {
        terrain_tile.terrain_type = TerrainType::Soil; // Convert to passable terrain
        terrain_tile.climbable = true;

        // Send terrain broken event
        terrain_broken_events.send(TerrainBrokenEvent {
            position,
            terrain_type: TerrainType::Ice,
            tool_used: ToolType::IceAxe,
        });

        // Remove Breakable component as terrain is now broken
        commands.entity(terrain_entity).remove::<Breakable>();

        info!("✅ Ice terrain broken! Path is now clear.");
    }
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

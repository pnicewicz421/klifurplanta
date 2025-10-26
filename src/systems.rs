use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::states::*;

/// Stamina and health regeneration system
pub fn stamina_regeneration_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_query: Query<(&mut MovementStats, &mut Health), With<Player>>,
) {
    for (mut stats, mut health) in player_query.iter_mut() {
        // Check if player is moving (any arrow key pressed)
        let is_moving = keyboard_input.pressed(KeyCode::ArrowUp) ||
                       keyboard_input.pressed(KeyCode::ArrowDown) ||
                       keyboard_input.pressed(KeyCode::ArrowLeft) ||
                       keyboard_input.pressed(KeyCode::ArrowRight);
        
        if !is_moving {
            // Regenerate stamina when not moving
            let stamina_regen_rate = 15.0; // Stamina per second when resting
            stats.stamina = (stats.stamina + stamina_regen_rate * time.delta_seconds()).min(stats.max_stamina);
            
            // Slow health regeneration when resting (if not in harsh conditions)
            let health_regen_rate = 2.0; // Health per second when resting
            health.current = (health.current + health_regen_rate * time.delta_seconds()).min(health.max);
        }
        
        // Death check
        if health.current <= 0.0 {
            error!("Player has died! Health reached zero.");
            // In a real game, you'd transition to a death/game over state here
        }
    }
}

// ===== PHASE 2: TERRAIN LOADING FROM FILES =====

/// System to load and spawn terrain from level files
pub fn load_terrain_from_level(
    mut commands: Commands,
) {
    // Load the tutorial level
    let level_path = "levels/tutorial_01.ron";
    
    match crate::levels::LevelDefinition::load_from_file(level_path) {
        Ok(level) => {
            info!("Loading level: {}", level.name);
            
            // Spawn terrain tiles from level data
            for (row_idx, row) in level.terrain.iter().enumerate() {
                for (col_idx, terrain_data) in row.iter().enumerate() {
                    let color = get_terrain_color(&terrain_data.terrain_type);
                    
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
                            climbable: terrain_data.climbable,
                        },
                    ));
                }
            }
            
            info!("Terrain loaded from {}: {}x{} tiles", level_path, level.width, level.height);
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
        TerrainType::Soil => Color::srgb(0.6, 0.4, 0.2),     // Brown
        TerrainType::Rock => Color::srgb(0.5, 0.5, 0.5),     // Gray  
        TerrainType::Ice => Color::srgb(0.6, 0.8, 1.0),      // Light blue
        TerrainType::Snow => Color::srgb(0.9, 0.9, 0.9),     // White
        TerrainType::Grass => Color::srgb(0.2, 0.7, 0.2),    // Green
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
                    climbable: matches!(terrain_type, TerrainType::Grass),
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
        (camera_query.get_single_mut(), player_query.get_single()) {
        
        let target_position = Vec3::new(
            player_transform.translation.x,
            player_transform.translation.y,
            camera_transform.translation.z, // Keep camera's Z position
        );
        
        // Smooth camera following with lerp
        let follow_speed = 2.0;
        camera_transform.translation = camera_transform.translation.lerp(
            target_position,
            follow_speed * time.delta_seconds(),
        );
    }
}

// ===== PHASE 1: BASIC PLAYER MOVEMENT =====

/// Advanced player movement system with stamina consumption and health management
pub fn player_movement_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_query: Query<(&mut Transform, &mut MovementStats, &mut Health), With<Player>>,
    terrain_query: Query<(&Transform, &TerrainTile), (With<TerrainTile>, Without<Player>)>,
) {
    for (mut player_transform, mut stats, mut health) in player_query.iter_mut() {
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

        // Calculate movement with terrain-specific speed modifiers and stamina
        if direction.length() > 0.0 {
            // Check if player has enough stamina to move
            if stats.stamina > 5.0 {
                direction = direction.normalize();
                
                // Get terrain modifier for current position
                let terrain_modifier = get_terrain_modifier_at_position(
                    player_transform.translation, 
                    &terrain_query
                );
                
                // Calculate stamina cost based on terrain difficulty
                let stamina_cost = get_stamina_cost_for_terrain(
                    player_transform.translation,
                    &terrain_query,
                    time.delta_seconds()
                );
                
                // Consume stamina
                stats.stamina = (stats.stamina - stamina_cost).max(0.0);
                
                let movement = direction * stats.speed * terrain_modifier * time.delta_seconds();
                let new_position = player_transform.translation + movement;
                
                // Check collision with solid terrain
                if can_move_to_position(new_position, &terrain_query) {
                    player_transform.translation = new_position;
                }
                
                // Log stamina if getting low
                if stats.stamina < 20.0 && stats.stamina > 0.0 {
                    info!("Stamina low: {:.1}/100", stats.stamina);
                }
            } else {
                info!("Too exhausted to move! Stamina: {:.1}/100", stats.stamina);
            }
        }
        
        // Health effects from environmental hazards
        apply_environmental_effects(&mut health, player_transform.translation, &terrain_query, time.delta_seconds());
    }
}

/// Check if the player can move to a specific position (collision detection)
fn can_move_to_position(
    position: Vec3,
    terrain_query: &Query<(&Transform, &TerrainTile), (With<TerrainTile>, Without<Player>)>,
) -> bool {
    let player_size = 16.0; // Half the player size for collision
    let tile_size = 16.0;   // Half the tile size
    
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
    terrain_query: &Query<(&Transform, &TerrainTile), (With<TerrainTile>, Without<Player>)>,
    delta_time: f32,
) -> f32 {
    let detection_range = 20.0;
    let base_stamina_cost = 8.0; // Stamina per second while moving
    
    for (terrain_transform, terrain_tile) in terrain_query.iter() {
        let distance = position.distance(terrain_transform.translation);
        if distance < detection_range {
            // Different terrain types cost different amounts of stamina
            let terrain_multiplier = match terrain_tile.terrain_type {
                TerrainType::Grass => 0.8,   // Easy terrain
                TerrainType::Soil => 1.0,    // Normal stamina cost
                TerrainType::Rock => 1.8,    // Very exhausting
                TerrainType::Ice => 1.2,     // Slippery but requires concentration
                TerrainType::Snow => 2.0,    // Most exhausting
            };
            
            return base_stamina_cost * terrain_multiplier * delta_time;
        }
    }
    
    base_stamina_cost * delta_time // Default cost
}

/// Apply environmental effects to health
fn apply_environmental_effects(
    health: &mut Health,
    position: Vec3,
    terrain_query: &Query<(&Transform, &TerrainTile), (With<TerrainTile>, Without<Player>)>,
    delta_time: f32,
) {
    let detection_range = 20.0;
    
    for (terrain_transform, terrain_tile) in terrain_query.iter() {
        let distance = position.distance(terrain_transform.translation);
        if distance < detection_range {
            // Some terrain types can damage health over time
            match terrain_tile.terrain_type {
                TerrainType::Ice => {
                    // Ice can cause minor health loss from cold
                    let cold_damage = 1.0 * delta_time;
                    health.current = (health.current - cold_damage).max(0.0);
                    if health.current < 50.0 {
                        info!("Feeling cold on ice! Health: {:.1}/100", health.current);
                    }
                }
                TerrainType::Snow => {
                    // Snow is even colder
                    let cold_damage = 2.0 * delta_time;
                    health.current = (health.current - cold_damage).max(0.0);
                    if health.current < 50.0 {
                        info!("Freezing in snow! Health: {:.1}/100", health.current);
                    }
                }
                _ => {
                    // Other terrain types don't damage health
                }
            }
            break;
        }
    }
}

/// Get movement speed modifier based on terrain at current position
fn get_terrain_modifier_at_position(
    position: Vec3,
    terrain_query: &Query<(&Transform, &TerrainTile), (With<TerrainTile>, Without<Player>)>,
) -> f32 {
    let detection_range = 20.0; // How close to terrain center to apply modifier
    
    for (terrain_transform, terrain_tile) in terrain_query.iter() {
        let distance = position.distance(terrain_transform.translation);
        if distance < detection_range {
            // Apply terrain-specific movement modifiers
            return match terrain_tile.terrain_type {
                TerrainType::Soil => 1.0,      // Normal speed
                TerrainType::Grass => 1.1,     // Slightly faster on grass
                TerrainType::Rock => 0.7,      // Slower on rock
                TerrainType::Ice => 1.4,       // Faster/slippery on ice
                TerrainType::Snow => 0.6,      // Much slower in snow
            };
        }
    }
    1.0 // Default speed if not on any specific terrain
}

/// Simple time update system (renamed from time_system)
pub fn update_time(
    time: Res<Time>,
    mut game_time: ResMut<GameTime>,
) {
    game_time.update(time.delta_seconds());
}

// ===== CORE SYSTEMS =====

pub fn time_system(
    time: Res<Time>,
    mut game_time: ResMut<GameTime>,
) {
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
            GameState::Menu => {},
            _ => next_state.set(GameState::Menu),
        }
    }
    
    if keys.just_pressed(KeyCode::KeyI) {
        match current_state.get() {
            GameState::Inventory => next_state.set(GameState::Climbing),
            GameState::Climbing => next_state.set(GameState::Inventory),
            _ => {},
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
    // Simple shop interactions - in a real game you'd have a proper UI
    if keys.just_pressed(KeyCode::Digit1) {
        if let Some(rope_item) = shop.items.get("rope") {
            if inventory.money >= rope_item.price && inventory.can_add_item(&rope_item.item) {
                inventory.money -= rope_item.price;
                inventory.add_item(rope_item.item.clone());
                info!("Bought rope for {}", rope_item.price);
            } else {
                info!("Cannot buy rope - not enough money or space");
            }
        }
    }
    
    if keys.just_pressed(KeyCode::Digit2) {
        if let Some(tent_item) = shop.items.get("tent") {
            if inventory.money >= tent_item.price && inventory.can_add_item(&tent_item.item) {
                inventory.money -= tent_item.price;
                inventory.add_item(tent_item.item.clone());
                info!("Bought tent for {}", tent_item.price);
            } else {
                info!("Cannot buy tent - not enough money or space");
            }
        }
    }
    
    if keys.just_pressed(KeyCode::Enter) {
        next_state.set(GameState::Climbing);
        info!("Leaving shop, starting climb!");
    }
}

pub fn inventory_ui_system(
    inventory: Res<PlayerInventory>,
    game_time: Res<GameTime>,
) {
    // In a real game, this would update UI elements
    // For now, just log every few seconds
    if game_time.real_seconds_elapsed % 3.0 < 0.1 {
        info!("Money: ${:.2}, Weight: {:.1}/{:.1} kg", 
            inventory.money, inventory.current_weight, inventory.max_weight);
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
        let mut movement = Vec3::ZERO;
        
        if keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA) {
            movement.x -= 1.0;
        }
        if keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD) {
            movement.x += 1.0;
        }
        if keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyW) {
            movement.y += 1.0;
            stats.stamina -= 20.0 * time.delta_seconds(); // Climbing uses stamina
        }
        if keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyS) {
            movement.y -= 1.0;
        }
        
        if movement.length() > 0.0 {
            movement = movement.normalize();
            transform.translation += movement * stats.speed * time.delta_seconds();
            
            // Regenerate stamina when not climbing up
            if movement.y <= 0.0 {
                stats.stamina = (stats.stamina + 10.0 * time.delta_seconds()).min(stats.max_stamina);
            }
        } else {
            // Faster stamina regen when not moving
            stats.stamina = (stats.stamina + 15.0 * time.delta_seconds()).min(stats.max_stamina);
        }
        
        // Prevent movement if out of stamina
        if stats.stamina <= 0.0 && movement.y > 0.0 {
            movement.y = 0.0;
        }
    }
}

pub fn terrain_interaction_system(
    player_query: Query<&Transform, With<Player>>,
    terrain_query: Query<&TerrainTile>,
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
            },
            _ => {},
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
            let distance = animal_transform.translation.distance(player_transform.translation);
            
            if distance < wildlife.flee_distance {
                // Animal flees from player
                let flee_direction = (animal_transform.translation - player_transform.translation).normalize();
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
    npc_query: Query<&NPC>,
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
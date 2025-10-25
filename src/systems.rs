use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::states::*;

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
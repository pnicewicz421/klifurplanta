use bevy::prelude::*;
use crate::components::*;
use std::collections::HashMap;

// ===== TIME & WORLD STATE =====

#[derive(Resource, Default)]
pub struct GameTime {
    pub real_seconds_elapsed: f32,
    pub game_hours_elapsed: f32,
    pub hours_per_real_second: f32, // Default: 1 hour per 25 seconds = 0.04
    pub day: u32,
    pub hour: u8, // 0-23
}

impl GameTime {
    pub fn new() -> Self {
        Self {
            real_seconds_elapsed: 0.0,
            game_hours_elapsed: 0.0,
            hours_per_real_second: 1.0 / 25.0, // 1 hour per 25 real seconds
            day: 1,
            hour: 8, // Start at 8 AM
        }
    }

    pub fn update(&mut self, delta_seconds: f32) {
        self.real_seconds_elapsed += delta_seconds;
        let hours_passed = delta_seconds * self.hours_per_real_second;
        self.game_hours_elapsed += hours_passed;
        
        let total_hours = (self.game_hours_elapsed + 8.0) as u32; // Start at 8 AM
        self.day = (total_hours / 24) + 1;
        self.hour = (total_hours % 24) as u8;
    }

    pub fn is_night(&self) -> bool {
        self.hour < 6 || self.hour > 20
    }

    pub fn is_day(&self) -> bool {
        !self.is_night()
    }

    pub fn light_level(&self) -> f32 {
        match self.hour {
            0..=5 => 0.1,    // Deep night
            6..=7 => 0.3,    // Dawn
            8..=17 => 1.0,   // Full day
            18..=19 => 0.7,  // Evening
            20..=23 => 0.2,  // Night
            _ => 0.1,
        }
    }
}

// ===== PLAYER RESOURCES =====

#[derive(Resource, Default)]
pub struct PlayerInventory {
    pub money: f32,
    pub items: Vec<Item>,
    pub max_weight: f32,
    pub current_weight: f32,
}

impl PlayerInventory {
    pub fn new(starting_money: f32, max_weight: f32) -> Self {
        Self {
            money: starting_money,
            items: Vec::new(),
            max_weight,
            current_weight: 0.0,
        }
    }

    pub fn can_add_item(&self, item: &Item) -> bool {
        self.current_weight + item.weight <= self.max_weight
    }

    pub fn add_item(&mut self, item: Item) -> bool {
        if self.can_add_item(&item) {
            self.current_weight += item.weight;
            self.items.push(item);
            true
        } else {
            false
        }
    }

    pub fn remove_item(&mut self, item_id: &str) -> Option<Item> {
        if let Some(pos) = self.items.iter().position(|item| item.id == item_id) {
            let item = self.items.remove(pos);
            self.current_weight -= item.weight;
            Some(item)
        } else {
            None
        }
    }
}

// ===== SHOP SYSTEM =====

#[derive(Resource)]
pub struct ShopInventory {
    pub items: HashMap<String, ShopItem>,
}

#[derive(Clone)]
pub struct ShopItem {
    pub item: Item,
    pub price: f32,
    pub stock: Option<u32>, // None = unlimited
}

impl Default for ShopInventory {
    fn default() -> Self {
        let mut items = HashMap::new();
        
        // Climbing gear
        items.insert("rope".to_string(), ShopItem {
            item: Item {
                id: "rope".to_string(),
                name: "Climbing Rope".to_string(),
                weight: 2.0,
                item_type: ItemType::ClimbingGear,
                durability: Some(100.0),
                properties: ItemProperties {
                    strength: Some(50.0),
                    ..Default::default()
                },
            },
            price: 45.0,
            stock: Some(5),
        });

        items.insert("tent".to_string(), ShopItem {
            item: Item {
                id: "tent".to_string(),
                name: "Weather Tent".to_string(),
                weight: 3.5,
                item_type: ItemType::Shelter,
                durability: Some(80.0),
                properties: ItemProperties {
                    protection: Some(30.0),
                    warmth: Some(25.0),
                    ..Default::default()
                },
            },
            price: 70.0,
            stock: Some(3),
        });

        items.insert("jacket".to_string(), ShopItem {
            item: Item {
                id: "jacket".to_string(),
                name: "Heavy Weather Jacket".to_string(),
                weight: 1.2,
                item_type: ItemType::Clothing,
                durability: Some(90.0),
                properties: ItemProperties {
                    warmth: Some(40.0),
                    protection: Some(15.0),
                    ..Default::default()
                },
            },
            price: 85.0,
            stock: Some(4),
        });

        items.insert("harness".to_string(), ShopItem {
            item: Item {
                id: "harness".to_string(),
                name: "Climbing Harness".to_string(),
                weight: 0.8,
                item_type: ItemType::ClimbingGear,
                durability: Some(95.0),
                properties: ItemProperties {
                    strength: Some(35.0),
                    protection: Some(20.0),
                    ..Default::default()
                },
            },
            price: 55.0,
            stock: Some(6),
        });

        Self { items }
    }
}

// ===== LEVEL MANAGEMENT =====

#[derive(Resource, Default)]
pub struct CurrentLevel {
    pub level_id: String,
    pub terrain_map: Vec<Vec<TerrainTile>>,
    pub width: usize,
    pub height: usize,
    pub start_position: (usize, usize),
    pub goal_positions: Vec<(usize, usize)>,
}

// ===== WEATHER & ENVIRONMENT =====

#[derive(Resource)]
pub struct WeatherSystem {
    pub current_weather: Weather,
    pub temperature: f32, // Celsius
    pub wind_speed: f32,
    pub visibility: f32, // 0.0 to 1.0
    pub weather_change_timer: f32,
}

impl Default for WeatherSystem {
    fn default() -> Self {
        Self {
            current_weather: Weather::Clear,
            temperature: 5.0,
            wind_speed: 10.0,
            visibility: 1.0,
            weather_change_timer: 0.0,
        }
    }
}

// ===== PARTY MANAGEMENT =====

#[derive(Resource, Default)]
pub struct Party {
    pub members: Vec<Entity>,
    pub leader: Option<Entity>,
    pub max_size: usize,
}

impl Party {
    pub fn new(max_size: usize) -> Self {
        Self {
            members: Vec::new(),
            leader: None,
            max_size,
        }
    }

    pub fn add_member(&mut self, entity: Entity) -> bool {
        if self.members.len() < self.max_size && !self.members.contains(&entity) {
            self.members.push(entity);
            if self.leader.is_none() {
                self.leader = Some(entity);
            }
            true
        } else {
            false
        }
    }

    pub fn remove_member(&mut self, entity: Entity) {
        self.members.retain(|&e| e != entity);
        if self.leader == Some(entity) {
            self.leader = self.members.first().copied();
        }
    }
}

// Import Weather from states
use crate::states::Weather;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::components::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LevelDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub width: usize,
    pub height: usize,
    pub terrain: Vec<Vec<TerrainData>>,
    pub start_position: (usize, usize),
    pub goal_positions: Vec<(usize, usize)>,
    pub weather_conditions: WeatherConditions,
    pub wildlife_spawns: Vec<WildlifeSpawn>,
    pub npc_spawns: Vec<NPCSpawn>,
    pub items: Vec<ItemSpawn>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TerrainData {
    pub terrain_type: TerrainType,
    pub slope: f32,
    pub stability: f32,
    pub climbable: bool,
    pub climbing_difficulty: Option<f32>,
    pub required_gear: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WeatherConditions {
    pub base_temperature: f32,
    pub wind_speed: f32,
    pub weather_type: String, // "clear", "snow", "storm", etc.
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WildlifeSpawn {
    pub species: String,
    pub position: (f32, f32),
    pub aggression: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NPCSpawn {
    pub name: String,
    pub npc_type: String,
    pub position: (f32, f32),
    pub dialogue_file: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemSpawn {
    pub item_id: String,
    pub position: (f32, f32),
    pub quantity: u32,
}

impl LevelDefinition {
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let level: LevelDefinition = ron::from_str(&content)?;
        Ok(level)
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = ron::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn spawn_level(&self, commands: &mut Commands) {
        // Spawn terrain tiles
        for (y, row) in self.terrain.iter().enumerate() {
            for (x, terrain_data) in row.iter().enumerate() {
                let mut entity_commands = commands.spawn((
                    Transform::from_translation(Vec3::new(x as f32 * 32.0, y as f32 * 32.0, 0.0)),
                    TerrainTile {
                        terrain_type: terrain_data.terrain_type.clone(),
                        slope: terrain_data.slope,
                        stability: terrain_data.stability,
                        climbable: terrain_data.climbable,
                    },
                ));

                if terrain_data.climbable {
                    entity_commands.insert(Climbable {
                        difficulty: terrain_data.climbing_difficulty.unwrap_or(1.0),
                        required_gear: terrain_data.required_gear.clone(),
                    });
                }
            }
        }

        // Spawn wildlife
        for wildlife_spawn in &self.wildlife_spawns {
            let species = match wildlife_spawn.species.as_str() {
                "bear" => WildlifeSpecies::Bear,
                "puma" => WildlifeSpecies::Puma,
                "wolf" => WildlifeSpecies::Wolf,
                "horse" => WildlifeSpecies::Horse,
                "sheep" => WildlifeSpecies::Sheep,
                _ => WildlifeSpecies::Wolf,
            };

            commands.spawn((
                Transform::from_translation(Vec3::new(wildlife_spawn.position.0, wildlife_spawn.position.1, 1.0)),
                Wildlife {
                    species,
                    aggression: wildlife_spawn.aggression,
                    flee_distance: 100.0,
                    attack_damage: 10.0,
                },
            ));
        }

        // Spawn NPCs
        for npc_spawn in &self.npc_spawns {
            let npc_type = match npc_spawn.npc_type.as_str() {
                "climber" => NPCType::Climber,
                "guide" => NPCType::Guide,
                "trader" => NPCType::Trader,
                "viking" => NPCType::Viking,
                "mage" => NPCType::Mage,
                _ => NPCType::Climber,
            };

            commands.spawn((
                Transform::from_translation(Vec3::new(npc_spawn.position.0, npc_spawn.position.1, 1.0)),
                Npc {
                    name: npc_spawn.name.clone(),
                    npc_type,
                    dialogue_tree: npc_spawn.dialogue_file.clone(),
                    join_probability: 0.2,
                },
                Interactable {
                    interaction_type: InteractionType::Talk,
                    range: 50.0,
                },
            ));
        }
    }
}

// Sample level creation functions
pub fn create_tutorial_level() -> LevelDefinition {
    let width = 20;
    let height = 15;
    let mut terrain = vec![vec![TerrainData {
        terrain_type: TerrainType::Soil,
        slope: 0.0,
        stability: 1.0,
        climbable: false,
        climbing_difficulty: None,
        required_gear: vec![],
    }; width]; height];

    // Create a simple climbing route
    #[allow(clippy::needless_range_loop)]
    for y in 5..12 {
        for x in 8..12 {
            terrain[y][x] = TerrainData {
                terrain_type: TerrainType::Rock,
                slope: 0.6,
                stability: 0.8,
                climbable: true,
                climbing_difficulty: Some(1.0),
                required_gear: vec![],
            };
        }
    }

    // Add some ice sections for variety
    for x in 10..14 {
        terrain[10][x] = TerrainData {
            terrain_type: TerrainType::Ice,
            slope: 0.8,
            stability: 0.6,
            climbable: true,
            climbing_difficulty: Some(2.0),
            required_gear: vec!["ice_axe".to_string()],
        };
    }

    LevelDefinition {
        id: "tutorial_01".to_string(),
        name: "First Steps".to_string(),
        description: "A gentle introduction to mountain climbing".to_string(),
        width,
        height,
        terrain,
        start_position: (2, 2),
        goal_positions: vec![(15, 12)],
        weather_conditions: WeatherConditions {
            base_temperature: 10.0,
            wind_speed: 5.0,
            weather_type: "clear".to_string(),
        },
        wildlife_spawns: vec![
            WildlifeSpawn {
                species: "sheep".to_string(),
                position: (100.0, 150.0),
                aggression: 0.0,
            }
        ],
        npc_spawns: vec![
            NPCSpawn {
                name: "Erik the Guide".to_string(),
                npc_type: "guide".to_string(),
                position: (150.0, 100.0),
                dialogue_file: "erik_guide.ron".to_string(),
            }
        ],
        items: vec![
            ItemSpawn {
                item_id: "rope".to_string(),
                position: (200.0, 80.0),
                quantity: 1,
            }
        ],
    }
}

pub fn create_iceland_glacier_level() -> LevelDefinition {
    let width = 30;
    let height = 25;
    let mut terrain = vec![vec![TerrainData {
        terrain_type: TerrainType::Snow,
        slope: 0.2,
        stability: 0.7,
        climbable: false,
        climbing_difficulty: None,
        required_gear: vec![],
    }; width]; height];

    // Create glacier (jökull) terrain
    #[allow(clippy::needless_range_loop)]
    for y in 10..20 {
        for x in 5..25 {
            terrain[y][x] = TerrainData {
                terrain_type: TerrainType::Ice,
                slope: 0.9,
                stability: 0.5,
                climbable: true,
                climbing_difficulty: Some(4.0),
                required_gear: vec!["ice_axe".to_string(), "crampons".to_string()],
            };
        }
    }

    // Add some crevasses (dangerous areas)
    for x in 12..18 {
        terrain[15][x] = TerrainData {
            terrain_type: TerrainType::Ice,
            slope: 1.0,
            stability: 0.1,
            climbable: true,
            climbing_difficulty: Some(5.0),
            required_gear: vec!["rope".to_string(), "harness".to_string()],
        };
    }

    LevelDefinition {
        id: "iceland_glacier_01".to_string(),
        name: "Vatnajökull Challenge".to_string(),
        description: "Scale the mighty Icelandic glacier with proper gear and Viking courage".to_string(),
        width,
        height,
        terrain,
        start_position: (2, 5),
        goal_positions: vec![(25, 22)],
        weather_conditions: WeatherConditions {
            base_temperature: -15.0,
            wind_speed: 25.0,
            weather_type: "blizzard".to_string(),
        },
        wildlife_spawns: vec![
            WildlifeSpawn {
                species: "wolf".to_string(),
                position: (300.0, 200.0),
                aggression: 0.7,
            },
            WildlifeSpawn {
                species: "horse".to_string(),
                position: (100.0, 100.0),
                aggression: 0.0,
            }
        ],
        npc_spawns: vec![
            NPCSpawn {
                name: "Björn the Viking".to_string(),
                npc_type: "viking".to_string(),
                position: (400.0, 150.0),
                dialogue_file: "bjorn_viking.ron".to_string(),
            },
            NPCSpawn {
                name: "Freydis the Mage".to_string(),
                npc_type: "mage".to_string(),
                position: (500.0, 300.0),
                dialogue_file: "freydis_mage.ron".to_string(),
            }
        ],
        items: vec![
            ItemSpawn {
                item_id: "warm_cloak".to_string(),
                position: (250.0, 180.0),
                quantity: 1,
            },
            ItemSpawn {
                item_id: "rune_stone".to_string(),
                position: (450.0, 250.0),
                quantity: 1,
            }
        ],
    }
}

pub fn save_sample_levels() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all("levels")?;
    
    let tutorial = create_tutorial_level();
    tutorial.save_to_file("levels/tutorial_01.ron")?;
    
    let glacier = create_iceland_glacier_level();
    glacier.save_to_file("levels/iceland_glacier_01.ron")?;
    
    info!("Sample levels saved to levels/ directory");
    Ok(())
}
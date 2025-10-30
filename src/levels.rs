use crate::components::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use rand::prelude::*;

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
                Transform::from_translation(Vec3::new(
                    wildlife_spawn.position.0,
                    wildlife_spawn.position.1,
                    1.0,
                )),
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
                Transform::from_translation(Vec3::new(
                    npc_spawn.position.0,
                    npc_spawn.position.1,
                    1.0,
                )),
                Npc {
                    name: npc_spawn.name.clone(),
                    npc_type,
                    dialogue_tree: npc_spawn.dialogue_file.clone(),
                    join_probability: 0.2,
                    reputation_modifier: 0.0,
                    current_mood: 0.5,
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
    let mut terrain = vec![
        vec![
            TerrainData {
                terrain_type: TerrainType::Soil,
                slope: 0.0,
                stability: 1.0,
                climbable: false,
                climbing_difficulty: None,
                required_gear: vec![],
            };
            width
        ];
        height
    ];

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
        wildlife_spawns: vec![WildlifeSpawn {
            species: "sheep".to_string(),
            position: (100.0, 150.0),
            aggression: 0.0,
        }],
        npc_spawns: vec![NPCSpawn {
            name: "Erik the Guide".to_string(),
            npc_type: "guide".to_string(),
            position: (150.0, 100.0),
            dialogue_file: "erik_guide.ron".to_string(),
        }],
        items: vec![ItemSpawn {
            item_id: "rope".to_string(),
            position: (200.0, 80.0),
            quantity: 1,
        }],
    }
}

pub fn create_iceland_glacier_level() -> LevelDefinition {
    let width = 30;
    let height = 25;
    let mut terrain = vec![
        vec![
            TerrainData {
                terrain_type: TerrainType::Snow,
                slope: 0.2,
                stability: 0.7,
                climbable: false,
                climbing_difficulty: None,
                required_gear: vec![],
            };
            width
        ];
        height
    ];

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
        description: "Scale the mighty Icelandic glacier with proper gear and Viking courage"
            .to_string(),
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
            },
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
            },
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
            },
        ],
    }
}

pub fn save_sample_levels() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all("levels")?;

    let tutorial = create_tutorial_level();
    tutorial.save_to_file("levels/tutorial_01.ron")?;

    let glacier = create_iceland_glacier_level();
    glacier.save_to_file("levels/iceland_glacier_01.ron")?;

    // Generate large procedural levels
    let large_mountain = create_large_mountain_level();
    large_mountain.save_to_file("levels/large_mountain_01.ron")?;
    
    let coastal_cliffs = create_coastal_cliffs_level();
    coastal_cliffs.save_to_file("levels/coastal_cliffs_01.ron")?;
    
    let volcanic_peaks = create_volcanic_peaks_level();
    volcanic_peaks.save_to_file("levels/volcanic_peaks_01.ron")?;

    info!("Sample levels saved to levels/ directory");
    Ok(())
}

/// Create a large mountainous level with glaciers, lava fields, and varied terrain
pub fn create_large_mountain_level() -> LevelDefinition {
    create_mountain_terrain(200, 150) // Much larger: 200x150 = 30,000 tiles (40x larger than current levels)
}

/// Create a coastal cliffs level with dramatic sea cliffs and rock climbing
pub fn create_coastal_cliffs_level() -> LevelDefinition {
    create_coastal_terrain(180, 120) // 180x120 = 21,600 tiles
}

/// Create a volcanic peaks level with lava fields and challenging volcanic terrain
pub fn create_volcanic_peaks_level() -> LevelDefinition {
    create_volcanic_terrain(220, 180) // 220x180 = 39,600 tiles
}

/// Create a detailed mountain terrain with procedural generation
fn create_mountain_terrain(width: usize, height: usize) -> LevelDefinition {
    let mut rng = thread_rng();
    
    // Initialize with base terrain (Coast for lowlands)
    let mut terrain = vec![
        vec![
            TerrainData {
                terrain_type: TerrainType::Coast,
                slope: 0.1,
                stability: 0.9,
                climbable: false,
                climbing_difficulty: None,
                required_gear: vec![],
            };
            width
        ];
        height
    ];

    // Generate elevation map using multiple octaves of noise
    let elevation_map = generate_elevation_map(width, height, &mut rng);
    
    // Apply terrain based on elevation and features
    apply_terrain_by_elevation(&mut terrain, &elevation_map, width, height, &mut rng);
    
    // Add mountain features
    add_mountain_glacier(&mut terrain, width, height, &mut rng);
    add_lava_fields(&mut terrain, width, height, &mut rng);
    add_coastal_features(&mut terrain, width, height, &mut rng);
    add_rock_formations(&mut terrain, width, height, &mut rng);

    // Create appropriate wildlife
    let wildlife_spawns = generate_mountain_wildlife(width, height, &mut rng);
    
    // Create NPCs
    let npc_spawns = generate_mountain_npcs(width, height, &mut rng);
    
    // Scatter appropriate items
    let items = generate_mountain_items(width, height, &mut rng);

    LevelDefinition {
        id: "large_mountain_01".to_string(),
        name: "Great Mountain Range".to_string(),
        description: "A vast mountainous region with glaciers, lava fields, coastal areas, and varied terrain for advanced climbing challenges.".to_string(),
        width,
        height,
        terrain,
        start_position: (width / 8, height - 20), // Start at coastal area
        goal_positions: vec![(width * 3 / 4, height / 6)], // Summit
        weather_conditions: WeatherConditions {
            base_temperature: -5.0,
            wind_speed: 35.0,
            weather_type: "harsh_wind".to_string(),
        },
        wildlife_spawns,
        npc_spawns,
        items,
    }
}

/// Generate elevation map using layered noise for realistic terrain
fn generate_elevation_map(width: usize, height: usize, rng: &mut ThreadRng) -> Vec<Vec<f32>> {
    let mut elevation = vec![vec![0.0; width]; height];
    
    // Main mountain formation (Snæfellsjökull in upper portion)
    let peak_x = width * 3 / 4;
    let peak_y = height / 6;
    
    for y in 0..height {
        for x in 0..width {
            // Distance from peak
            let dx = (x as f32 - peak_x as f32) / width as f32;
            let dy = (y as f32 - peak_y as f32) / height as f32;
            let distance = (dx * dx + dy * dy).sqrt();
            
            // Base elevation from mountain
            let mountain_elevation = (1.0 - (distance * 2.5).min(1.0)).max(0.0);
            
            // Add coastal elevation (higher inland)
            let coastal_elevation = (y as f32 / height as f32) * 0.3;
            
            // Add random noise for natural variation
            let noise = (rng.gen::<f32>() - 0.5) * 0.2;
            
            elevation[y][x] = (mountain_elevation + coastal_elevation + noise).clamp(0.0, 1.0);
        }
    }
    
    elevation
}

/// Apply terrain types based on elevation and location
fn apply_terrain_by_elevation(
    terrain: &mut Vec<Vec<TerrainData>>, 
    elevation_map: &[Vec<f32>], 
    width: usize, 
    height: usize,
    rng: &mut ThreadRng
) {
    for y in 0..height {
        for x in 0..width {
            let elevation = elevation_map[y][x];
            let coastal_distance = y as f32 / height as f32;
            
            terrain[y][x].terrain_type = match elevation {
                e if e > 0.8 => TerrainType::Snow,     // High elevation snow
                e if e > 0.6 => TerrainType::Rock,     // Rocky highlands
                e if e > 0.4 => TerrainType::Grass,    // Mountain meadows
                e if e > 0.2 => {
                    if coastal_distance < 0.3 {
                        TerrainType::Coast           // Coastal areas
                    } else {
                        TerrainType::Soil           // Inland lowlands
                    }
                }
                _ => TerrainType::Coast,                // Sea level
            };
            
            // Set appropriate properties based on terrain type
            terrain[y][x].slope = elevation * 0.8 + rng.gen::<f32>() * 0.3;
            terrain[y][x].stability = match terrain[y][x].terrain_type {
                TerrainType::Rock => 0.9,
                TerrainType::Coast => 0.7,
                TerrainType::Soil => 0.8,
                TerrainType::Grass => 0.8,
                _ => 0.6,
            };
        }
    }
}

/// Add a mountain glacier
fn add_mountain_glacier(
    terrain: &mut Vec<Vec<TerrainData>>, 
    width: usize, 
    height: usize, 
    rng: &mut ThreadRng
) {
    let glacier_center_x = width * 3 / 4;
    let glacier_center_y = height / 6;
    let glacier_radius = (width.min(height) / 8) as f32;
    
    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - glacier_center_x as f32;
            let dy = y as f32 - glacier_center_y as f32;
            let distance = (dx * dx + dy * dy).sqrt();
            
            if distance < glacier_radius {
                // Inner core: pure glacier ice
                if distance < glacier_radius * 0.6 {
                    terrain[y][x] = TerrainData {
                        terrain_type: TerrainType::Glacier,
                        slope: 0.9,
                        stability: 0.4,
                        climbable: true,
                        climbing_difficulty: Some(4.5),
                        required_gear: vec!["ice_axe".to_string(), "crampons".to_string()],
                    };
                }
                // Outer ring: ice patches mixed with snow
                else if rng.gen::<f32>() < 0.7 {
                    terrain[y][x] = TerrainData {
                        terrain_type: TerrainType::Ice,
                        slope: 0.7,
                        stability: 0.5,
                        climbable: true,
                        climbing_difficulty: Some(3.0),
                        required_gear: vec!["ice_axe".to_string()],
                    };
                }
            }
        }
    }
}

/// Add volcanic lava fields characteristic of Iceland
fn add_lava_fields(
    terrain: &mut Vec<Vec<TerrainData>>, 
    width: usize, 
    height: usize, 
    rng: &mut ThreadRng
) {
    // Add several lava field areas scattered around
    let lava_areas = 4;
    
    for _ in 0..lava_areas {
        let center_x = rng.gen_range(0..width);
        let center_y = rng.gen_range(height / 3..height); // More towards southern areas
        let field_size = rng.gen_range(15..35);
        
        create_lava_field_area(terrain, center_x, center_y, field_size, rng);
    }
}

fn create_lava_field_area(
    terrain: &mut Vec<Vec<TerrainData>>, 
    center_x: usize, 
    center_y: usize, 
    size: usize, 
    rng: &mut ThreadRng
) {
    let height = terrain.len();
    let width = terrain[0].len();
    
    for y in center_y.saturating_sub(size)..=(center_y + size).min(height - 1) {
        for x in center_x.saturating_sub(size)..=(center_x + size).min(width - 1) {
            let dx = (x as i32 - center_x as i32).abs();
            let dy = (y as i32 - center_y as i32).abs();
            let distance = ((dx * dx + dy * dy) as f32).sqrt();
            
            if distance < size as f32 && rng.gen::<f32>() < 0.6 {
                terrain[y][x] = TerrainData {
                    terrain_type: TerrainType::Lava,
                    slope: 0.3,
                    stability: 0.2,
                    climbable: true,
                    climbing_difficulty: Some(5.0),
                    required_gear: vec!["heat_protection".to_string(), "sturdy_boots".to_string()],
                };
            }
        }
    }
}

/// Add dramatic coastal cliff features
fn add_coastal_features(
    terrain: &mut Vec<Vec<TerrainData>>, 
    width: usize, 
    height: usize, 
    rng: &mut ThreadRng
) {
    // Add rocky cliffs along the coast
    for y in (height * 4 / 5)..height {
        for x in 0..width {
            if rng.gen::<f32>() < 0.4 {
                terrain[y][x] = TerrainData {
                    terrain_type: TerrainType::Rock,
                    slope: 0.8,
                    stability: 0.9,
                    climbable: true,
                    climbing_difficulty: Some(2.5),
                    required_gear: vec!["climbing_gear".to_string()],
                };
            }
        }
    }
}

/// Add distinctive rock formations and crags
fn add_rock_formations(
    terrain: &mut Vec<Vec<TerrainData>>, 
    width: usize, 
    height: usize, 
    rng: &mut ThreadRng
) {
    let num_formations = 8;
    
    for _ in 0..num_formations {
        let center_x = rng.gen_range(0..width);
        let center_y = rng.gen_range(0..height * 2 / 3); // Avoid southern coastal area
        let formation_size = rng.gen_range(8..20);
        
        create_rock_formation(terrain, center_x, center_y, formation_size, rng);
    }
}

fn create_rock_formation(
    terrain: &mut Vec<Vec<TerrainData>>, 
    center_x: usize, 
    center_y: usize, 
    size: usize, 
    rng: &mut ThreadRng
) {
    let height = terrain.len();
    let width = terrain[0].len();
    
    for y in center_y.saturating_sub(size)..=(center_y + size).min(height - 1) {
        for x in center_x.saturating_sub(size)..=(center_x + size).min(width - 1) {
            let dx = (x as i32 - center_x as i32).abs();
            let dy = (y as i32 - center_y as i32).abs();
            let distance = ((dx * dx + dy * dy) as f32).sqrt();
            
            if distance < size as f32 && rng.gen::<f32>() < 0.8 {
                terrain[y][x] = TerrainData {
                    terrain_type: TerrainType::Rock,
                    slope: 0.6 + rng.gen::<f32>() * 0.3,
                    stability: 0.8,
                    climbable: true,
                    climbing_difficulty: Some(2.0 + rng.gen::<f32>() * 2.0),
                    required_gear: vec!["rope".to_string()],
                };
            }
        }
    }
}

/// Generate mountain wildlife (horses, sheep, occasional foxes)
fn generate_mountain_wildlife(width: usize, height: usize, rng: &mut ThreadRng) -> Vec<WildlifeSpawn> {
    let mut wildlife = Vec::new();
    
    // Mountain horses (gentle, grazing in lowlands)
    for _ in 0..rng.gen_range(8..15) {
        wildlife.push(WildlifeSpawn {
            species: "horse".to_string(),
            position: (
                rng.gen_range(0.0..(width as f32 * 32.0)),
                rng.gen_range((height as f32 * 0.6 * 32.0)..(height as f32 * 32.0))
            ),
            aggression: 0.0,
        });
    }
    
    // Sheep (scattered across grasslands)
    for _ in 0..rng.gen_range(15..25) {
        wildlife.push(WildlifeSpawn {
            species: "sheep".to_string(),
            position: (
                rng.gen_range(0.0..(width as f32 * 32.0)),
                rng.gen_range((height as f32 * 0.4 * 32.0)..(height as f32 * 0.9 * 32.0))
            ),
            aggression: 0.1,
        });
    }
    
    // Occasional mountain foxes (rare, in remote areas)
    for _ in 0..rng.gen_range(2..5) {
        wildlife.push(WildlifeSpawn {
            species: "wolf".to_string(), // Using wolf as proxy for mountain fox
            position: (
                rng.gen_range(0.0..(width as f32 * 32.0)),
                rng.gen_range(0.0..(height as f32 * 0.5 * 32.0))
            ),
            aggression: 0.3,
        });
    }
    
    wildlife
}

/// Generate mountain NPCs with Nordic names and roles
fn generate_mountain_npcs(width: usize, height: usize, rng: &mut ThreadRng) -> Vec<NPCSpawn> {
    let mut npcs = Vec::new();
    
    let viking_names = ["Björn", "Erik", "Leif", "Ragnar", "Thorvald", "Gunnar"];
    let female_names = ["Freydis", "Gudrun", "Astrid", "Ingrid", "Sigrid", "Helga"];
    
    // Viking climbers and guides
    for _ in 0..rng.gen_range(4..8) {
        let is_female = rng.gen_bool(0.3);
        let name = if is_female {
            female_names[rng.gen_range(0..female_names.len())]
        } else {
            viking_names[rng.gen_range(0..viking_names.len())]
        };
        
        npcs.push(NPCSpawn {
            name: format!("{} the {}", name, if is_female { "Wise" } else { "Bold" }),
            npc_type: "viking".to_string(),
            position: (
                rng.gen_range(0.0..(width as f32 * 32.0)),
                rng.gen_range((height as f32 * 0.5 * 32.0)..(height as f32 * 32.0))
            ),
            dialogue_file: "mountain_viking.ron".to_string(),
        });
    }
    
    // Mystical mages (fewer, in remote areas)
    for _ in 0..rng.gen_range(2..4) {
        let name = female_names[rng.gen_range(0..female_names.len())];
        
        npcs.push(NPCSpawn {
            name: format!("{} the Seer", name),
            npc_type: "mage".to_string(),
            position: (
                rng.gen_range(0.0..(width as f32 * 32.0)),
                rng.gen_range(0.0..(height as f32 * 0.4 * 32.0))
            ),
            dialogue_file: "mountain_mage.ron".to_string(),
        });
    }
    
    npcs
}

/// Generate appropriate items for mountain climbing
fn generate_mountain_items(width: usize, height: usize, rng: &mut ThreadRng) -> Vec<ItemSpawn> {
    let mut items = Vec::new();
    
    let mountain_items = [
        "warm_cloak", "ice_axe", "crampons", "rope", "climbing_gear", 
        "energy_bar", "dried_food", "wool_gloves", "hiking_boots"
    ];
    
    // Scatter items across the terrain
    for _ in 0..rng.gen_range(15..25) {
        items.push(ItemSpawn {
            item_id: mountain_items[rng.gen_range(0..mountain_items.len())].to_string(),
            position: (
                rng.gen_range(0.0..(width as f32 * 32.0)),
                rng.gen_range(0.0..(height as f32 * 32.0))
            ),
            quantity: 1,
        });
    }
    
    items
}

/// Create coastal cliff terrain with dramatic sea cliffs and rocky shores
fn create_coastal_terrain(width: usize, height: usize) -> LevelDefinition {
    let mut rng = thread_rng();
    
    // Initialize with coastal base terrain
    let mut terrain = vec![
        vec![
            TerrainData {
                terrain_type: TerrainType::Coast,
                slope: 0.1,
                stability: 0.8,
                climbable: false,
                climbing_difficulty: None,
                required_gear: vec![],
            };
            width
        ];
        height
    ];

    // Generate coastal elevation (high cliffs inland, beaches near water)
    let elevation_map = generate_coastal_elevation(width, height, &mut rng);
    apply_coastal_terrain(&mut terrain, &elevation_map, width, height, &mut rng);
    add_sea_cliffs(&mut terrain, width, height, &mut rng);
    add_rock_formations(&mut terrain, width, height, &mut rng);

    LevelDefinition {
        id: "coastal_cliffs_01".to_string(),
        name: "Dramatic Coastal Cliffs".to_string(),
        description: "Towering sea cliffs with challenging rock climbing routes and stunning coastal vistas.".to_string(),
        width,
        height,
        terrain,
        start_position: (5, height - 10),
        goal_positions: vec![(width - 10, height / 4)],
        weather_conditions: WeatherConditions {
            base_temperature: 8.0,
            wind_speed: 25.0,
            weather_type: "ocean_winds".to_string(),
        },
        wildlife_spawns: generate_coastal_wildlife(width, height, &mut rng),
        npc_spawns: generate_coastal_npcs(width, height, &mut rng),
        items: generate_coastal_items(width, height, &mut rng),
    }
}

/// Create volcanic terrain with lava fields, ash slopes, and volcanic peaks
fn create_volcanic_terrain(width: usize, height: usize) -> LevelDefinition {
    let mut rng = thread_rng();
    
    // Initialize with volcanic base terrain
    let mut terrain = vec![
        vec![
            TerrainData {
                terrain_type: TerrainType::Rock,
                slope: 0.3,
                stability: 0.7,
                climbable: false,
                climbing_difficulty: None,
                required_gear: vec![],
            };
            width
        ];
        height
    ];

    let elevation_map = generate_volcanic_elevation(width, height, &mut rng);
    apply_volcanic_terrain(&mut terrain, &elevation_map, width, height, &mut rng);
    add_volcanic_peaks(&mut terrain, width, height, &mut rng);
    add_extensive_lava_fields(&mut terrain, width, height, &mut rng);

    LevelDefinition {
        id: "volcanic_peaks_01".to_string(),
        name: "Ancient Volcanic Peaks".to_string(),
        description: "Challenging volcanic landscape with active lava flows, ash fields, and treacherous volcanic summits.".to_string(),
        width,
        height,
        terrain,
        start_position: (20, height - 30),
        goal_positions: vec![(width / 2, 30)],
        weather_conditions: WeatherConditions {
            base_temperature: 18.0,
            wind_speed: 12.0,
            weather_type: "volcanic_ash".to_string(),
        },
        wildlife_spawns: generate_volcanic_wildlife(width, height, &mut rng),
        npc_spawns: generate_volcanic_npcs(width, height, &mut rng),
        items: generate_volcanic_items(width, height, &mut rng),
    }
}

/// Generate elevation map for coastal terrain with high cliffs and low beaches
fn generate_coastal_elevation(width: usize, height: usize, rng: &mut ThreadRng) -> Vec<Vec<f32>> {
    let mut elevation = vec![vec![0.0; width]; height];
    
    for y in 0..height {
        for x in 0..width {
            let coastal_factor = (x as f32 / width as f32).powf(2.0);
            let cliff_height = coastal_factor * 0.8 + rng.gen::<f32>() * 0.3;
            elevation[y][x] = cliff_height.clamp(0.0, 1.0);
        }
    }
    elevation
}

/// Apply terrain types based on coastal elevation patterns
fn apply_coastal_terrain(terrain: &mut Vec<Vec<TerrainData>>, elevation_map: &Vec<Vec<f32>>, 
                        width: usize, height: usize, rng: &mut ThreadRng) {
    for y in 0..height {
        for x in 0..width {
            let elevation = elevation_map[y][x];
            let terrain_data = &mut terrain[y][x];
            
            match elevation {
                e if e < 0.2 => {
                    terrain_data.terrain_type = TerrainType::Coast;
                    terrain_data.slope = 0.05;
                    terrain_data.stability = 0.9;
                }
                e if e < 0.5 => {
                    terrain_data.terrain_type = TerrainType::Rock;
                    terrain_data.slope = 0.4;
                    terrain_data.stability = 0.7;
                    terrain_data.climbable = true;
                    terrain_data.climbing_difficulty = Some(3.0 + rng.gen::<f32>() * 3.0);
                }
                _ => {
                    terrain_data.terrain_type = TerrainType::Snow;
                    terrain_data.slope = 0.6;
                    terrain_data.stability = 0.6;
                    terrain_data.climbable = true;
                    terrain_data.climbing_difficulty = Some(5.0 + rng.gen::<f32>() * 3.0);
                    terrain_data.required_gear = vec!["rope".to_string(), "pitons".to_string()];
                }
            }
        }
    }
}

/// Add dramatic sea cliffs to the coastal terrain
fn add_sea_cliffs(terrain: &mut Vec<Vec<TerrainData>>, width: usize, height: usize, rng: &mut ThreadRng) {
    let cliff_regions = rng.gen_range(3..6);
    
    for _ in 0..cliff_regions {
        let start_x = rng.gen_range(width / 3..width - 20);
        let cliff_width = rng.gen_range(15..30);
        let cliff_height = rng.gen_range(40..80);
        
        for y in (height - cliff_height)..height {
            for x in start_x..(start_x + cliff_width).min(width) {
                if rng.gen::<f32>() > 0.3 {
                    terrain[y][x].terrain_type = TerrainType::Rock;
                    terrain[y][x].slope = 0.8;
                    terrain[y][x].stability = 0.5;
                    terrain[y][x].climbable = true;
                    terrain[y][x].climbing_difficulty = Some(6.0 + rng.gen::<f32>() * 3.0);
                }
            }
        }
    }
}

/// Generate elevation map for volcanic terrain with peaks and valleys
fn generate_volcanic_elevation(width: usize, height: usize, rng: &mut ThreadRng) -> Vec<Vec<f32>> {
    let mut elevation = vec![vec![0.0; width]; height];
    let num_peaks = rng.gen_range(2..5);
    
    for _ in 0..num_peaks {
        let peak_x = rng.gen_range(width / 4..3 * width / 4);
        let peak_y = rng.gen_range(height / 4..3 * height / 4);
        let peak_radius = rng.gen_range(30.0..60.0);
        
        for y in 0..height {
            for x in 0..width {
                let dist = ((x as f32 - peak_x as f32).powi(2) + 
                           (y as f32 - peak_y as f32).powi(2)).sqrt();
                let peak_influence = (1.0 - (dist / peak_radius).min(1.0)).max(0.0);
                elevation[y][x] = (elevation[y][x] + peak_influence * 0.8).min(1.0);
            }
        }
    }
    elevation
}

/// Apply volcanic terrain types based on elevation
fn apply_volcanic_terrain(terrain: &mut Vec<Vec<TerrainData>>, elevation_map: &Vec<Vec<f32>>, 
                         width: usize, height: usize, rng: &mut ThreadRng) {
    for y in 0..height {
        for x in 0..width {
            let elevation = elevation_map[y][x];
            let terrain_data = &mut terrain[y][x];
            
            match elevation {
                e if e < 0.2 => {
                    terrain_data.terrain_type = TerrainType::Rock;
                    terrain_data.slope = 0.2;
                    terrain_data.stability = 0.8;
                }
                e if e < 0.6 => {
                    terrain_data.terrain_type = TerrainType::Rock;
                    terrain_data.slope = 0.5;
                    terrain_data.stability = 0.6;
                    terrain_data.climbable = true;
                    terrain_data.climbing_difficulty = Some(4.0 + rng.gen::<f32>() * 3.0);
                }
                _ => {
                    terrain_data.terrain_type = TerrainType::Snow;
                    terrain_data.slope = 0.7;
                    terrain_data.stability = 0.5;
                    terrain_data.climbable = true;
                    terrain_data.climbing_difficulty = Some(6.0 + rng.gen::<f32>() * 3.0);
                }
            }
        }
    }
}

/// Add volcanic peaks with challenging climbing routes
fn add_volcanic_peaks(terrain: &mut Vec<Vec<TerrainData>>, width: usize, height: usize, rng: &mut ThreadRng) {
    let num_peaks = rng.gen_range(2..4);
    
    for _ in 0..num_peaks {
        let peak_x = rng.gen_range(40..width - 40);
        let peak_y = rng.gen_range(40..height - 40);
        let peak_size = rng.gen_range(15..25);
        
        for dy in -(peak_size as i32)..(peak_size as i32) {
            for dx in -(peak_size as i32)..(peak_size as i32) {
                let x = (peak_x as i32 + dx) as usize;
                let y = (peak_y as i32 + dy) as usize;
                
                if x < width && y < height {
                    let dist = (dx * dx + dy * dy) as f32;
                    if dist < (peak_size * peak_size) as f32 && rng.gen::<f32>() > 0.2 {
                        terrain[y][x].terrain_type = TerrainType::Rock;
                        terrain[y][x].slope = 0.8;
                        terrain[y][x].stability = 0.4;
                        terrain[y][x].climbable = true;
                        terrain[y][x].climbing_difficulty = Some(7.0 + rng.gen::<f32>() * 3.0);
                    }
                }
            }
        }
    }
}

/// Add extensive lava fields to volcanic terrain
fn add_extensive_lava_fields(terrain: &mut Vec<Vec<TerrainData>>, width: usize, height: usize, rng: &mut ThreadRng) {
    let num_lava_fields = rng.gen_range(4..7);
    
    for _ in 0..num_lava_fields {
        let field_x = rng.gen_range(0..width - 50);
        let field_y = rng.gen_range(0..height - 50);
        let field_width = rng.gen_range(25..60);
        let field_height = rng.gen_range(25..60);
        
        for y in field_y..(field_y + field_height).min(height) {
            for x in field_x..(field_x + field_width).min(width) {
                if rng.gen::<f32>() > 0.25 {
                    terrain[y][x].terrain_type = TerrainType::Lava;
                    terrain[y][x].slope = 0.1;
                    terrain[y][x].stability = 0.3;
                    terrain[y][x].climbable = false;
                }
            }
        }
    }
}

/// Generate coastal wildlife spawns
fn generate_coastal_wildlife(width: usize, height: usize, rng: &mut ThreadRng) -> Vec<WildlifeSpawn> {
    let mut spawns = Vec::new();
    let spawn_count = rng.gen_range(8..15);
    
    for _ in 0..spawn_count {
        spawns.push(WildlifeSpawn {
            position: (rng.gen_range(0..width) as f32, rng.gen_range(0..height) as f32),
            species: match rng.gen_range(0..4) {
                0 => "seagull".to_string(),
                1 => "seal".to_string(),
                2 => "puffin".to_string(),
                _ => "crab".to_string(),
            },
            aggression: rng.gen::<f32>() * 0.5,
        });
    }
    spawns
}

/// Generate coastal NPCs
fn generate_coastal_npcs(width: usize, height: usize, rng: &mut ThreadRng) -> Vec<NPCSpawn> {
    let mut npcs = Vec::new();
    let npc_count = rng.gen_range(2..5);
    
    for _ in 0..npc_count {
        npcs.push(NPCSpawn {
            name: match rng.gen_range(0..3) {
                0 => "lighthouse_keeper".to_string(),
                1 => "fisherman".to_string(),
                _ => "coastal_guide".to_string(),
            },
            position: (rng.gen_range(10..width - 10) as f32, rng.gen_range(10..height - 10) as f32),
            npc_type: match rng.gen_range(0..3) {
                0 => "lighthouse_keeper".to_string(),
                1 => "fisherman".to_string(),
                _ => "coastal_guide".to_string(),
            },
            dialogue_file: "coastal_character.ron".to_string(),
        });
    }
    npcs
}

/// Generate coastal items
fn generate_coastal_items(width: usize, height: usize, rng: &mut ThreadRng) -> Vec<ItemSpawn> {
    let mut items = Vec::new();
    let item_count = rng.gen_range(15..25);
    
    let coastal_items = [
        "rope", "climbing_harness", "helmet", "first_aid_kit",
        "compass", "binoculars", "waterproof_jacket", "flare_gun"
    ];
    
    for _ in 0..item_count {
        items.push(ItemSpawn {
            position: (rng.gen_range(0..width) as f32, rng.gen_range(0..height) as f32),
            item_id: coastal_items[rng.gen_range(0..coastal_items.len())].to_string(),
            quantity: rng.gen_range(1..4),
        });
    }
    items
}

/// Generate volcanic wildlife spawns
fn generate_volcanic_wildlife(width: usize, height: usize, rng: &mut ThreadRng) -> Vec<WildlifeSpawn> {
    let mut spawns = Vec::new();
    let spawn_count = rng.gen_range(6..12);
    
    for _ in 0..spawn_count {
        spawns.push(WildlifeSpawn {
            position: (rng.gen_range(0..width) as f32, rng.gen_range(0..height) as f32),
            species: match rng.gen_range(0..3) {
                0 => "volcanic_lizard".to_string(),
                1 => "fire_salamander".to_string(),
                _ => "mountain_goat".to_string(),
            },
            aggression: rng.gen::<f32>() * 0.7 + 0.3,
        });
    }
    spawns
}

/// Generate volcanic NPCs
fn generate_volcanic_npcs(width: usize, height: usize, rng: &mut ThreadRng) -> Vec<NPCSpawn> {
    let mut npcs = Vec::new();
    let npc_count = rng.gen_range(2..4);
    
    for _ in 0..npc_count {
        npcs.push(NPCSpawn {
            name: match rng.gen_range(0..3) {
                0 => "volcanologist".to_string(),
                1 => "rescue_worker".to_string(),
                _ => "mountain_guide".to_string(),
            },
            position: (rng.gen_range(30..width - 30) as f32, rng.gen_range(30..height - 30) as f32),
            npc_type: match rng.gen_range(0..3) {
                0 => "volcanologist".to_string(),
                1 => "rescue_worker".to_string(),
                _ => "mountain_guide".to_string(),
            },
            dialogue_file: "volcanic_character.ron".to_string(),
        });
    }
    npcs
}

/// Generate volcanic items
fn generate_volcanic_items(width: usize, height: usize, rng: &mut ThreadRng) -> Vec<ItemSpawn> {
    let mut items = Vec::new();
    let item_count = rng.gen_range(12..20);
    
    let volcanic_items = [
        "heat_resistant_suit", "gas_mask", "temperature_sensor", "ice_axe",
        "rope", "climbing_harness", "emergency_beacon", "cooling_pack"
    ];
    
    for _ in 0..item_count {
        items.push(ItemSpawn {
            position: (rng.gen_range(0..width) as f32, rng.gen_range(0..height) as f32),
            item_id: volcanic_items[rng.gen_range(0..volcanic_items.len())].to_string(),
            quantity: rng.gen_range(1..3),
        });
    }
    items
}

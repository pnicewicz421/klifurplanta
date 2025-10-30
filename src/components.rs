use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ===== PHASE 2: TERRAIN COMPONENTS =====

#[derive(Component)]
pub struct Terrain {
    pub terrain_type: TerrainType,
    pub movement_modifier: f32, // Speed multiplier: 1.0 = normal, 0.5 = half speed, etc.
    pub solid: bool,            // Can player pass through?
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TerrainType {
    Soil,  // Brown - normal movement
    Ice,   // Light blue - slippery (faster)
    Rock,  // Gray - slow movement
    Grass, // Green - normal movement
    Snow,  // White - slow movement
}

impl TerrainType {
    pub fn color(&self) -> Color {
        match self {
            TerrainType::Soil => Color::srgb(0.6, 0.4, 0.2), // Brown
            TerrainType::Ice => Color::srgb(0.7, 0.9, 1.0),  // Light blue
            TerrainType::Rock => Color::srgb(0.5, 0.5, 0.5), // Gray
            TerrainType::Grass => Color::srgb(0.3, 0.7, 0.3), // Green
            TerrainType::Snow => Color::srgb(0.9, 0.9, 0.9), // White
        }
    }

    pub fn movement_modifier(&self) -> f32 {
        match self {
            TerrainType::Soil => 1.0,
            TerrainType::Ice => 1.3,  // Slippery - faster
            TerrainType::Rock => 0.6, // Slow and difficult
            TerrainType::Grass => 1.0,
            TerrainType::Snow => 0.7, // Slow in snow
        }
    }
}

// ===== EXISTING COMPONENTS =====

#[derive(Component)]
pub struct Player {
    pub id: u8, // 1-4 for multiplayer support
}

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct Hunger {
    pub current: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct Thirst {
    pub current: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct Morale {
    pub current: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct MovementStats {
    pub speed: f32,
    pub climbing_skill: f32,
    pub stamina: f32,
    pub max_stamina: f32,
}

// ===== INVENTORY & EQUIPMENT =====

#[derive(Component)]
pub struct Inventory {
    pub items: Vec<Item>,
    pub capacity: usize,
    pub weight_limit: f32,
    pub current_weight: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub weight: f32,
    pub item_type: ItemType,
    pub durability: Option<f32>,
    pub properties: ItemProperties,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ItemType {
    ClimbingGear,
    Clothing,
    Tool,
    Food,
    Shelter,
    Magical,
    Animal,
    Misc,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ItemProperties {
    pub warmth: Option<f32>,
    pub strength: Option<f32>,
    pub magic_power: Option<f32>,
    pub nutrition: Option<f32>,
    pub water: Option<f32>,
    pub protection: Option<f32>,
}

#[derive(Component)]
pub struct EquippedItems {
    pub axe: Option<Item>,
    pub boots: Option<Item>,
    pub jacket: Option<Item>,
    pub gloves: Option<Item>,
    pub backpack: Option<Item>,
}

impl Default for EquippedItems {
    fn default() -> Self {
        Self::new()
    }
}

impl EquippedItems {
    pub fn new() -> Self {
        Self {
            axe: None,
            boots: None,
            jacket: None,
            gloves: None,
            backpack: None,
        }
    }

    pub fn get_total_warmth(&self) -> f32 {
        let mut warmth = 0.0;
        if let Some(boots) = &self.boots {
            warmth += boots.properties.warmth.unwrap_or(0.0);
        }
        if let Some(jacket) = &self.jacket {
            warmth += jacket.properties.warmth.unwrap_or(0.0);
        }
        if let Some(gloves) = &self.gloves {
            warmth += gloves.properties.warmth.unwrap_or(0.0);
        }
        warmth
    }

    pub fn get_climbing_bonus(&self) -> f32 {
        let mut bonus = 0.0;
        if let Some(axe) = &self.axe {
            bonus += axe.properties.strength.unwrap_or(0.0);
        }
        if let Some(boots) = &self.boots {
            bonus += boots.properties.strength.unwrap_or(0.0);
        }
        bonus
    }
}

// ===== TERRAIN & ENVIRONMENT =====

#[derive(Component)]
pub struct TerrainTile {
    pub terrain_type: TerrainType,
    pub slope: f32,     // 0.0 = flat, 1.0 = vertical
    pub stability: f32, // How stable the terrain is
    pub climbable: bool,
}

#[derive(Component)]
pub struct Climbable {
    pub difficulty: f32,
    pub required_gear: Vec<String>,
}

#[derive(Component)]
pub struct Hazardous {
    pub damage_per_second: f32,
    pub hazard_type: HazardType,
}

#[derive(Clone, Debug)]
pub enum HazardType {
    Fall,
    Cold,
    Heat,
    Wildlife,
    Weather,
    Magic,
}

// ===== NPCs & WILDLIFE =====

#[derive(Component)]
pub struct Npc {
    pub name: String,
    pub npc_type: NPCType,
    pub dialogue_tree: String, // Reference to dialogue file
    pub join_probability: f32,
}

#[derive(Clone, Debug)]
pub enum NPCType {
    Climber,
    Guide,
    Trader,
    Hermit,
    Viking,
    Mage,
}

#[derive(Component)]
pub struct Wildlife {
    pub species: WildlifeSpecies,
    pub aggression: f32,
    pub flee_distance: f32,
    pub attack_damage: f32,
}

#[derive(Clone, Debug)]
pub enum WildlifeSpecies {
    Bear,
    Puma,
    Cougar,
    Wolf,
    Eagle,
    // Icelandic animals
    Horse,
    Sheep,
    Cattle,
    Goat,
    Pig,
    Dog,
}

// ===== MAGIC & SUPERNATURAL =====

#[derive(Component)]
pub struct MagicUser {
    pub magic_type: MagicType,
    pub mana: f32,
    pub max_mana: f32,
    pub known_spells: Vec<String>,
}

#[derive(Clone, Debug)]
pub enum MagicType {
    Elf,
    Norse, // Thor, Loki, etc.
    Nature,
    Rune,
}

#[derive(Component)]
pub struct Spell {
    pub name: String,
    pub mana_cost: f32,
    pub effect: SpellEffect,
    pub duration: Option<f32>,
}

#[derive(Clone, Debug)]
pub enum SpellEffect {
    Heal(f32),
    BoostClimbing(f32),
    WeatherControl,
    AnimalFriend,
    RockStability,
    Light,
    Warmth,
}

// ===== STRUCTURES & BUILDINGS =====

#[derive(Component)]
pub struct Structure {
    pub structure_type: StructureType,
    pub durability: f32,
    pub capacity: usize, // How many people can use it
}

#[derive(Clone, Debug)]
pub enum StructureType {
    Tent,
    Hut,
    Shelter,
    FirePit,
    Altar,
}

// ===== POSITION & PHYSICS =====

#[derive(Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32, // For elevation
}

#[derive(Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Component)]
pub struct Climbing {
    pub is_climbing: bool,
    pub anchor_point: Option<Entity>,
    pub rope_length: f32,
}

// ===== UI & INTERACTION =====

#[derive(Component)]
pub struct Interactable {
    pub interaction_type: InteractionType,
    pub range: f32,
}

#[derive(Clone, Debug)]
pub enum InteractionType {
    Pickup,
    Talk,
    Climb,
    Rest,
    Shop,
    Build,
    Cast,
}

// ===== MARKERS =====

#[derive(Component)]
pub struct SelectedCharacter;

#[derive(Component)]
pub struct InConversation;

#[derive(Component)]
pub struct Sleeping {
    pub time_remaining: f32,
}

// ===== UI COMPONENTS =====

#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct StaminaBar;

#[derive(Component)]
pub struct HealthBarFill;

#[derive(Component)]
pub struct StaminaBarFill;

// ===== INVENTORY UI COMPONENTS =====

#[derive(Component)]
pub struct InventoryUI;

#[derive(Component)]
pub struct InventorySlot {
    pub slot_index: usize,
}

#[derive(Component)]
pub struct InventorySlotImage {
    pub slot_index: usize,
}

#[derive(Component)]
pub struct InventorySlotText {
    pub slot_index: usize,
}

#[derive(Component)]
pub struct EquipmentSlot {
    pub slot_type: EquipmentSlotType,
}

#[derive(Component)]
pub struct CloseButton;

#[derive(Clone, Debug)]
pub enum EquipmentSlotType {
    Axe,
    Boots,
    Jacket,
    Gloves,
    Backpack,
}

// ===== ICE AXE INTERACTION COMPONENTS =====

/// Component marking terrain that can be broken with ice axes
#[derive(Component)]
pub struct Breakable {
    pub tool_required: ToolType,
    pub durability: f32,
    pub max_durability: f32,
}

/// Types of tools that can break terrain
#[derive(Clone, Debug, PartialEq)]
pub enum ToolType {
    IceAxe,
    Pickaxe,
    Hammer,
}

/// Component for tracking ice axe usage state
#[derive(Component)]
pub struct IceAxeUsage {
    pub is_breaking: bool,
    pub target_position: Option<Vec3>,
    pub break_progress: f32,
    pub break_duration: f32, // Time needed to break terrain
}

/// Event for when terrain is broken
#[derive(Event)]
pub struct TerrainBrokenEvent {
    pub position: Vec3,
    pub terrain_type: TerrainType,
    pub tool_used: ToolType,
}

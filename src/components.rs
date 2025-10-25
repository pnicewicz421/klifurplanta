use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ===== CORE PLAYER COMPONENTS =====

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

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemProperties {
    pub warmth: Option<f32>,
    pub strength: Option<f32>,
    pub magic_power: Option<f32>,
    pub nutrition: Option<f32>,
    pub water: Option<f32>,
    pub protection: Option<f32>,
}

// ===== TERRAIN & ENVIRONMENT =====

#[derive(Component)]
pub struct TerrainTile {
    pub terrain_type: TerrainType,
    pub slope: f32, // 0.0 = flat, 1.0 = vertical
    pub stability: f32, // How stable the terrain is
    pub climbable: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TerrainType {
    Soil,
    Rock,
    Ice,
    Snow,
    Glacier,
    Lava,
    Water,
    Vegetation,
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
pub struct NPC {
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
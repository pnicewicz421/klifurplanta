use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    CharacterSelection,
    Shop,
    Climbing,
    Conversation,
    Sleeping,
    Inventory,
    Magic,
    Building,
    GameOver,
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum ClimbingState {
    #[default]
    Planning,    // Looking at the route, planning next moves
    Moving,      // Actively climbing/moving
    Resting,     // Taking a break on a ledge
    Emergency,   // Dealing with hazards, falling, etc.
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum TimeOfDay {
    #[default]
    Dawn,
    Morning,
    Midday,
    Afternoon,
    Evening,
    Night,
    Midnight,
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum Weather {
    #[default]
    Clear,
    Cloudy,
    Rain,
    Snow,
    Storm,
    Blizzard,
    Fog,
    Wind,
}
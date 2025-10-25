# Klifurplanta (Mountain Climber)

An Iceland climbing adventure I'm building to help me learn Rust in a creative way. I'm trying bevy (https://bevy.org/) for the first time.

## About the Game

Mountain Climber is a puzzle-adventure game where you play as a mountaineer navigating challenging Icelandic terrain with magical and supernatural elements. The game combines realistic climbing mechanics with Norse mythology and survival elements.

### Key Features

- **Character System**: Play as 1-4 climbers with different abilities
- **Terrain Variety**: Navigate soil, ice, rock, and other terrain types with unique properties
- **Equipment Management**: Limited inventory space and money for gear selection
- **Time System**: Day/night cycles (25 seconds = 1 game hour)
- **Survival Mechanics**: Health, food, and water meters
- **Weather & Hazards**: Dynamic weather, rockfall, earthquakes, volcanic activity
- **Magic System**: Learn spells and call upon Norse gods (Thor, Loki)
- **NPCs & Animals**: Meet other climbers, encounter wildlife, domesticate animals
- **Icelandic Elements**: Build huts, manage livestock (horses, sheep, cattle, goats, pigs, dogs)

## Learning Goals

This project is designed to explore:
- **Entity Component System (ECS)** architecture with Bevy
- Proper component design and entity queries
- Game state management
- Resource systems
- Event-driven programming patterns

## Technology Stack

- **Rust** - Programming language
- **Bevy** - Game engine (ECS architecture)
- **Bevy Cheatbook** - Primary reference guide

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Cargo (comes with Rust)

### Installation

1. Clone the repository:
```bash
git clone <your-repo-url>
cd climber
```

2. Run the game:
```bash
cargo run
```

### Development

Build for development:
```bash
cargo build
```

Run with optimizations:
```bash
cargo run --release
```

## Game Mechanics

### Core Systems

1. **Shop System**: Purchase equipment before climbing
   - Tents ($70), jackets, ropes, harnesses, belay devices, quickdraws
   - Limited inventory space and budget constraints

2. **Climbing System**: 
   - Different terrain types require different strategies
   - Tool usage affects climbing success
   - Body positioning and technique matter

3. **Time & Environment**:
   - Real-time day/night cycle
   - Visibility affected by lighting conditions
   - Moon phases provide natural light

4. **Survival Elements**:
   - Health meter affected by accidents and conditions
   - Food and water consumption over time
   - Death occurs when health reaches zero

5. **Social Interaction**:
   - Meet NPCs on the mountain
   - Conversation system
   - Recruitment mechanics (mostly rejection, occasional acceptance)

6. **Magic & Mythology**:
   - Learn and cast spells
   - Invoke Norse deities
   - Discover magical items and artifacts

### Hazards & Challenges

- **Natural Disasters**: Rockfall, earthquakes, volcanic eruptions
- **Weather**: Strong winds, rain, extreme cold
- **Wildlife**: Encounters with pumas, cougars, bears
- **Terrain**: Glaciers (jökull) requiring specialized gear

### Activities

- **Rest**: Sleep through dangerous night hours
- **Morale**: Read books, play music to maintain team spirit
- **Exploration**: Find lost items, rocks, eggs, and other useful objects
- **Construction**: Build shelters and huts
- **Animal Husbandry**: Care for and utilize domestic animals

## Project Structure

```
src/
├── main.rs           # Main game loop and setup
├── components.rs     # ECS components definition
├── systems.rs        # Game logic systems
├── resources.rs      # Global game resources
├── states.rs         # Game state management
└── levels.rs         # Level design and management
```

## Contributing

This is a learning project, but feedback and suggestions are welcome! Feel free to:
- Report bugs or issues
- Suggest improvements to the ECS architecture
- Share Bevy best practices
- Contribute documentation improvements

## References

- [Bevy Game Engine](https://bevy.org/)
- [Bevy Cheatbook](https://bevy-cheatbook.github.io/programming)
- [Rust Programming Language](https://www.rust-lang.org/)

## License

This project is licensed under the GNU Lesser General Public License v3.0 (LGPL-3.0). See the [LICENSE](LICENSE) file for details.

---

*Klifurplanta* - From Icelandic "klifur" (climber) + "planta" (plan/design). An adventure in both mountaineering and Rust programming!
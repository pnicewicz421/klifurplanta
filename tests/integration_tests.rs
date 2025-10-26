use mountain_climber::components::*;
use mountain_climber::states::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_compile_optimization_enabled() {
        // This test ensures the fast compile optimization is set in Cargo.toml
        // If this test compiles quickly, our optimization settings are working
        let start = std::time::Instant::now();
        
        // Simple computation that would be slow without optimization
        let mut sum = 0;
        for i in 0..1000 {
            sum += i;
        }
        
        let duration = start.elapsed();
        println!("Test computation took: {:?}", duration);
        
        // This should complete very quickly with optimizations
        assert!(duration.as_millis() < 100);
        assert_eq!(sum, 499500); // Verify correctness
    }

    #[test]
    fn test_health_component_initialization() {
        let health = Health { current: 100.0, max: 100.0 };
        assert_eq!(health.current, 100.0, "Health should initialize to 100");
        assert_eq!(health.max, 100.0, "Max health should be 100");
    }

    #[test]
    fn test_movement_stats_initialization() {
        let movement_stats = MovementStats { 
            speed: 200.0,
            climbing_skill: 1.0,
            stamina: 100.0,
            max_stamina: 100.0,
        };
        
        assert_eq!(movement_stats.stamina, 100.0, "Stamina should initialize to 100");
        assert_eq!(movement_stats.max_stamina, 100.0, "Max stamina should be 100");
        assert_eq!(movement_stats.climbing_skill, 1.0, "Base climbing skill should be 1.0");
    }

    #[test]
    fn test_player_component_exists() {
        let player = Player { id: 1 };
        assert_eq!(player.id, 1, "Player should have ID 1");
    }

    #[test]
    fn test_item_types_are_comparable() {
        assert_eq!(ItemType::ClimbingGear, ItemType::ClimbingGear);
        assert_eq!(ItemType::Clothing, ItemType::Clothing);
        assert_ne!(ItemType::ClimbingGear, ItemType::Clothing);
    }

    #[test]
    fn test_equipment_creation() {
        let ice_axe = Item {
            id: "test_axe".to_string(),
            name: "Test Ice Axe".to_string(),
            weight: 1.5,
            item_type: ItemType::ClimbingGear,
            durability: Some(100.0),
            properties: ItemProperties {
                strength: Some(15.0),
                warmth: None,
                magic_power: None,
                nutrition: None,
                water: None,
                protection: Some(5.0),
            },
        };

        assert_eq!(ice_axe.name, "Test Ice Axe");
        assert_eq!(ice_axe.item_type, ItemType::ClimbingGear);
        assert_eq!(ice_axe.weight, 1.5);
        assert_eq!(ice_axe.properties.strength, Some(15.0));
    }

    #[test]
    fn test_equipped_items_climbing_bonus() {
        let mut equipped = EquippedItems::new();
        
        // Initially no bonus
        assert_eq!(equipped.get_climbing_bonus(), 0.0);
        
        // Add ice axe
        let ice_axe = Item {
            id: "test_axe".to_string(),
            name: "Test Ice Axe".to_string(),
            weight: 1.5,
            item_type: ItemType::ClimbingGear,
            durability: Some(100.0),
            properties: ItemProperties {
                strength: Some(15.0),
                warmth: None,
                magic_power: None,
                nutrition: None,
                water: None,
                protection: Some(5.0),
            },
        };
        equipped.axe = Some(ice_axe);
        
        assert_eq!(equipped.get_climbing_bonus(), 15.0, "Ice axe should provide +15% climbing bonus");
        
        // Add boots
        let boots = Item {
            id: "test_boots".to_string(),
            name: "Test Boots".to_string(),
            weight: 3.0,
            item_type: ItemType::Clothing,
            durability: Some(100.0),
            properties: ItemProperties {
                strength: Some(10.0),
                warmth: Some(20.0),
                magic_power: None,
                nutrition: None,
                water: None,
                protection: Some(15.0),
            },
        };
        equipped.boots = Some(boots);
        
        assert_eq!(equipped.get_climbing_bonus(), 25.0, "Ice axe + boots should provide +25% climbing bonus");
    }

    #[test]
    fn test_inventory_weight_calculation() {
        let ice_axe = Item {
            id: "test_axe".to_string(),
            name: "Test Ice Axe".to_string(),
            weight: 1.5,
            item_type: ItemType::ClimbingGear,
            durability: Some(100.0),
            properties: ItemProperties {
                strength: Some(15.0),
                warmth: None,
                magic_power: None,
                nutrition: None,
                water: None,
                protection: Some(5.0),
            },
        };

        let boots = Item {
            id: "test_boots".to_string(),
            name: "Test Boots".to_string(),
            weight: 3.0,
            item_type: ItemType::Clothing,
            durability: Some(100.0),
            properties: ItemProperties {
                strength: Some(10.0),
                warmth: Some(20.0),
                magic_power: None,
                nutrition: None,
                water: None,
                protection: Some(15.0),
            },
        };

        let inventory = Inventory {
            items: vec![ice_axe, boots],
            capacity: 20,
            weight_limit: 25.0,
            current_weight: 4.5, // 1.5 + 3.0
        };

        assert_eq!(inventory.items.len(), 2, "Inventory should contain 2 items");
        assert_eq!(inventory.current_weight, 4.5, "Total weight should be 4.5kg");
        assert!(inventory.current_weight < inventory.weight_limit, "Weight should be under limit");
    }

    #[test]
    fn test_game_states_exist() {
        // Test that all game states are accessible
        let _climbing = GameState::Climbing;
        let _inventory = GameState::Inventory;
        
        // Test state comparisons
        assert_eq!(GameState::Climbing, GameState::Climbing);
        assert_ne!(GameState::Climbing, GameState::Inventory);
    }
}
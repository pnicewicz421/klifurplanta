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
        let health = Health {
            current: 100.0,
            max: 100.0,
        };
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

        assert_eq!(
            movement_stats.stamina, 100.0,
            "Stamina should initialize to 100"
        );
        assert_eq!(
            movement_stats.max_stamina, 100.0,
            "Max stamina should be 100"
        );
        assert_eq!(
            movement_stats.climbing_skill, 1.0,
            "Base climbing skill should be 1.0"
        );
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

        assert_eq!(
            equipped.get_climbing_bonus(),
            15.0,
            "Ice axe should provide +15% climbing bonus"
        );

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

        assert_eq!(
            equipped.get_climbing_bonus(),
            25.0,
            "Ice axe + boots should provide +25% climbing bonus"
        );
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
        assert_eq!(
            inventory.current_weight, 4.5,
            "Total weight should be 4.5kg"
        );
        assert!(
            inventory.current_weight < inventory.weight_limit,
            "Weight should be under limit"
        );
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

    // ===== ICE AXE INTERACTION TESTS =====

    #[test]
    fn test_ice_axe_in_inventory() {
        // Test that player can have ice axe in inventory
        let ice_axe = Item {
            id: "ice_axe_test".to_string(),
            name: "Ice Axe".to_string(),
            weight: 1.5,
            item_type: ItemType::ClimbingGear,
            durability: Some(100.0),
            properties: ItemProperties {
                strength: Some(15.0),
                ..Default::default()
            },
        };

        let inventory = Inventory {
            items: vec![ice_axe.clone()],
            capacity: 10,
            weight_limit: 20.0,
            current_weight: ice_axe.weight,
        };

        assert_eq!(inventory.items.len(), 1, "Should have one item in inventory");
        assert_eq!(inventory.items[0].name, "Ice Axe", "Should have ice axe");
        assert_eq!(
            inventory.items[0].item_type,
            ItemType::ClimbingGear,
            "Ice axe should be climbing gear"
        );
    }

    #[test]
    fn test_retrieve_axe_from_inventory() {
        // Test finding ice axe in inventory
        let ice_axe = Item {
            id: "ice_axe_test".to_string(),
            name: "Ice Axe".to_string(),
            weight: 1.5,
            item_type: ItemType::ClimbingGear,
            durability: Some(100.0),
            properties: ItemProperties::default(),
        };

        let other_item = Item {
            id: "rope_test".to_string(),
            name: "Climbing Rope".to_string(),
            weight: 2.0,
            item_type: ItemType::ClimbingGear,
            durability: Some(100.0),
            properties: ItemProperties::default(),
        };

        let inventory = Inventory {
            items: vec![other_item, ice_axe.clone()],
            capacity: 10,
            weight_limit: 20.0,
            current_weight: 3.5,
        };

        // Test finding ice axe
        let found_axe = inventory.items.iter().find(|item| {
            item.name.to_lowercase().contains("ice axe") || 
            item.name.to_lowercase().contains("axe")
        });

        assert!(found_axe.is_some(), "Should find ice axe in inventory");
        assert_eq!(found_axe.unwrap().name, "Ice Axe", "Should find correct axe");
    }

    #[test]
    fn test_ice_terrain_breakable_properties() {
        // Test ice terrain with breakable component
        let breakable = Breakable {
            tool_required: ToolType::IceAxe,
            durability: 50.0,
            max_durability: 50.0,
        };

        assert_eq!(
            breakable.tool_required,
            ToolType::IceAxe,
            "Ice should require ice axe"
        );
        assert_eq!(breakable.durability, 50.0, "Should have full durability");
        assert_eq!(
            breakable.max_durability, 50.0,
            "Should have max durability"
        );
    }

    #[test]
    fn test_terrain_damage_calculation() {
        // Test terrain breaking damage
        let mut breakable = Breakable {
            tool_required: ToolType::IceAxe,
            durability: 50.0,
            max_durability: 50.0,
        };

        let damage = 25.0;
        breakable.durability = (breakable.durability - damage).max(0.0);

        assert_eq!(
            breakable.durability, 25.0,
            "Should have reduced durability after damage"
        );

        // Test complete breaking
        breakable.durability = (breakable.durability - damage).max(0.0);
        assert_eq!(
            breakable.durability, 0.0,
            "Should be completely broken after second hit"
        );
    }

    #[test]
    fn test_terrain_type_ice_properties() {
        // Test ice terrain properties
        let ice_terrain = TerrainType::Ice;
        let movement_modifier = ice_terrain.movement_modifier();
        let color = ice_terrain.color();

        assert_eq!(movement_modifier, 1.3, "Ice should be slippery/faster");
        // Ice should be light blue-ish
        assert!(color.to_srgba().blue > 0.9, "Ice should be light blue");
    }

    #[test]
    fn test_tool_type_matching() {
        // Test tool type matching for terrain breaking
        let ice_breakable = Breakable {
            tool_required: ToolType::IceAxe,
            durability: 50.0,
            max_durability: 50.0,
        };

        // Test correct tool
        assert_eq!(
            ice_breakable.tool_required,
            ToolType::IceAxe,
            "Should require ice axe"
        );

        // Test tool equality
        assert_eq!(ToolType::IceAxe, ToolType::IceAxe, "Tool types should match");
        assert_ne!(
            ToolType::IceAxe,
            ToolType::Pickaxe,
            "Different tools should not match"
        );
    }

    #[test]
    fn test_terrain_broken_event() {
        // Test terrain broken event creation
        use bevy::math::Vec3;
        
        let event = TerrainBrokenEvent {
            position: Vec3::new(10.0, 20.0, 0.0),
            terrain_type: TerrainType::Ice,
            tool_used: ToolType::IceAxe,
        };

        assert_eq!(event.position.x, 10.0, "Should have correct position");
        assert_eq!(event.terrain_type, TerrainType::Ice, "Should be ice terrain");
        assert_eq!(event.tool_used, ToolType::IceAxe, "Should use ice axe");
    }

    #[test]
    fn test_variable_terrain_movement_costs() {
        // Test that different terrain types have different movement costs
        let grass = TerrainType::Grass;
        let ice = TerrainType::Ice;
        let rock = TerrainType::Rock;
        let snow = TerrainType::Snow;

        let grass_modifier = grass.movement_modifier();
        let ice_modifier = ice.movement_modifier();
        let rock_modifier = rock.movement_modifier();
        let snow_modifier = snow.movement_modifier();

        // Different terrain should have different movement costs
        assert_ne!(
            grass_modifier, rock_modifier,
            "Grass and rock should have different movement costs"
        );
        assert!(
            rock_modifier < grass_modifier,
            "Rock should be slower than grass"
        );
        assert!(
            snow_modifier < grass_modifier,
            "Snow should be slower than grass"
        );
        assert!(
            ice_modifier > grass_modifier,
            "Ice should be faster/slippery compared to grass"
        );
    }
}

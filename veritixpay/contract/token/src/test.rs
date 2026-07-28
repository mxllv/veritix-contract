

// Tests for set_max_supply functionality
#[test]
fn test_set_max_supply_lower_cap_succeeds() {
    let (env, admin, _user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);
    
    // Initialize with max supply of 1,000,000
    client.initialize_with_max_supply(
        &admin,
        &String::from_str(&env, "Veritix"),
        &String::from_str(&env, "VTX"),
        &7u32,
        &1_000_000i128
    );
    
    // Mint some tokens to reach 500,000
    let recipient = Address::generate(&env);
    client.mint(&admin, &recipient, &500_000i128);
    
    // Lower max supply to 750,000 (which is above current supply of 500,000)
    client.set_max_supply(&admin, &750_000i128);
    
    // Verify max supply was updated
    assert_eq!(client.max_supply(), 750_000i128);
}

#[test]
#[should_panic(expected = "CannotRaiseMaxSupply")]
fn test_set_max_supply_raise_cap_panics() {
    let (env, admin, _user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);
    
    // Initialize with max supply of 1,000,000
    client.initialize_with_max_supply(
        &admin,
        &String::from_str(&env, "Veritix"),
        &String::from_str(&env, "VTX"),
        &7u32,
        &1_000_000i128
    );
    
    // Attempt to raise max supply to 2,000,000 - should panic
    client.set_max_supply(&admin, &2_000_000i128);
}

#[test]
#[should_panic(expected = "Cannot set max supply below current total supply")]
fn test_set_max_supply_below_current_supply_panics() {
    let (env, admin, _user) = setup();
    env.mock_all_auths();
    let client = create_client(&env);
    
    // Initialize with max supply of 1,000,000
    client.initialize_with_max_supply(
        &admin,
        &String::from_str(&env, "Veritix"),
        &String::from_str(&env, "VTX"),
        &7u32,
        &1_000_000i128
    );
    
    // Mint 600,000 tokens
    let recipient = Address::generate(&env);
    client.mint(&admin, &recipient, &600_000i128);
    
    // Attempt to set max supply to 500,000 which is below current supply of 600,000 - should panic
    client.set_max_supply(&admin, &500_000i128);
}
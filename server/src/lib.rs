use log;
use rand::Rng;
use spacetimedb::{reducer, table, Identity, ReducerContext, Table, Timestamp};
use std::time::Duration;

// ------------------------------------------------------------
// Constants
// ------------------------------------------------------------
const PLAYER_COLORS: &[&str] = &["red", "green", "yellow", "orange", "purple"];
const BOARD_SIZE: u32 = 7;
const NATURAL_DEFENSE: u32 = 1;

// ------------------------------------------------------------
// Helper Functions
// ------------------------------------------------------------
fn create_new_deck(ctx: &ReducerContext) {
    let suits = ["hearts", "diamonds", "clubs", "spades"];
    for suit in suits.iter() {
        for value in 1..=13 {
            ctx.db.card().insert(Card {
                card_id: 0,
                suit: suit.to_string(),
                value,
                owner_color: None, // Cards start in the deck
            });
        }
    }
}

// ------------------------------------------------------------
// Schemas
// ------------------------------------------------------------

#[spacetimedb::table(name = tile, public)]
#[derive(Clone)]
pub struct Tile {
    #[primary_key]
    #[auto_inc]
    tile_id: u32,
    x: u32,
    y: u32,
    owner_color: Option<String>, // None means neutral
    troops: u32,                 // Number of infantry troops on this tile
    tanks: u32,                  // Number of tanks on this tile
}

#[spacetimedb::table(name = player, public)]
#[derive(Clone)]
pub struct Player {
    #[primary_key]
    color: String,
    identity: Option<Identity>,
    online: bool,
    gold: u32,    // Amount of gold the player has
    stamina: u32, // Amount of stamina the player has (max 2)
}

#[spacetimedb::table(name = game, public)]
pub struct Game {
    #[primary_key]
    game_name: String,
    turn: u64,
    current_player_index: u32,
}

#[spacetimedb::table(name = next_turn_timer, scheduled(next_turn))]
pub struct NextTurnTimer {
    #[primary_key]
    #[auto_inc]
    scheduled_id: u64,
    scheduled_at: spacetimedb::ScheduleAt,
}

// New tables for card system
#[spacetimedb::table(name = card, public)]
#[derive(Clone)]
pub struct Card {
    #[primary_key]
    #[auto_inc]
    card_id: u32,
    suit: String,                // "hearts", "diamonds", "clubs", "spades"
    value: u8,                   // 1-13 (Ace through King)
    owner_color: Option<String>, // None means card is in deck, Some(player_color) means card belongs to player
}

#[spacetimedb::reducer(init)]
pub fn init(ctx: &ReducerContext) {
    // Initialize the game with turn 1
    ctx.db.game().insert(Game {
        game_name: "main".to_string(),
        turn: 1,
        current_player_index: 0,
    });

    // Initialize the 5 players with their colors
    for color in PLAYER_COLORS {
        ctx.db.player().insert(Player {
            color: color.to_string(),
            identity: None,
            online: false,
            gold: 0,    // Initialize gold to 0
            stamina: 0, // Initialize stamina to 0
        });
    }

    // Initialize the board with tiles
    for x in 0..BOARD_SIZE {
        for y in 0..BOARD_SIZE {
            // Initialize all tiles with natural defense
            let mut tile = Tile {
                tile_id: 0,
                x,
                y,
                owner_color: None,
                troops: 0,
                tanks: 0,
            };

            // Set initial player positions with 5 troops
            match (x, y) {
                (1, 1) => {
                    tile.owner_color = Some("red".to_string());
                    tile.troops = 5;
                }
                (5, 1) => {
                    tile.owner_color = Some("green".to_string());
                    tile.troops = 5;
                }
                (1, 5) => {
                    tile.owner_color = Some("yellow".to_string());
                    tile.troops = 5;
                }
                (5, 5) => {
                    tile.owner_color = Some("orange".to_string());
                    tile.troops = 5;
                }
                (3, 3) => {
                    tile.owner_color = Some("purple".to_string());
                    tile.troops = 5;
                }
                _ => {}
            }

            ctx.db.tile().insert(tile);
        }
    }

    // Set up the timer to increment turns every 10 seconds
    ctx.db.next_turn_timer().insert(NextTurnTimer {
        scheduled_id: 0,
        scheduled_at: spacetimedb::ScheduleAt::Interval(Duration::from_secs(5).into()),
    });

    // Initialize the deck of cards
    create_new_deck(ctx);
}

#[reducer]
pub fn join_game(ctx: &ReducerContext, color: String) {
    // First check if the color is valid
    if !PLAYER_COLORS.contains(&color.as_str()) {
        return;
    }

    // Check if the color is already assigned
    if let Some(player) = ctx.db.player().color().find(&color) {
        if player.identity.is_some() {
            return; // Color is already assigned
        }
    }

    // Update the player with the new identity
    if let Some(player) = ctx.db.player().color().find(&color) {
        ctx.db.player().color().update(Player {
            identity: Some(ctx.sender),
            online: true,
            ..player
        });
    }
}

#[reducer(client_disconnected)]
pub fn client_disconnected(ctx: &ReducerContext) {
    // Find the player with this identity and mark them as offline
    for player in ctx.db.player().iter() {
        if player.identity == Some(ctx.sender) {
            ctx.db.player().color().update(Player {
                online: false,
                ..player
            });
            break;
        }
    }
}

#[reducer]
pub fn next_turn(ctx: &ReducerContext, _timer: NextTurnTimer) {
    // Get the current game state
    if let Some(game) = ctx.db.game().iter().next() {
        let mut current_index = game.current_player_index;
        let mut current_turn = game.turn;

        // Get the current player's color
        let current_color = PLAYER_COLORS[current_index as usize];
        log::info!(
            "Current player: {} (index: {})",
            current_color,
            current_index
        );

        // Award 2 gold and 1 stamina to the current player if they are online
        if let Some(player) = ctx.db.player().color().find(&current_color.to_string()) {
            let player_color = player.color.clone();
            let new_gold = player.gold + 2;
            let new_stamina = (player.stamina + 1).min(2); // Cap at 2

            // Update player's gold and stamina
            ctx.db.player().color().update(Player {
                gold: new_gold,
                stamina: new_stamina,
                ..player.clone()
            });
            log::info!(
                "Awarded 2 gold and 1 stamina to player {}. New totals: gold={}, stamina={}",
                player_color,
                new_gold,
                new_stamina
            );

            // Deal 2 cards to the current player if they are online
            for _ in 0..2 {
                // Find a random card in the deck
                let available_cards: Vec<Card> = ctx
                    .db
                    .card()
                    .iter()
                    .filter(|card| card.owner_color.is_none())
                    .collect();

                if available_cards.is_empty() {
                    // Deck is empty, create a new deck
                    log::info!("Deck depleted, creating new deck...");
                    // Delete all cards that are in the deck (owner_color is None)
                    for card in ctx
                        .db
                        .card()
                        .iter()
                        .filter(|card| card.owner_color.is_none())
                    {
                        ctx.db.card().card_id().delete(&card.card_id);
                    }
                    // Create a new deck of cards
                    create_new_deck(ctx);
                    // Get the newly created cards
                    let available_cards: Vec<Card> = ctx
                        .db
                        .card()
                        .iter()
                        .filter(|card| card.owner_color.is_none())
                        .collect();
                }

                if let Some(card) =
                    available_cards.get(ctx.rng().gen_range(0..available_cards.len()))
                {
                    // Deal card to player
                    let mut card = (*card).clone();
                    let suit = card.suit.clone();
                    let value = card.value;
                    let player_color = player.color.clone();

                    card.owner_color = Some(player_color.clone());
                    ctx.db.card().card_id().update(card);

                    // Log the dealt card
                    let value_str = match value {
                        1 => "Ace",
                        2 => "2",
                        3 => "3",
                        4 => "4",
                        5 => "5",
                        6 => "6",
                        7 => "7",
                        8 => "8",
                        9 => "9",
                        10 => "10",
                        11 => "Jack",
                        12 => "Queen",
                        13 => "King",
                        _ => "Unknown",
                    };
                    log::info!("Dealt {} of {} to player {}", value_str, suit, player_color);
                }
            }
        }

        // Move to the next player (we have 5 players total)
        current_index = (current_index + 1) % PLAYER_COLORS.len() as u32;

        // If we've gone through all players, increment the turn number
        if current_index == 0 {
            current_turn += 1;
            log::info!("Turn {} has begun!", current_turn);
        }

        // Update the game state
        ctx.db.game().game_name().update(Game {
            game_name: "main".to_string(),
            current_player_index: current_index,
            turn: current_turn,
        });
    }
}

#[reducer]
pub fn build_infantry(ctx: &ReducerContext, x: u32, y: u32) -> Result<(), String> {
    // Get the current player
    let player = ctx
        .db
        .player()
        .iter()
        .find(|p| p.identity == Some(ctx.sender))
        .ok_or("Player not found")?;
    let player_color = player.color.clone();

    // Get the target tile
    let tile = ctx
        .db
        .tile()
        .iter()
        .find(|t| t.x == x && t.y == y)
        .ok_or("Tile not found")?;

    // Check if player owns the tile
    if tile.owner_color.as_ref() != Some(&player_color) {
        return Err("You don't own this tile".into());
    }

    // Check if player has enough gold
    if player.gold < 1 {
        return Err("Not enough gold".into());
    }

    // Update player's gold and tile's troops
    ctx.db.player().color().update(Player {
        gold: player.gold - 1,
        ..player
    });

    ctx.db.tile().tile_id().update(Tile {
        troops: tile.troops + 1,
        ..tile
    });

    log::info!(
        "Player {} built 1 infantry on tile ({}, {})",
        player_color,
        x,
        y
    );
    Ok(())
}

/// Handles an attack between two tiles on the game board.
///
/// # Arguments
/// * `ctx` - The reducer context containing database access and sender information
/// * `from_tile_id` - The ID of the tile initiating the attack (source tile)
/// * `to_tile_id` - The ID of the tile being attacked (target tile)
///
/// # Returns
/// * `Result<(), String>` - Ok(()) if the attack succeeds, Err with message if it fails
///
/// # Attack Rules
/// 1. Source tile must be owned by the attacking player
/// 2. Source tile must have at least 2 troops
/// 3. Attack power must be greater than defense
/// 4. For unowned tiles, defense = NATURAL_DEFENSE + troops
/// 5. For owned tiles, defense = troops only
/// 6. After successful attack:
///    - Source tile keeps 1 troop
///    - Target tile is captured and gets remaining troops
#[spacetimedb::reducer]
pub fn attack(ctx: &ReducerContext, from_tile_id: u32, to_tile_id: u32) -> Result<(), String> {
    // Step 1: Retrieve the source and target tiles from the database
    // Returns error if either tile doesn't exist
    let from_tile = ctx
        .db
        .tile()
        .tile_id()
        .find(&from_tile_id)
        .ok_or("Source tile not found")?;
    let to_tile = ctx
        .db
        .tile()
        .tile_id()
        .find(&to_tile_id)
        .ok_or("Destination tile not found")?;

    // Step 2: Get the current player's information
    // Returns error if player not found
    let player = ctx
        .db
        .player()
        .iter()
        .find(|p| p.identity == Some(ctx.sender))
        .ok_or("Player not found")?;
    let player_color = player.color.clone();

    // Step 3: Verify the attacking player owns the source tile
    if from_tile.owner_color.as_ref() != Some(&player_color) {
        return Err("You can only attack from your own tiles".to_string());
    }

    // Step 4: Verify the source tile has enough troops to attack
    // Must have at least 2 troops (1 to leave behind, 1 to attack with)
    if from_tile.troops <= 1 {
        return Err("You need at least 2 troops to attack".to_string());
    }

    // Step 5: Calculate attack power and defense
    // Each tank counts as 2 attack power
    let attack_power = from_tile.troops + (from_tile.tanks * 2);

    // For unowned tiles, only use NATURAL_DEFENSE
    // For owned tiles, use troops + (tanks * 2) just like the attacker
    let defense = if to_tile.owner_color.is_none() {
        NATURAL_DEFENSE
    } else {
        to_tile.troops + (to_tile.tanks * 2)
    };

    // Step 6: Verify the attack is strong enough to succeed
    if attack_power <= defense {
        return Err("Attack power must be greater than defense".to_string());
    }

    // Step 7: Calculate losses and remaining troops
    let total_losses = attack_power - defense;

    // Tanks take losses first, then troops
    let mut remaining_tanks = from_tile.tanks;
    let mut remaining_troops = from_tile.troops;
    let mut losses_remaining = total_losses;

    // First, tanks take losses (each tank absorbs 2 damage)
    while losses_remaining >= 2 && remaining_tanks > 0 {
        remaining_tanks -= 1;
        losses_remaining -= 2;
    }

    // Then troops take any remaining losses
    if losses_remaining > 0 {
        remaining_troops = remaining_troops.saturating_sub(losses_remaining);
    }

    // Step 8: Update the source tile to leave 1 troop behind
    ctx.db.tile().tile_id().update(Tile {
        troops: 1, // Leave 1 troop in source tile
        tanks: remaining_tanks,
        ..from_tile
    });

    // Step 9: Update the target tile with new owner and remaining troops
    ctx.db.tile().tile_id().update(Tile {
        owner_color: Some(player_color.clone()),
        troops: remaining_troops,
        tanks: from_tile.tanks, // Tanks transfer to the captured tile
        ..to_tile
    });

    Ok(())
}

/// Builds a tank on a tile by spending a pair of cards with the same number.
///
/// # Arguments
/// * `ctx` - The reducer context containing database access and sender information
/// * `x` - The x coordinate of the target tile
/// * `y` - The y coordinate of the target tile
/// * `card_ids` - Vector of exactly two card IDs that must be a pair (same number)
///
/// # Returns
/// * `Result<(), String>` - Ok(()) if tank is built successfully, Err with message if it fails
///
/// # Build Rules
/// 1. Target tile must be owned by the player
/// 2. Must provide exactly two cards
/// 3. Cards must be a pair (same number)
/// 4. Player must own both cards
#[spacetimedb::reducer]
pub fn build_tank(ctx: &ReducerContext, x: u32, y: u32, card_ids: Vec<u32>) -> Result<(), String> {
    // Step 1: Verify exactly two cards provided
    if card_ids.len() != 2 {
        return Err("Must provide exactly two cards".to_string());
    }

    // Step 2: Get the current player
    let player = ctx
        .db
        .player()
        .iter()
        .find(|p| p.identity == Some(ctx.sender))
        .ok_or("Player not found")?;
    let player_color = player.color.clone();

    // Step 3: Get the target tile
    let tile = ctx
        .db
        .tile()
        .iter()
        .find(|t| t.x == x && t.y == y)
        .ok_or("Tile not found")?;

    // Step 4: Verify ownership
    if tile.owner_color.as_ref() != Some(&player_color) {
        return Err("You can only build on your own tiles".to_string());
    }

    // Step 5: Get both cards and verify ownership
    let card1 = ctx
        .db
        .card()
        .card_id()
        .find(&card_ids[0])
        .ok_or("First card not found")?;
    let card2 = ctx
        .db
        .card()
        .card_id()
        .find(&card_ids[1])
        .ok_or("Second card not found")?;

    // Step 6: Verify card ownership
    if card1.owner_color.as_ref() != Some(&player_color)
        || card2.owner_color.as_ref() != Some(&player_color)
    {
        return Err("You don't own both cards".to_string());
    }

    // Step 7: Verify cards are a pair (same number)
    if card1.value != card2.value {
        return Err("Cards must be a pair (same number)".to_string());
    }

    // Step 8: Delete both cards
    ctx.db.card().card_id().delete(&card_ids[0]);
    ctx.db.card().card_id().delete(&card_ids[1]);

    // Step 9: Add one tank to the tile
    ctx.db.tile().tile_id().update(Tile {
        tanks: tile.tanks + 1,
        ..tile
    });

    Ok(())
}

/// Moves troops and tanks from one tile to an adjacent tile.
///
/// # Arguments
/// * `ctx` - The reducer context containing database access and sender information
/// * `from_tile_id` - The ID of the source tile
/// * `to_tile_id` - The ID of the destination tile
/// * `troops_to_move` - Number of troops to move
/// * `tanks_to_move` - Number of tanks to move
///
/// # Returns
/// * `Result<(), String>` - Ok(()) if move succeeds, Err with message if it fails
///
/// # Move Rules
/// 1. Source and destination tiles must be owned by the player
/// 2. Source and destination tiles must be adjacent
/// 3. Source tile must have enough troops and tanks to move
/// 4. Source tile must keep at least 1 troop after the move
#[spacetimedb::reducer]
pub fn move_units(
    ctx: &ReducerContext,
    from_tile_id: u32,
    to_tile_id: u32,
    troops_to_move: u32,
    tanks_to_move: u32,
) -> Result<(), String> {
    // Step 1: Get the current player
    let player = ctx
        .db
        .player()
        .iter()
        .find(|p| p.identity == Some(ctx.sender))
        .ok_or("Player not found")?;
    let player_color = player.color.clone();

    // Step 2: Get source and destination tiles
    let from_tile = ctx
        .db
        .tile()
        .tile_id()
        .find(&from_tile_id)
        .ok_or("Source tile not found")?;
    let to_tile = ctx
        .db
        .tile()
        .tile_id()
        .find(&to_tile_id)
        .ok_or("Destination tile not found")?;

    // Step 3: Verify ownership of both tiles
    if from_tile.owner_color.as_ref() != Some(&player_color)
        || to_tile.owner_color.as_ref() != Some(&player_color)
    {
        return Err("You must own both the source and destination tiles".to_string());
    }

    // Step 4: Verify tiles are adjacent
    let dx = from_tile.x.abs_diff(to_tile.x);
    let dy = from_tile.y.abs_diff(to_tile.y);
    if dx + dy != 1 {
        return Err("Tiles must be adjacent".to_string());
    }

    // Step 5: Verify enough units to move
    if from_tile.troops < troops_to_move {
        return Err("Not enough troops to move".to_string());
    }
    if from_tile.tanks < tanks_to_move {
        return Err("Not enough tanks to move".to_string());
    }

    // Step 6: Verify source tile keeps at least 1 troop
    if from_tile.troops - troops_to_move < 1 {
        return Err("Source tile must keep at least 1 troop".to_string());
    }

    // Step 7: Update source tile (remove units)
    ctx.db.tile().tile_id().update(Tile {
        troops: from_tile.troops - troops_to_move,
        tanks: from_tile.tanks - tanks_to_move,
        ..from_tile
    });

    // Step 8: Update destination tile (add units)
    ctx.db.tile().tile_id().update(Tile {
        troops: to_tile.troops + troops_to_move,
        tanks: to_tile.tanks + tanks_to_move,
        ..to_tile
    });

    log::info!(
        "Player {} moved {} troops and {} tanks from tile {} to tile {}",
        player_color,
        troops_to_move,
        tanks_to_move,
        from_tile_id,
        to_tile_id
    );

    Ok(())
}

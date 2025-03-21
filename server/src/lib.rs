use log;
use rand::Rng;
use spacetimedb::{reducer, table, Identity, ReducerContext, Table, Timestamp};
use std::time::Duration;

// ------------------------------------------------------------
// Constants
// ------------------------------------------------------------
const PLAYER_COLORS: &[&str] = &["red", "green", "yellow", "orange", "purple"];

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

#[spacetimedb::table(name = player, public)]
pub struct Player {
    #[primary_key]
    color: String,
    identity: Option<Identity>,
    online: bool,
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
        });
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

        // Deal 2 cards to the current player if they are online
        if let Some(player) = ctx.db.player().color().find(&current_color.to_string()) {
            // Deal 2 cards to the current player
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

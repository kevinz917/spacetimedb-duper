use spacetimedb::{reducer, table, Identity, ReducerContext, Table, Timestamp};
use std::time::Duration;

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
    turn: u64,
    current_player_index: u32,
}

#[table(name = message, public)]
pub struct Message {
    sender: Identity,
    sent: Timestamp,
    text: String,
}

#[spacetimedb::table(name = next_turn_timer, scheduled(next_turn))]
pub struct NextTurnTimer {
    #[primary_key]
    #[auto_inc]
    scheduled_id: u64,
    scheduled_at: spacetimedb::ScheduleAt,
}

#[spacetimedb::reducer(init)]
pub fn init(ctx: &ReducerContext) {
    // Initialize the game with turn 1
    ctx.db.game().insert(Game {
        turn: 1,
        current_player_index: 0,
    });

    // Initialize the 5 players with their colors
    let colors = vec!["red", "green", "yellow", "orange", "purple"];
    for color in colors {
        ctx.db.player().insert(Player {
            color: color.to_string(),
            identity: None,
            online: false,
        });
    }

    // Set up the timer to increment turns every 10 seconds
    ctx.db.next_turn_timer().insert(NextTurnTimer {
        scheduled_id: 0,
        scheduled_at: spacetimedb::ScheduleAt::Interval(Duration::from_secs(10).into()),
    });
}

#[reducer]
pub fn join_game(ctx: &ReducerContext, color: String) {
    // First check if the color is valid
    let valid_colors = vec!["red", "green", "yellow", "orange", "purple"];
    if !valid_colors.contains(&color.as_str()) {
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

        // Move to the next player (we have 5 players total)
        current_index = (current_index + 1) % 5;

        // If we've gone through all players, increment the turn number
        if current_index == 0 {
            current_turn += 1;
        }

        // Update the game state
        ctx.db.game().delete(game);
        ctx.db.game().insert(Game {
            current_player_index: current_index,
            turn: current_turn,
        });
    }
}

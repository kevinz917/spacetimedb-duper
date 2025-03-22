# Game Rules

## Tile ID Mappings

The game board uses the following tile IDs for each coordinate (x,y):

```
(0,0) -> ID: 1
(0,1) -> ID: 2
(0,2) -> ID: 3
(0,3) -> ID: 4
(0,4) -> ID: 5
(0,5) -> ID: 6
(0,6) -> ID: 7
(1,0) -> ID: 8
(1,1) -> ID: 9    [Red Player Start]
(1,2) -> ID: 10
(1,3) -> ID: 11
(1,4) -> ID: 12
(1,5) -> ID: 13
(1,6) -> ID: 14
(2,0) -> ID: 15
(2,1) -> ID: 16
(2,2) -> ID: 17
(2,3) -> ID: 18
(2,4) -> ID: 19
(2,5) -> ID: 20
(2,6) -> ID: 21
(3,0) -> ID: 22
(3,1) -> ID: 23
(3,2) -> ID: 24
(3,3) -> ID: 25    [Purple Player Start]
(3,4) -> ID: 26
(3,5) -> ID: 27
(3,6) -> ID: 28
(4,0) -> ID: 29
(4,1) -> ID: 30
(4,2) -> ID: 31
(4,3) -> ID: 32
(4,4) -> ID: 33
(4,5) -> ID: 34
(4,6) -> ID: 35
(5,0) -> ID: 36
(5,1) -> ID: 37    [Green Player Start]
(5,2) -> ID: 38
(5,3) -> ID: 39
(5,4) -> ID: 40
(5,5) -> ID: 41    [Orange Player Start]
(5,6) -> ID: 42
(6,0) -> ID: 43
(6,1) -> ID: 44
(6,2) -> ID: 45
(6,3) -> ID: 46
(6,4) -> ID: 47
(6,5) -> ID: 48
(6,6) -> ID: 49
```

## Overview

This is a turn-based strategy game where players compete to control territory on a 7x7 grid. Each player starts with a base tile and can expand their territory through various actions.

## Player Setup

- The game supports 5 players with distinct colors: red, green, yellow, orange, and purple
- Each player starts with:
  - One base tile with 5 troops
  - 0 gold
  - 0 stamina
  - No cards

## Turn Structure

1. Each player's turn lasts 5 seconds
2. At the start of their turn, players receive:
   - 2 gold
   - 1 stamina (capped at 2)
   - 2 cards from the deck
3. Players can perform actions during their turn
4. If the deck is depleted, a new deck is created

## Resources

### Gold

- Used to build infantry units
- Costs 1 gold per infantry unit
- Received at the start of each turn

### Stamina

- Maximum of 2 stamina points
- Gained 1 per turn
- Used for special actions (to be implemented)

### Cards

- Standard deck of 52 cards (4 suits, 13 values each)
- Cards are dealt at the start of each turn
- Used for building tanks (requires pairs)

## Units

### Infantry

- Basic combat unit
- Costs 1 gold to build
- Contributes 1 to attack power
- Contributes 1 to defense

### Tanks

- Advanced combat unit
- Built using pairs of cards (same number)
- Contributes 2 to attack power
- Contributes 2 to defense
- Absorb damage before infantry in combat

## Actions

### Building Infantry

- Requirements:
  - Player must own the target tile
  - Player must have at least 1 gold
- Cost: 1 gold per infantry unit

### Building Tanks

- Requirements:
  - Player must own the target tile
  - Player must have a pair of cards (same number)
  - Player must own both cards
- Cost: Two cards of the same number (consumed after use)

### Moving Units

- Requirements:
  - Player must own both source and destination tiles
  - Tiles must be adjacent (sharing an edge)
  - Source tile must have enough units to move
  - Source tile must keep at least 1 troop after the move
- Can move any combination of:
  - Infantry troops
  - Tanks

### Attacking

- Requirements:
  - Player must own the source tile
  - Source tile must have at least 2 troops
  - Attack power must be greater than defense
- Attack Power Calculation:
  - Each infantry = 1 attack power
  - Each tank = 2 attack power
- Defense Calculation:
  - For unowned tiles: NATURAL_DEFENSE (1)
  - For owned tiles: troops + (tanks × 2)
- Combat Resolution:
  1. Calculate total losses (attack power - defense)
  2. Tanks absorb damage first (each tank absorbs 2 damage)
  3. Remaining damage is applied to troops
  4. Source tile keeps 1 troop
  5. Target tile is captured and receives remaining troops
  6. Tanks transfer to the captured tile

## Victory Conditions

- To be implemented

## Game Board

- 7x7 grid
- Each tile can contain:
  - An owner (player color)
  - Infantry troops
  - Tanks
- Tiles can be:
  - Neutral (unowned)
  - Owned by a player

## Initial Setup

- Players start in fixed positions with 5 troops each:
  - Red: (1,1) with 5 troops
  - Green: (5,1) with 5 troops
  - Yellow: (1,5) with 5 troops
  - Orange: (5,5) with 5 troops
  - Purple: (3,3) with 5 troops
- All other tiles start neutral with 0 troops

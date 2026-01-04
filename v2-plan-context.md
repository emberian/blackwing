# V2 Plan Context - Lessons from Existing Codebase

## TypeScript Implementation Overview

### Core Game Mechanics (from types.ts, events.ts, actions.ts)

#### Resources System
- 5 core resources: `credits`, `fuel`, `supplies`, `hull`, `integrity`
- Resources have implicit ranges (hull 0-100, integrity 0-100)
- Some resources trigger game over at zero (hull, integrity)

#### Card System
- Cards have: `CardDefId` (static definition) vs `CardInstanceId` (player-owned instance)
- Card types: `cargo`, `crew`, `module`, `contract`, `echo`
- Card rarity: `common`, `uncommon`, `rare`, `legendary`
- Cards have tags for matching (organic, mineral, tech, contraband, etc.)
- Cards can grant tags to the ship
- Modules have slot types: `sensor`, `defense`, `cargo`, `propulsion`, `utility`
- Contracts have destination, cargo requirements, cycle limits, rewards, penalties

#### Ship State
- Named module slots (sensor, defense, cargo1, cargo2, propulsion, utility1, utility2)
- This is more specific than our generic schema approach - consider if we want schema-driven slots

#### Time System
- Simple cycle counter
- Contracts and cooldowns use cycles
- "acquiredAt" timestamps on cards

#### World/Faction System
- Ports with status (thriving, stable, declining, ruined, abandoned, unknown)
- Market modifiers per card definition per port
- Faction reputation (-100 to 100)
- Known vs visited ports

### Event/Scenelet System (from types.ts, events.ts)

#### Scenelet Structure
- Frontmatter: id, title, tags, context (journey/port/any), weight, cooldown
- Requirements: context, shipTags, crewTags, cargoTags, minResources, maxResources, factionRep, requiredFlags, excludedFlags
- Passages with prose and choices
- Choices have requirements, effects, and navigation targets

#### Effects
- Resource modification (+=, -=, =)
- Flag setting (boolean, number, string)
- Add/remove cards (removeCards supports wildcards like "cargo.*")
- Chronicle entries
- Damage (hull, integrity)
- Reputation changes

#### Tag-based Requirements
- Ship tags from equipped cards
- Crew tags from active crew
- Cargo tags from cargo in deck
- This is important: tags are DERIVED from cards, not stored directly

### Scene File Format (.scene files)

#### Frontmatter (YAML)
```yaml
---
id: journey_sera_contact
title: Sera Infestation
tags: [danger, sera, pest]
context: journey
weight: 8
cooldown: 6
requires:
  shipTags: [sensor]
  minResources:
    fuel: 10
---
```

#### Passages
```
=== intro
Prose text here.

* [Choice text] { crew.combat, credits >= 50 }
  ~ credits -= 20
  ~ flag encountered_sera = true
  ~ damage hull 5
  ~ chronicle "Title"
    Longer text description.
  -> next_passage

* [Another choice]
  -> END
```

### Key Insights for Rust Implementation

1. **Tag derivation is important**: Tags should be computed from deck cards, not stored. Use demand-driven queries.

2. **Wildcard card removal**: `removeCards cargo.*` pattern needs proper implementation

3. **Chronicle system**: Need to track narrative entries with title, text, tags, entity references

4. **Module slots are game-specific**: The slot system (sensor, defense, cargo1/2, etc.) should be schema-defined, not hardcoded

5. **Market system**: Per-port price modifiers per card definition - need IndexMap<(PortId, CardId), f64>

6. **Contract expiration**: Contracts have cycles_remaining that tick down

## Missing from Current Rust Implementation

1. **Port/Market system** - Not yet modeled
2. **Contract system** - Not yet modeled  
3. **Chronicle system** - Events have ChronicleAdded but no storage
4. **Module slot system** - Not in GameState
5. **Card condition decay** - Cards have condition field but no mechanics
6. **Upgrade paths** - CardDef.upgradesTo not modeled

## Parser Observations

### Token Types
- FRONTMATTER_DELIM (---)
- PASSAGE_HEADER (=== name)
- CHOICE_MARKER (*)
- EFFECT_MARKER (~)
- ARROW (->)
- Comparison operators (>=, <=, ==, !=, >, <)
- Assignment operators (+=, -=, =)
- Identifiers, numbers, strings

### Condition Syntax
- `{ crew.combat }` - tag check on crew
- `{ ship.sensor }` - tag check on ship  
- `{ credits >= 50 }` - resource check
- `{ !flag_name }` - negated flag
- `{ flag count > 3 }` - flag comparison

### Effect Syntax
- `~ credits += 20` - resource modification
- `~ flag name = value` - flag setting
- `~ addCard card_id` - add card
- `~ removeCards pattern*` - remove cards (wildcard)
- `~ damage hull 10` - damage resource
- `~ chronicle "Title"` followed by indented text
- `~ reputation faction += 5` - reputation change

## Considerations for Analyzer

1. **Tag computation**: Analyzer needs to understand which cards grant which tags
2. **Resource bounds**: Track min/max possible values through paths
3. **Flag type inference**: Flags can be bool/int/string - need type tracking
4. **Dead code detection**: Unreachable passages/choices
5. **Balance analysis**: Expected resource changes per scene
6. **Softlock detection**: States where no scene is eligible

## Next Steps for Implementation

1. ~~engine-core~~ - Mostly complete, needs:
   - [ ] Port/Location state
   - [ ] Chronicle storage
   - [ ] Module slot system (or make it schema-driven)

2. engine-runtime - Needs:
   - [ ] Command handler producing events
   - [ ] Event application to state  
   - [ ] Tag computation (demand-driven)
   - [ ] RNG system
   - [ ] Scene selection logic

3. holdsmith-parser - Port lexer/parser from TS:
   - [ ] Same token types
   - [ ] Same AST structure
   - [ ] Error recovery

4. holdsmith-compiler - AST -> engine-core types:
   - [ ] Validate against schema
   - [ ] Resolve navigation targets to indices
   - [ ] Type-check conditions and effects

5. holdsmith-analyzer - Z3-backed analysis:
   - [ ] Build CFG from scenes
   - [ ] Symbolic state representation
   - [ ] Reachability queries
   - [ ] Resource bound analysis

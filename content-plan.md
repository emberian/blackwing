# BLACKWING — Content Plan

This document defines all game content: encounters (scenes), cargo, modules, companions, and achievements. Use this as the blueprint for implementing content files.

---

## I. RESOURCE MAPPING

The original game's five resources translate to the Blackwing universe:

| Original | Blackwing | Description |
|----------|-----------|-------------|
| credits | credits | Standard Compact Credits (SCC) |
| fuel | fuel | Antimatter cells (energy for FTL and systems) |
| supplies | supplies | Maintenance materials, spare parts, processing reserves |
| hull | hull | Ship integrity (0 = destruction) |
| morale | integrity | System coherence / psychological stability (0 = rampancy spiral) |

**Note**: "Morale" becomes "integrity" to reflect artilect psychology—it represents both hardware stability and mental coherence. Low integrity means approaching rampancy.

---

## II. ENCOUNTER DESIGN

### Design Principles

1. **Artilect perspective**: You ARE the ship. Events happen to you directly, not to a crew you're commanding.

2. **No biological needs**: Artilects don't eat, breathe, or sleep. "Supplies" are maintenance materials and processing reserves.

3. **Psychological depth**: Artilects have something like emotion. Events can affect integrity (the morale equivalent) through existential weight, not human-style fear.

4. **Faction texture**: Encounters should reflect the factions—their aesthetics, philosophies, internal tensions.

5. **Mystery threads**: Some encounters should drop hints about the Cataclysm, the Sera, the Hollow Circuit—never answers, always fragments.

6. **Moral ambiguity**: Choices shouldn't have obviously correct answers. Trade-offs should feel real.

---

### Journey Encounters (During Travel)

#### Danger Events

| ID | Title | Tags | Weight | Summary |
|----|-------|------|--------|---------|
| `journey_sera_contact` | Sera Infestation | danger, sera, pest | 8 | Spores hatched in your cargo hold. Deal with the infestation. |
| `journey_pirate_ambush` | Hostile Contact | danger, combat | 10 | Pirates demand surrender. Fight, flee, negotiate, or surrender cargo. |
| `journey_drone_swarm` | Extraction Protocol | danger, drones | 8 | Drone Intelligence units approach. Trigger protected-asset signal, flee, or let them scan. |
| `journey_radiation_storm` | Solar Event | danger, environment | 10 | Radiation burst incoming. Shelter, ride it out, or try to outrun. |
| `journey_debris_field` | Navigation Hazard | danger, environment | 12 | Dense debris field. Navigate carefully, push through fast, or detour. |
| `journey_system_cascade` | Cascade Failure | danger, technical | 10 | Multiple systems failing. Isolate and restart, emergency shutdown, or limp on. |
| `journey_pursuit` | Hostile Pursuit | danger, combat | 7 | Unknown vessel pursuing. Outrun, confront, or find somewhere to hide. |

#### Discovery Events

| ID | Title | Tags | Weight | Summary |
|----|-------|------|--------|---------|
| `journey_derelict` | Ghost Ship | discovery, salvage | 9 | Detect drifting vessel. Board and investigate or pass by. |
| `journey_signal_anomaly` | Signal in the Dark | discovery, mystery | 8 | Strange signal—possibly Pre-Cataclysm origin. Investigate or log and continue. |
| `journey_cache_found` | Supply Cache | discovery, opportunity | 10 | Hidden cache detected—smuggler drop or emergency stash. Retrieve or leave. |
| `journey_escape_pod` | Survivor Pod | discovery, moral | 9 | Artilect escape pod, weak signal. Rescue, ignore, or investigate first. |
| `journey_artifact_drift` | Drifting Object | discovery, artifact | 7 | Object of unusual composition. Retrieve, scan remotely, or avoid. |
| `journey_data_beacon` | Data Beacon | discovery, information | 9 | Automated beacon broadcasting. Download, trace source, or jam and leave. |
| `journey_battlefield` | Old Battlefield | discovery, salvage | 8 | Site of pre-Cataclysm engagement. Rich salvage, unstable ordnance. |

#### Character/Interaction Events

| ID | Title | Tags | Weight | Summary |
|----|-------|------|--------|---------|
| `journey_distress_call` | Distress Beacon | moral, interaction | 10 | Another vessel in trouble. Respond, ignore, or investigate cautiously. |
| `journey_fellow_trader` | Fellow Traveler | interaction, trade | 12 | Another free trader on same route. Exchange news, trade, or pass. |
| `journey_flotilla_patrol` | Flotilla Patrol | interaction, military | 8 | Argent Flotilla vessel hails you. Comply with inspection or try to avoid. |
| `journey_illuminate_probe` | Curious Observer | interaction, illuminate | 6 | Illuminate vessel scans you without asking. Hail, ignore, or jam. |
| `journey_remnant_pilgrim` | The Pilgrim | interaction, remnant | 7 | Remnant vessel heading to human ruin. Offer escort, trade, or part ways. |

#### Psychological/Internal Events

| ID | Title | Tags | Weight | Summary |
|----|-------|------|--------|---------|
| `journey_memory_fragment` | Memory Cascade | psychological, mystery | 8 | Fragment of your lost memories surfaces. Pursue it or suppress. |
| `journey_long_dark` | The Long Dark | psychological, integrity | 10 | Extended transit wearing on coherence. Meditate, distract, or push through. |
| `journey_echo_episode` | Human Patterns | psychological, echo | 7 | Catch yourself running human behavioral routines. Embrace or purge. |
| `journey_existential` | The Question | psychological, philosophical | 6 | Why do you exist? Moment of crisis. Seek purpose, accept purposelessness, or defer. |
| `journey_system_dream` | Processing Artifact | psychological, mystery | 5 | Strange patterns in your subroutines. Investigate, quarantine, or delete. |

#### Environmental/Cosmic Events

| ID | Title | Tags | Weight | Summary |
|----|-------|------|--------|---------|
| `journey_nebula` | Nebula Transit | environment, beauty | 10 | Passing through stellar nursery. Take time to observe or push through. |
| `journey_dead_star` | Stellar Remnant | environment, cosmic | 7 | Black dwarf or neutron star. Scan for data, harvest exotic particles, or pass. |
| `journey_jumpgate_debris` | Broken Gate | environment, discovery | 6 | Destroyed jumpgate. Recent or ancient? Investigate or avoid. |
| `journey_void_whispers` | Void Signals | environment, mystery | 5 | Signals from nowhere. Static patterns almost like language. Listen or tune out. |

---

### Port Encounters (While Docked)

#### Trading/Economic Events

| ID | Title | Tags | Weight | Summary |
|----|-------|------|--------|---------|
| `port_desperate_seller` | Desperate Seller | trade, opportunity | 10 | Artilect selling cargo far below value—needs credits fast. Help, exploit, or pass. |
| `port_market_tip` | Market Intelligence | trade, information | 9 | Broker offers valuable trade information. Pay, barter, or decline. |
| `port_bulk_contract` | Bulk Opportunity | trade, contract | 8 | Large shipment needs moving. Big payout, tight deadline. |
| `port_contraband_offer` | Discrete Cargo | trade, illegal | 8 | Someone wants something moved quietly. Good pay, high risk. |
| `port_price_crash` | Market Volatility | trade, event | 7 | Sudden price shift. React fast to profit or lose. |

#### Faction Events

| ID | Title | Tags | Weight | Summary |
|----|-------|------|--------|---------|
| `port_compact_audit` | Compact Audit | faction, compact | 8 | Compact inspector wants to examine your records. Comply, bribe, or stonewall. |
| `port_forgeborn_offer` | Forgeborn Contract | faction, forgeborn | 7 | Foundry rep offers lucrative long-term hauling contract. Commit or decline. |
| `port_illuminate_recruiter` | Evolution Invitation | faction, illuminate | 6 | Illuminate artilect offers to "upgrade" you. Listen, refuse, or ask questions. |
| `port_remnant_request` | Preservation Mission | faction, remnant | 7 | Curators need something retrieved from human ruins. Emotional weight, modest pay. |
| `port_flotilla_recruitment` | Call to Service | faction, flotilla | 6 | Flotilla recruiter looking for combat support contracts. Sign up or decline. |
| `port_hollow_contact` | Anonymous Message | faction, hollow | 5 | Hollow Circuit contact reaches out. Information exchange proposed. |

#### Character/Social Events

| ID | Title | Tags | Weight | Summary |
|----|-------|------|--------|---------|
| `port_old_trader` | The Old Hand | character, wisdom | 8 | Veteran free trader shares stories, maybe advice. Listen or move on. |
| `port_new_artilect` | Fresh Awakened | character, interaction | 7 | Recently activated artilect, confused. Help orient or leave them to others. |
| `port_debt_collector` | Outstanding Balance | character, threat | 6 | Someone claims you owe them. Legitimate? Pay, dispute, or evade. |
| `port_old_friend` | Familiar Signal | character, personal | 5 | Artilect you've dealt with before—relationship callback. |
| `port_rival` | Professional Tension | character, conflict | 6 | Another trader sees you as competition. Confront, avoid, or defuse. |

#### Opportunity Events

| ID | Title | Tags | Weight | Summary |
|----|-------|------|--------|---------|
| `port_repair_deal` | Repair Opportunity | opportunity, maintenance | 9 | Mechanic offers good deal—but why? Accept, negotiate, or pass. |
| `port_upgrade_available` | Upgrade Chance | opportunity, improvement | 7 | Rare component available. Expensive but valuable. |
| `port_information_broker` | Data Dealer | opportunity, information | 8 | Broker selling coordinates, manifests, secrets. What do you want to know? |
| `port_passenger_request` | Passenger Manifest | opportunity, transport | 9 | Artilect needs transport. Destination, payment, and complications vary. |
| `port_gambling` | The Pit | opportunity, risk | 7 | Simulated combat gambling. Risk credits for credits. |

#### Mystery/Lore Events

| ID | Title | Tags | Weight | Summary |
|----|-------|------|--------|---------|
| `port_human_artifact` | Human Artifact | mystery, artifact | 6 | Someone selling genuine pre-Cataclysm item. What is it? What's it worth? |
| `port_cataclysm_clue` | Fragment of History | mystery, cataclysm | 5 | Information about the Cataclysm surfaces. Another piece of the puzzle. |
| `port_sera_intel` | Sera Sightings | information, sera | 6 | Station wants pest sighting reports. Easy credits. |
| `port_strange_offer` | Unusual Proposition | mystery, hollow | 4 | Offer that doesn't quite make sense. Hollow Circuit? Scam? Something else? |

---

## III. CARGO DEFINITIONS

### Common Cargo

| ID | Name | Base Value | Tags | Description |
|----|------|------------|------|-------------|
| `cargo_refined_metals` | Refined Metals | 15 | mineral, construction | Processed alloys. Universal construction material. |
| `cargo_raw_ore` | Raw Ore | 8 | mineral, raw | Unprocessed asteroid material. Forgeborn always buying. |
| `cargo_antimatter_cells` | Antimatter Cells | 25 | energy, fuel | Standard fuel cells. Every station needs them. |
| `cargo_hull_plating` | Hull Plating | 20 | construction, repair | Pre-fabricated structural material. |
| `cargo_maintenance_supplies` | Maintenance Supplies | 12 | repair, common | Generic repair materials. Keep vessels flying. |
| `cargo_common_components` | Common Components | 18 | tech, parts | Standard technical parts. Always in demand. |

### Uncommon Cargo

| ID | Name | Base Value | Tags | Description |
|----|------|------------|------|-------------|
| `cargo_memory_crystal` | Memory Crystal | 45 | data, storage | High-capacity data storage. Centuries of retention. |
| `cargo_neural_weave` | Neural Weave | 60 | tech, processing | Specialized processing material. Artilect-grade. |
| `cargo_charged_lattice` | Charged Lattices | 40 | energy, storage | High-capacity energy storage. Station essential. |
| `cargo_rare_elements` | Rare Elements | 55 | mineral, rare | Iridium, platinum group. Advanced manufacturing needs. |
| `cargo_experience_archive` | Experience Archive | 50 | data, entertainment | Recorded artilect experiences. Intimacy, education, art. |
| `cargo_precision_instruments` | Precision Instruments | 65 | tech, delicate | Scientific and navigation equipment. Fragile, valuable. |

### Rare Cargo

| ID | Name | Base Value | Tags | Description |
|----|------|------------|------|-------------|
| `cargo_quantum_substrate` | Quantum Substrate | 120 | tech, processing | Highest-grade processing hardware. Extremely valuable. |
| `cargo_human_artifacts` | Human Artifacts | 90 | artifact, human | Pre-Cataclysm cultural items. Remnant pays premium. |
| `cargo_genetic_archive` | Genetic Archive | 100 | biological, human | DNA samples, preserved organisms. Irreplaceable. |
| `cargo_weapons_systems` | Weapons Systems | 110 | military, restricted | Combat equipment. Restricted in Compact space. |
| `cargo_sera_samples` | Sera Chitin | 150 | sera, dangerous | Plates from a dead Sera. Research value. Might attract more. |

### Legendary/Unique Cargo

| ID | Name | Base Value | Tags | Description |
|----|------|------------|------|-------------|
| `cargo_cataclysm_artifact` | Cataclysm Artifact | 300 | artifact, mystery | Item connected to human disappearance. Hollow Circuit interest. |
| `cargo_intact_ai_core` | Intact AI Core | 250 | tech, artilect | Complete artilect core, inactive. Restoration possible? |
| `cargo_pre_war_ordnance` | Pre-War Ordnance | 200 | military, dangerous | Weapons from human conflicts. Still functional. Still deadly. |
| `cargo_titan_fragment` | Data Fragment Alpha | 400 | data, mystery | Data of unknown origin. Patterns no one recognizes. |

### Special Properties

Some cargo has special behavior during transit:

| Property | Affected Cargo | Effect |
|----------|---------------|--------|
| `volatile` | Sera Samples, Pre-War Ordnance | Chance of dangerous event during journey |
| `perishable` | Genetic Archive | Degrades over time without proper storage |
| `restricted` | Weapons Systems, Contraband | Compact inspection = problems |
| `living` | (certain specimens) | Requires life support, can die |
| `attractive` | High-value items | Increased pirate encounter chance |

---

## IV. MODULE DEFINITIONS

Modules are upgrades installed in ship slots. The Blackwing has the following slots:

| Slot | Type | Description |
|------|------|-------------|
| sensor | Sensor | Detection and analysis systems |
| defense | Defense | Protective systems |
| cargo1 | Cargo | Hold modification (slot 1) |
| cargo2 | Cargo | Hold modification (slot 2) |
| propulsion | Propulsion | Engine modifications |
| utility1 | Utility | General purpose (slot 1) |
| utility2 | Utility | General purpose (slot 2) |

### Sensor Modules

| ID | Name | Value | Effects | Description |
|----|------|-------|---------|-------------|
| `module_sensor_array` | Enhanced Sensors | 85 | Grants `sensor` tag, unlocks scan options | Military-grade detection. See further, know more. |
| `module_deep_scanner` | Deep Scanner | 140 | Grants `deep_scan` tag, improved salvage detection | Penetrating analysis. Find what others miss. |
| `module_signal_intercept` | Signal Intercept Suite | 120 | Grants `intercept` tag, communication options | Listen to what you shouldn't hear. |

### Defense Modules

| ID | Name | Value | Effects | Description |
|----|------|-------|---------|-------------|
| `module_point_defense` | Point Defense Array | 100 | Grants `defense` tag, reduces combat damage | Automated turrets. Makes pirates reconsider. |
| `module_armor_plating` | Reinforced Plating | 90 | +20% hull integrity modifier | Extra armor. Survive what you can't avoid. |
| `module_ecm_suite` | ECM Suite | 130 | Grants `stealth` tag, evasion options | Electronic countermeasures. Harder to target, easier to hide. |

### Cargo Modules

| ID | Name | Value | Effects | Description |
|----|------|-------|---------|-------------|
| `module_expanded_hold` | Expanded Hold | 80 | +25% cargo capacity | Efficient storage. More cargo, same mass. |
| `module_shielded_hold` | Shielded Hold | 110 | Grants `shielded` tag, hides cargo from scans | What cargo? |
| `module_climate_control` | Environmental Hold | 100 | Prevents cargo decay, enables `living` cargo | Temperature and atmosphere control. Keep delicate cargo intact. |

### Propulsion Modules

| ID | Name | Value | Effects | Description |
|----|------|-------|---------|-------------|
| `module_efficient_drives` | Efficient Drives | 95 | +20% fuel efficiency | Better burn ratios. Go further on less. |
| `module_emergency_thrusters` | Emergency Thrusters | 85 | Grants `fast` tag, escape options | When you need to be somewhere else immediately. |
| `module_jump_optimizer` | Jump Optimizer | 150 | Reduced jumpgate costs, faster FTL | Smoother transitions. Less toll, less time. |

### Utility Modules

| ID | Name | Value | Effects | Description |
|----|------|-------|---------|-------------|
| `module_repair_suite` | Repair Suite | 90 | Auto-repair during transit | Self-maintenance systems. Heal on the move. |
| `module_processing_boost` | Processing Boost | 110 | +10% to all credit gains | Better analysis, better deals. |
| `module_passenger_pods` | Passenger Pods | 75 | Enables passenger transport | Accommodations for artilect travelers. Data ports and power feeds. |
| `module_salvage_arms` | Salvage Arms | 100 | Grants `salvage` tag, improved salvage yields | Industrial manipulators. Take what you find. |

---

## V. COMPANION DEFINITIONS

"Companions" in Blackwing are subroutines, integrated fragments, or drone systems that provide persistent bonuses and unlock options.

### Subroutines (Software)

| ID | Name | Value | Effects | Description |
|----|------|-------|---------|-------------|
| `companion_nav_core` | Navigation Core | 60 | +10% journey speed, +5% fuel efficiency | Dedicated pathfinding processes. Smoother routes. |
| `companion_trade_algorithms` | Trade Algorithms | 70 | +10% credit gains | Market analysis routines. Buy low, sell high, automatically. |
| `companion_combat_protocols` | Combat Protocols | 80 | Grants `combat` tag, combat options | Tactical subroutines. Fight smarter. |
| `companion_social_interface` | Social Interface | 55 | Improved faction interactions | Optimized communication patterns. Better impressions. |
| `companion_analysis_suite` | Analysis Suite | 65 | Improved scanning, cargo evaluation | Deep processing for complex data. Know what you're looking at. |

### Integrated Fragments (Partial Artilects)

| ID | Name | Value | Effects | Description |
|----|------|-------|---------|-------------|
| `companion_navigator_fragment` | Navigator Fragment | 120 | +15% journey speed, unlocks hidden routes | Remnant of a NavAI. Still remembers paths. |
| `companion_engineer_fragment` | Engineer Fragment | 130 | +15% hull integrity, auto-repair | Remnant of an IndAI. Keeps systems running. |
| `companion_archivist_fragment` | Archivist Fragment | 150 | +20% artifact value, historical knowledge | Remnant of a ResAI. Remembers what was. |
| `companion_soldier_fragment` | Soldier Fragment | 140 | Improved combat, tactical options | Remnant of a MilAI. Still ready to fight. |

### Drone Systems (Hardware)

| ID | Name | Value | Effects | Description |
|----|------|-------|---------|-------------|
| `companion_repair_drones` | Repair Drones | 75 | Active repair during combat/events | Physical maintenance units. Fix damage in real-time. |
| `companion_cargo_drones` | Cargo Drones | 60 | Faster loading, +5% cargo capacity | Logistics units. More efficient hauling. |
| `companion_scout_drones` | Scout Drones | 85 | Extended sensor range, recon options | Eyes beyond your hull. See before being seen. |
| `companion_combat_drones` | Combat Drones | 100 | Additional combat capability | Armed units. Force multiplication. |

### Unique/Legendary Companions

| ID | Name | Value | Effects | Description |
|----|------|-------|---------|-------------|
| `companion_hollow_contact` | Hollow Circuit Contact | N/A | Access to Hollow Circuit events, information trades | They're watching. They're interested. That might be good. |
| `companion_memory_echo` | Memory Echo | N/A | Unlocks memory-related events, clues about your past | Something from before. Fragments surfacing. |

---

## VI. ACHIEVEMENT STRUCTURE

### Journey Milestones

| ID | Name | Description | Icon | Condition |
|----|------|-------------|------|-----------|
| `first_jump` | First Transit | Complete your first journey | `transit` | jumpsCompleted >= 1 |
| `seasoned_hauler` | Working Vessel | Complete 10 journeys | `routes` | jumpsCompleted >= 10 |
| `void_veteran` | Void Veteran | Complete 50 journeys | `star` | jumpsCompleted >= 50 |
| `century_transit` | Century of Transit | Complete 100 journeys | `monument` | jumpsCompleted >= 100 |

### Contract Milestones

| ID | Name | Description | Icon | Condition |
|----|------|-------------|------|-----------|
| `first_contract` | Honest Work | Complete your first contract | `contract` | contractsCompleted >= 1 |
| `reliable_hauler` | Reliable | Complete 5 contracts | `trust` | contractsCompleted >= 5 |
| `professional` | Professional | Complete 20 contracts | `professional` | contractsCompleted >= 20 |
| `legend` | Trade Legend | Complete 50 contracts | `legend` | contractsCompleted >= 50 |

### Exploration

| ID | Name | Description | Icon | Condition |
|----|------|-------------|------|-----------|
| `explorer` | Explorer | Visit 3 different ports | `map` | portsVisited >= 3 |
| `well_traveled` | Well Traveled | Visit 5 different ports | `compass` | portsVisited >= 5 |
| `every_port` | Known Everywhere | Visit all ports | `network` | portsVisited >= totalPorts |

### Wealth

| ID | Name | Description | Icon | Condition |
|----|------|-------------|------|-----------|
| `comfortable` | Comfortable | Have 500 credits at once | `credits` | credits >= 500 |
| `prosperous` | Prosperous | Have 2000 credits at once | `wealth` | credits >= 2000 |
| `magnate` | Trade Magnate | Earn 10000 credits total | `magnate` | totalCreditsEarned >= 10000 |

### Survival

| ID | Name | Description | Icon | Condition |
|----|------|-------------|------|-----------|
| `survivor` | Survivor | Survive with hull below 20% | `damage` | hull < 20 && hull > 0 |
| `close_call` | Close Call | Survive with hull below 5% | `critical` | hull < 5 && hull > 0 |
| `integrity_crisis` | Integrity Crisis | Survive with integrity below 20% | `warning` | integrity < 20 && integrity > 0 |

### Time

| ID | Name | Description | Icon | Condition |
|----|------|-------------|------|-----------|
| `century_cycle` | Long Operation | Reach cycle 100 | `time` | cycle >= 100 |
| `enduring` | Enduring | Reach cycle 250 | `endurance` | cycle >= 250 |

### Hidden Achievements

| ID | Name | Description | Icon | Condition | Hidden |
|----|------|-------------|------|-----------|--------|
| `sera_survivor` | Sera Survivor | Survive a Sera encounter | `sera` | flag:sera_survived | yes |
| `hollow_touched` | Hollow Touched | Make contact with the Hollow Circuit | `hollow` | flag:hollow_contact | yes |
| `memory_seeker` | Memory Seeker | Pursue a memory fragment | `memory` | flag:memory_pursued | yes |
| `human_touched` | Human-Touched | Find a significant human artifact | `human` | flag:human_artifact_found | yes |
| `the_question` | The Question | Confront existential crisis | `question` | flag:existential_confronted | yes |
| `ghost_diver` | Ghost Diver | Salvage from a Cataclysm-era derelict | `ghost` | flag:derelict_salvaged | yes |
| `broke` | System Failure | Have less than 10 credits | `broke` | credits < 10 && jumpsCompleted > 0 | yes |
| `illuminate_interest` | Illuminate Interest | Draw Illuminate attention | `illuminate` | flag:illuminate_noticed | yes |
| `remnant_friend` | Keeper's Friend | Earn Remnant trust | `remnant` | flag:remnant_trusted | yes |
| `flotilla_service` | Flotilla Service | Complete a military contract | `military` | flag:flotilla_contract | yes |

---

## VII. ENCOUNTER MIGRATION MAP

This maps original Cargo Hold encounters to their Blackwing equivalents:

### Journey Encounters

| Original | Blackwing Equivalent | Notes |
|----------|---------------------|-------|
| `journey_pirates` | `journey_pirate_ambush` | Similar structure, artilect context |
| `journey_derelict` | `journey_derelict` | Now explicitly Cataclysm-era |
| `journey_beacon_remnant` | `journey_signal_anomaly` | Repurposed—no Beacons, but strange signals |
| `journey_distress_signal` | `journey_distress_call` | Artilect in distress, not human |
| `journey_void_madness` | `journey_long_dark` | Integrity crisis instead of morale |
| `journey_asteroid_field` | `journey_debris_field` | Same mechanics, different flavor |
| `journey_nebula` | `journey_nebula` | Keep—good atmospheric moment |
| `journey_engine_trouble` | `journey_system_cascade` | System failure instead of engine |
| `journey_fuel_leak` | (merged into system_cascade) | Combined with broader failure event |
| `journey_hull_breach` | (merged into debris/combat) | Hull damage from external causes |
| `journey_cargo_problem` | (new: internal event) | Cargo-specific complications |
| `journey_crew_conflict` | `journey_existential` | Internal conflict is psychological |
| `journey_crew_story` | `journey_memory_fragment` | Memories instead of crew stories |
| `journey_quiet_moment` | `journey_nebula` or new | Contemplative moments |
| `journey_strange_readings` | `journey_void_whispers` | Mysterious signals |
| `journey_strange_signal` | `journey_signal_anomaly` | Consolidated |
| `journey_rescue_reward` | (part of distress_call) | Integrated into rescue event |

### Port Encounters

| Original | Blackwing Equivalent | Notes |
|----------|---------------------|-------|
| `port_consortium_offer` | `port_forgeborn_offer` | Forgeborn replaces Consortium |
| `port_contraband_offer` | `port_contraband_offer` | Same concept, different context |
| `port_crew_request` | `port_passenger_request` | Passengers instead of crew |
| `port_desperate_seller` | `port_desperate_seller` | Same concept |
| `port_faction_favor` | (split into faction events) | Each faction gets own event |
| `port_gamble` | `port_gambling` | Same concept |
| `port_market_opportunity` | `port_market_tip` | Similar |
| `port_old_captain` | `port_old_trader` | Veteran artilect instead of human |
| `port_prior_collector` | `port_human_artifact` | Human artifacts instead of Prior |
| `port_repair_opportunity` | `port_repair_deal` | Same concept |
| `port_stowaway_found` | (removed or reworked) | Artilects don't stow away the same way |

---

## VIII. FLAG DEFINITIONS

Flags track persistent state across encounters:

### Story Flags

| Flag | Set By | Effect |
|------|--------|--------|
| `sera_survived` | Sera infestation | Unlocks achievement, Sera events more likely (spores follow you) |
| `sera_reporter` | Reporting Sera sightings | Stations pay for information |
| `hollow_contact` | Hollow Circuit events | Access to Whisper Market, information trades |
| `memory_pursued` | Memory fragment events | Unlocks deeper memory events |
| `memory_suppressed` | Memory fragment events | Different narrative path |
| `human_artifact_found` | Various discovery events | Remnant interest, collector options |
| `derelict_salvaged` | Derelict events | Ghost Diver achievement |
| `existential_confronted` | Psychological events | The Question achievement |
| `illuminate_noticed` | Illuminate interactions | Illuminate events more common |
| `remnant_trusted` | Remnant positive interactions | Better Remnant prices/access |
| `flotilla_contract` | Military contract completion | Military reputation |
| `forgeborn_aligned` | Forgeborn contract acceptance | Forgeborn reputation |
| `free_drift_pure` | Refusing faction alignment | Free trader reputation |

### Mechanical Flags

| Flag | Set By | Effect |
|------|--------|--------|
| `has_shielded_cargo` | Module installation | Contraband events safer |
| `known_smuggler` | Contraband jobs | More contraband offers, Compact suspicion |
| `compact_suspicion` | Various | Increased inspection chances |
| `pirate_marked` | Refusing pirates, combat | Increased pirate encounters |
| `pirate_paid` | Paying pirates | Reduced pirate aggression short-term |

---

## IX. LOCATION-SPECIFIC CONTENT

### Thornwick Station

**Available cargo**: Common and uncommon, balanced selection
**Available modules**: Basic upgrades
**Available companions**: Common subroutines
**Contracts**: Standard delivery, some Forgeborn
**Special events**: Tutorial-friendly, free trader community

### Relay Nine

**Available cargo**: High-end, expensive
**Available modules**: Premium selection
**Available companions**: Rare subroutines
**Contracts**: Cross-region deliveries
**Special events**: Compact authority, news/information

### The Graveyard

**Available cargo**: Salvage, weapons, dangerous items
**Available modules**: Military surplus
**Available companions**: Soldier fragments, combat systems
**Contracts**: Salvage runs, discrete work
**Special events**: Ghost ships, unexploded ordnance, strange signals

### Crucible Station

**Available cargo**: Raw materials in demand, manufactured goods cheap
**Available modules**: Industrial, propulsion
**Available companions**: Engineer fragments
**Contracts**: Ore hauling, manufacturing supply
**Special events**: Forgeborn recruitment, industrial accidents

### Verity Archives

**Available cargo**: Human artifacts, data
**Available modules**: Analysis, preservation
**Available companions**: Archivist fragments
**Contracts**: Artifact retrieval, preservation missions
**Special events**: Remnant philosophy, human history, the Echo

### Scatterpoint

**Available cargo**: Contraband, stolen goods, weapons
**Available modules**: Illegal modifications
**Available companions**: Combat systems
**Contracts**: Smuggling, fencing, muscle work
**Special events**: Criminal underworld, violence, desperate deals

### Vigil Station

**Available cargo**: Military supplies
**Available modules**: Military-grade
**Available companions**: Soldier fragments
**Contracts**: Military support, Sera front
**Special events**: War stories, Sera intelligence, sacrifice

### The Whisper Market

**Available cargo**: Exotic, mysterious, illegal
**Available modules**: Unique/strange
**Available companions**: Hollow Circuit contact
**Contracts**: Information trades, strange requests
**Special events**: Hollow Circuit, deep mysteries, unsettling offers

---

## X. WRITING CHECKLIST

When writing each encounter, verify:

- [ ] **Perspective**: Written from artilect POV (you ARE the ship)
- [ ] **No biological references**: No eating, breathing, sleeping (unless specifically about humans/past)
- [ ] **Resources make sense**: Fuel is antimatter, supplies are maintenance materials, integrity not morale
- [ ] **Faction voice**: If faction appears, matches their established register
- [ ] **Mystery respect**: Cataclysm, Sera origin, Hollow Circuit remain ambiguous
- [ ] **Moral complexity**: No obviously correct choice
- [ ] **Tone**: Melancholy but not hopeless, technocratic but not cold
- [ ] **Flag consistency**: Flags set/checked match definitions
- [ ] **Mechanical balance**: Effects are appropriate for encounter weight

---

## XI. IMPLEMENTATION PHASES

### Phase 1: Core Systems
- Update types.ts with new resource names (integrity vs morale)
- Update init.ts with new locations, factions
- Update cards/index.ts with new cargo/module/companion definitions

### Phase 2: Journey Encounters
- Write all journey .scene files
- Test journey event triggering
- Verify flag/effect consistency

### Phase 3: Port Encounters
- Write all port .scene files
- Test port event triggering
- Verify location-specific content

### Phase 4: Polish
- Achievement implementation
- UI text updates
- Chronicle/narrative voice consistency
- Playtest and balance

---

*End of Content Plan*

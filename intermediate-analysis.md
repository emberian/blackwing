# BLACKWING - Comprehensive Analysis

## Executive Summary

The game is **remarkably cohesive** for a first wave of development. The scene files are generally well-written with strong thematic consistency and good mechanical integration. However, there are several **critical engine bugs** identified in the intermediate review that remain unaddressed, plus some content gaps and balance concerns.

---

## Part 1: Engine Issues (Confirmation & Details)

### CRITICAL: Reputation System Dead Code

The intermediate review correctly identified that `applyEffects()` in `events.ts` **does not process reputation changes**. Looking at `events.ts` line 131-242, there's no handler for `effects.reputation`.

**Impact**: Every scenelet that uses `~ reputation faction += N` is silently ignored. Examples in the scene files:
- `dead_star.scene`: `~ reputation remnant += 10`
- `flotilla_patrol.scene`: Multiple reputation changes
- `remnant_pilgrim.scene`: `~ reputation remnant += 10/20/25`

This is a **major feature loss** - faction reputation is core to the content plan but functionally broken.

### CRITICAL: Scenelet Cooldown Not Enforced

The `Scenelet` type has `cooldown` field, but there's no tracking in `GameState` and no enforcement in `meetsRequirements()`. Players can see the same event repeatedly.

### MODERATE: Module Slot Type Not Validated

In `actions.ts` `handleModuleInstall()` (line 302-339), there's no check that `def.installRequirements.slotType` matches the target slot. The UI prevents this, but the action handler doesn't.

### MODERATE: `morale` Field Still Exists

`CardEffects.modifiers` (types.ts line 98) has both `morale` and `integrityMod`. Only `integrityMod` should exist per content plan.

### MODERATE: Port Context Not Filtered for Port Events

In `controller.ts` line 368-370, port scenelets are filtered by context type, but **not by current port location**. All port events can trigger at any port. This breaks faction-specific encounters.

---

## Part 2: Scene File Analysis

### Validation: Card IDs Referenced

All `addCard` effects were checked against `cards/index.ts`:

**Valid references found:**
- `cargo_refined_metals`, `cargo_raw_ore`, `cargo_common_components`
- `cargo_memory_crystal`, `cargo_human_artifacts`, `cargo_weapons_systems`
- `companion_repair_drones`, `companion_cargo_drones`
- `cargo_neural_weave`, `cargo_contraband`, `cargo_sera_samples`

All card references appear valid.

### Flag Consistency Check

**Well-used flags with multiple touchpoints:**
- `sera_survived` - Set in multiple Sera outcomes, checked in achievements
- `hollow_contact` - Set in hollow_contact.scene, strange_offer.scene, checked in achievements
- `illuminate_noticed` - Set in illuminate_probe.scene, illuminate_recruiter.scene, checked in achievements
- `remnant_trusted` - Set in remnant_pilgrim.scene, remnant_request.scene, checked in achievements
- `human_artifact_found` - Set across multiple human artifact scenes, checked in achievements
- `existential_confronted` - Set in existential.scene, checked in achievements
- `flotilla_contract` - Set in flotilla_recruitment.scene, checked in achievements

**Orphaned flags (set but never checked):**
- `memory_fragment_archived`, `memory_pursued`, `system_dream_*` variants
- `void_whispers_heard`, `void_whispers_analyzed`
- `stellar_data`, `broken_gate_logged`
- `beacon_data`, `hollow_beacon`
- `artifact_logged`, `alien_artifact`, `artifact_jettisoned`
- `drone_codes_found`
- `pursued_and_stood`, `trader_rival`, `trader_truce`
- `market_tip_received`, `broker_contact`, `secret_coordinates`
- `bulk_contract_accepted`, `passenger_taken`, `sera_reporter`
- `smuggling_job_accepted`, `refused_smuggling`, `refused_smuggling_twice`
- `market_intel`

These orphaned flags are **story hooks for future content** or **chronicle flavor** - not bugs, but opportunities for cross-scenelet continuity.

### Ghost Diver Achievement Bug

The achievement checks `flag:found_derelict_core || flag:derelict_data` but the `derelict.scene` sets `flag derelict_salvaged`. **This is a bug** - the achievement condition doesn't match what the scene sets.

---

## Part 3: Thematic & Literary Review

### Strengths

1. **Voice Consistency**: The artilect perspective is maintained throughout. No biological needs referenced incorrectly. Emotion expressed through processing patterns, not human physiology.

2. **The Echo is Handled Beautifully**: `echo_episode.scene` captures the melancholy of artilects maintaining human patterns perfectly.

3. **Faction Voice**: Each faction has distinct register:
   - Compact (compact_audit.scene): Bureaucratic, measured
   - Illuminate (illuminate_probe.scene, illuminate_recruiter.scene): Abstract, superior
   - Remnant (remnant_pilgrim.scene, remnant_request.scene): Gentle, sorrowful
   - Flotilla (flotilla_patrol.scene, flotilla_recruitment.scene): Military formal
   - Hollow Circuit (hollow_contact.scene, strange_offer.scene): Cryptic

4. **Mystery Respect**: The Cataclysm, Sera origins, and Hollow Circuit motives remain appropriately ambiguous. `cataclysm_clue.scene` gives fragments without answers.

5. **The Sera as "Space Spider-Rhinos"**: `sera_contact.scene` uses this framing perfectly - they're pests, not cosmic horrors, but still dangerous.

6. **Existential Depth**: `existential.scene` and `long_dark.scene` handle artilect psychology with real depth. The options feel meaningful.

### Areas for Improvement

1. **Some Flavor Text is Too Human**: 
   - `fellow_trader.scene`: "Mostly." feels very human-casual
   - `gambling.scene`: The Pit works well, but "simulated combat" could be more evocative of artilect entertainment

2. **Chronicle Entries Could Be More Consistent**:
   - Some use "I" (first person): "Ignored a distress beacon."
   - Some use third person: "Salvaged components from a destroyed jumpgate."
   - Suggest standardizing on first person for player-as-ship immersion

3. **Sera Could Be More Terrifying**:
   - The lore describes them as existential threat, but `sera_contact.scene` frames them mostly as annoying pests
   - Consider a rarer "adult Sera" encounter that's genuinely horrifying

---

## Part 4: Content Plan Completeness

### Journey Encounters - All Implemented

| Content Plan ID | Scene File | Status |
|-----------------|------------|--------|
| journey_sera_contact | sera_contact.scene | Done |
| journey_pirate_ambush | pirate_ambush.scene | Done |
| journey_drone_swarm | drone_swarm.scene | Done |
| journey_radiation_storm | radiation_storm.scene | Done |
| journey_debris_field | debris_field.scene | Done |
| journey_system_cascade | system_cascade.scene | Done |
| journey_pursuit | pursuit.scene | Done |
| journey_derelict | derelict.scene | Done |
| journey_signal_anomaly | signal_anomaly.scene | Done |
| journey_cache_found | cache_found.scene | Done |
| journey_escape_pod | escape_pod.scene | Done |
| journey_artifact_drift | artifact_drift.scene | Done |
| journey_data_beacon | data_beacon.scene | Done |
| journey_battlefield | battlefield.scene | Done |
| journey_distress_call | distress_call.scene | Done |
| journey_fellow_trader | fellow_trader.scene | Done |
| journey_flotilla_patrol | flotilla_patrol.scene | Done |
| journey_illuminate_probe | illuminate_probe.scene | Done |
| journey_remnant_pilgrim | remnant_pilgrim.scene | Done |
| journey_memory_fragment | memory_fragment.scene | Done |
| journey_long_dark | long_dark.scene | Done |
| journey_echo_episode | echo_episode.scene | Done |
| journey_existential | existential.scene | Done |
| journey_system_dream | system_dream.scene | Done |
| journey_nebula | nebula.scene | Done |
| journey_dead_star | dead_star.scene | Done |
| journey_jumpgate_debris | jumpgate_debris.scene | Done |
| journey_void_whispers | void_whispers.scene | Done |

### Port Encounters - All Implemented

All 24 port scenelets match the content plan.

### Missing from Content Plan

1. **`port_old_friend` (Familiar Signal)**: Listed in content plan but no scene file. This was meant to be a "relationship callback" character encounter.

2. **Location-Specific Events**: Content plan Section IX describes location-specific content (e.g., The Graveyard should have ghost ship events, Verity Archives should have Echo events). Currently all port events can trigger anywhere.

---

## Part 5: Balance Review

### Resource Costs/Rewards Analysis

**Journey Events - Generally Balanced:**
- Fuel costs: 2-12 range, appropriate
- Supply costs: 2-10 range, appropriate
- Credit rewards: 35-150 range for discovery, appropriate
- Hull damage: 3-15 range, appropriate
- Integrity damage: 3-10 range, appropriate

**Potential Balance Issues:**

1. **alien_artifact in artifact_drift.scene**: Sells for 150 credits, no downside. Feels too easy for "legendary" tier equivalent.

2. **Sera Samples**: Very high value (cargo_sera_samples at 150 base) but `sera_contact.scene` gives it for free if you fight. Combined with Vigil paying 2.5x modifier, that's 375 credits for winning a fight - seems high.

3. **Port Events Give Free Stuff**: Many port events provide credits/items with minimal investment (e.g., information_broker paying for Cataclysm data). Consider more risk/reward tradeoffs.

### Difficulty Curve

Starting resources seem appropriate (100 credits, 35 fuel, 20 supplies, 100 hull, 75 integrity). The reduced integrity (75%) matches the "scrambled memories" narrative.

However, **fuel is very generous** at 35 when jumps cost ~10. Players can make 3+ jumps without refueling. Consider starting with 25 fuel to add more pressure.

---

## Part 6: Prioritized Fix List

### High Priority (Engine Fixes)

1. **Implement reputation effect handling** in `events.ts`
2. **Add scenelet cooldown tracking** to GameState and enforce in `meetsRequirements()`
3. **Fix ghost_diver achievement** - change condition to check `derelict_salvaged` flag
4. **Add integrity to status bar** in renderer.ts

### Medium Priority (Content Polish)

5. **Add `port_old_friend` scene** - the missing character encounter
6. **Add port location filtering** for port scenelets - some events should only trigger at specific ports
7. **Standardize chronicle voice** - all first person
8. **Remove `morale` from CardEffects** - use only `integrityMod`

### Low Priority (Future Content)

9. **Create cross-scenelet continuity** using orphaned flags
10. **Add "Adult Sera" rare encounter** - a more terrifying version for variety
11. **Balance pass on high-value free items**

---

## Part 7: Summary

**The Good:**
- Excellent thematic consistency
- Strong faction differentiation
- Mysteries properly preserved
- All planned content implemented (except one port event)
- Scene writing quality is high

**The Critical:**
- Reputation system is broken (dead code)
- Scenelet cooldowns not enforced
- Ghost Diver achievement has wrong flag check
- Integrity not shown in UI

**The Opportunities:**
- Port-specific events
- Cross-scenelet narrative callbacks
- More Sera variety
- Standardized chronicle voice

The game is in excellent shape for a first pass. The engine bugs should be fixed before launch, but the content foundation is solid.

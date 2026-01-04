# BLACKWING - Intermediate Review

## Completeness Assessment: Content Plan vs. Implementation

### What's Implemented Well

1. **All 8 Ports** from the lore are present in `init.ts`:
   - Thornwick Station
   - Relay Nine
   - The Graveyard
   - Crucible Station
   - Verity Archives
   - Scatterpoint
   - Vigil Station
   - The Whisper Market

2. **All 6 Factions** are initialized:
   - Continuity Compact
   - Illuminate
   - Remnant
   - Forgeborn
   - Argent Flotilla
   - Hollow Circuit

3. **Cargo, Modules, Companions, Contracts** - The `cards/index.ts` has extensive implementation matching the content plan with 20 cargo types, 16 companions, 16 modules, and 11 contracts.

4. **All planned Journey & Port scenelets appear to be created** based on the index.ts exports:
   - 28 Journey scenelets
   - 24 Port scenelets

5. **Achievements** are well-implemented with 31 achievements including hidden achievements tied to story flags.

---

## Potential Gaps & Issues

### Critical Issues

1. **Reputation System Not Applied**: The faction reputation exists in types and contracts have `reputationReward`, but `applyEffects()` in `events.ts` doesn't process reputation changes. This is dead code.

2. **Scenelet Cooldown Not Enforced**: `Scenelet` has a `cooldown` field but it's never tracked or enforced. Scenelets can repeat indefinitely, breaking immersion.

3. **Module Slot Type Validation Missing**: In `handleModuleInstall()`, there's no server-side verification that the module's `slotType` matches the target slot. The renderer blocks this in UI, but the action handler doesn't enforce it—a bug waiting to happen.

### Moderate Issues

4. **Resource Naming Inconsistency**: The content plan calls "morale" -> "integrity", but `CardEffects.modifiers` has both `morale` and `integrityMod` fields. Only `integrityMod` should exist.

5. **Location-Specific Events Not Filtered**: Port scenelets don't check what port the player is at. All port events can trigger at any port, which breaks faction-specific encounters.

6. **"Echo" Card Type Orphaned**: The `RELIC_CARDS` use type `'echo'` but this isn't handled specially anywhere—they're just filtered as cargo in the hold view. Should have special treatment.

7. **Chronicle Too Sparse**: Only arrival/departure/milestones are logged. Event choices and outcomes don't create chronicle entries unless `effects.addChronicle` is explicitly specified in each choice.

### Minor Issues

8. **Living/Perishable Cargo Incomplete**: The content plan mentions `volatile`, `perishable`, `restricted`, `living`, `attractive` cargo properties but only `decayChance` and `eventChance` are implemented in `journeyBehavior`.

9. **Integrity Not in Status Bar**: The status bar shows credits, fuel, supplies, hull—but not integrity, which is a core resource and game-over condition.

10. **No Cargo Decay Warning**: Players aren't warned before travel that they have perishable cargo that might decay.

---

## Lore Fidelity Observations

1. **lore.txt vs game-specific-lore.md**: The `lore.txt` is from "Silicon Dawn" (a different game!) with Terran Colonial Federation, SOLCOM, etc. The `game-specific-lore.md` (Blackwing plot bible) is the canonical reference. They share artilect concepts but are different settings. `lore.txt` appears to be inspiration material, not canon.

2. **Player Starting Integrity at 75%**: The opening text mentions "woke 31 years ago...memories scrambled" which matches starting with reduced integrity (75 vs 100). Good thematic integration.

3. **Hidden Achievement Integration**: `the_question` achievement ties to `journey_existential` scenelet, `sera_survivor` ties to Sera encounters, `ghost_diver` ties to derelict exploration. Well-designed story-gameplay connection.

4. **Ship Name Choice**: "Blackwing" with flavor text about "scorched hull plating with iridescent darkness that catches light like feathers" is evocative and fits the tone.

---

## Engine/Compiler Review Questions for Scene Files

When reviewing the scene files, check:

- [ ] Do all `addCards` effects reference valid card IDs from `cards/index.ts`?
- [ ] Are flags set/checked consistently across related scenelets?
- [ ] Do choice requirements actually filter correctly?
- [ ] Is the tone consistent (melancholy but not hopeless, artilect POV)?
- [ ] Are there any biological references that break immersion (eating, breathing, sleeping)?
- [ ] Do faction interactions match their established registers?
  - Compact: Bureaucratic, measured
  - Illuminate: Abstract, superior
  - Remnant: Gentle, sorrowful
  - Forgeborn: Direct, production-focused
  - Flotilla: Military formal
  - Hollow Circuit: Cryptic, economical
  - Free Traders: Casual, practical
- [ ] Do resource costs/rewards feel balanced?
- [ ] Are mystery elements (Cataclysm, Sera, Hollow Circuit) kept ambiguous?

---

## Suggested Improvements

### Engine Fixes (Priority Order)

1. **Add reputation effect application** in `applyEffects()`:
```typescript
if (effects.reputation) {
  const faction = newState.world.factions[effects.reputation.faction];
  if (faction) {
    newState.world.factions = {
      ...newState.world.factions,
      [effects.reputation.faction]: {
        ...faction,
        reputation: Math.max(-100, Math.min(100, faction.reputation + effects.reputation.amount)),
      },
    };
  }
}
```

2. **Add scenelet cooldown tracking**:
   - Add `sceneletCooldowns: Record<SceneletId, number>` to GameState
   - Check cooldown in `meetsRequirements()`
   - Set cooldown on scenelet trigger

3. **Add location-based event filtering**:
   - Add `portId?: PortId` to `SceneletRequirements`
   - Check in `meetsRequirements()`

4. **Remove `morale` from CardEffects.modifiers** - use only `integrityMod`

### UI Improvements

5. **Add integrity to status bar** with visual warning at low values

6. **Add cargo warnings before travel** for volatile/perishable items

7. **Auto-chronicle event outcomes** when significant effects occur

### Content Improvements

8. **Add port-specific scenelets** that only trigger at certain locations

9. **Add faction reputation thresholds** that unlock/block certain events

10. **Add "Echo" card special handling** - perhaps they provide unique event options or passive bonuses

---

## Files Reviewed

- `src/core/types.ts` - Type definitions
- `src/core/actions.ts` - Action handlers
- `src/core/controller.ts` - Game controller
- `src/core/events.ts` - Event selection and effects
- `src/core/init.ts` - Initial game state
- `src/core/persistence.ts` - Save/load
- `src/core/simulate.ts` - Journey simulation
- `src/ui/renderer.ts` - UI rendering
- `src/ui/styles.css` - Styling
- `src/main.ts` - Entry point
- `src/content/cards/index.ts` - Card definitions
- `src/content/achievements/index.ts` - Achievement definitions
- `src/content/scenelets/index.ts` - Scenelet registry
- `content-plan.md` - Content blueprint
- `game-specific-lore.md` - Plot bible (canonical)
- `lore.txt` - Inspiration material (not canonical)

---

## Next Steps

1. Review all `.scene` files for thematic consistency and mechanical correctness
2. Verify flag usage consistency across scenelets
3. Check card ID references in scenelet effects
4. Assess balance of resource rewards/costs
5. Identify any missing content from content-plan.md

/**
 * Holdsmith Analyzer
 *
 * Static and dynamic analysis for .scene files and game integration.
 * Catches issues like:
 * - Invalid card/faction references
 * - Orphaned/duplicate flags
 * - Unreachable passages
 * - Balance anomalies
 * - Achievement/flag mismatches
 * - Thematic inconsistencies
 */

import type { SceneFile, Effect, Passage, Choice, PassageContent } from '../parser/ast.js';
import type { CompiledScenelet } from '../compiler/compiler.js';

// =============================================================================
// ANALYSIS TYPES
// =============================================================================

export type Severity = 'error' | 'warning' | 'info' | 'hint';

export interface AnalysisDiagnostic {
  readonly severity: Severity;
  readonly code: string;
  readonly message: string;
  readonly file?: string;
  readonly line?: number;
  readonly suggestion?: string;
}

export interface AnalysisContext {
  /** All known card definition IDs */
  readonly cardIds: Set<string>;
  /** All known faction IDs */
  readonly factionIds: Set<string>;
  /** All known port IDs */
  readonly portIds: Set<string>;
  /** All known achievement flag checks */
  readonly achievementFlags: Set<string>;
  /** Resource names */
  readonly resourceNames: Set<string>;
  /** All compiled scenelets for cross-reference */
  readonly scenelets: Map<string, CompiledScenelet>;
}

export interface AnalysisResult {
  readonly diagnostics: AnalysisDiagnostic[];
  readonly stats: AnalysisStats;
}

export interface AnalysisStats {
  readonly filesAnalyzed: number;
  readonly totalPassages: number;
  readonly totalChoices: number;
  readonly flagsSet: Set<string>;
  readonly flagsChecked: Set<string>;
  readonly cardsReferenced: Set<string>;
  readonly factionsReferenced: Set<string>;
  readonly resourceEffects: ResourceEffectSummary[];
}

export interface ResourceEffectSummary {
  readonly sceneId: string;
  readonly resource: string;
  readonly minDelta: number;
  readonly maxDelta: number;
}

// =============================================================================
// DEFAULT CONTEXT (Blackwing-specific)
// =============================================================================

export function createBlackwingContext(): AnalysisContext {
  return {
    cardIds: new Set([
      // Cargo
      'cargo_raw_ore', 'cargo_refined_metals', 'cargo_antimatter_cells',
      'cargo_hull_plating', 'cargo_maintenance_supplies', 'cargo_common_components',
      'cargo_memory_crystal', 'cargo_neural_weave', 'cargo_charged_lattice',
      'cargo_rare_elements', 'cargo_experience_archive', 'cargo_precision_instruments',
      'cargo_quantum_substrate', 'cargo_human_artifacts', 'cargo_genetic_archive',
      'cargo_weapons_systems', 'cargo_sera_samples', 'cargo_cataclysm_artifact',
      'cargo_intact_ai_core', 'cargo_contraband',
      // Companions
      'companion_nav_core', 'companion_trade_algorithms', 'companion_combat_protocols',
      'companion_social_interface', 'companion_analysis_suite', 'companion_navigator_fragment',
      'companion_engineer_fragment', 'companion_archivist_fragment', 'companion_soldier_fragment',
      'companion_repair_drones', 'companion_cargo_drones', 'companion_scout_drones',
      'companion_combat_drones', 'companion_hollow_contact', 'companion_memory_echo',
      // Modules
      'module_sensor_array', 'module_deep_scanner', 'module_signal_intercept',
      'module_point_defense', 'module_armor_plating', 'module_ecm_suite',
      'module_expanded_hold', 'module_shielded_hold', 'module_climate_control',
      'module_efficient_drives', 'module_emergency_thrusters', 'module_jump_optimizer',
      'module_repair_suite', 'module_processing_boost', 'module_passenger_pods',
      'module_salvage_arms',
      // Contracts
      'contract_standard_delivery', 'contract_ore_shipment', 'contract_artifact_retrieval',
      'contract_discrete_cargo', 'contract_military_support', 'contract_forgeborn_supply',
      'contract_salvage_run', 'contract_cross_region', 'contract_preservation_mission',
      'contract_sera_front', 'contract_information_exchange',
      // Relics
      'relic_cataclysm_shard', 'relic_sera_fragment', 'relic_makers_voice', 'relic_memory_core',
    ]),
    factionIds: new Set([
      'compact', 'illuminate', 'remnant', 'forgeborn', 'flotilla', 'hollow',
      'faction_compact', 'faction_illuminate', 'faction_remnant',
      'faction_forgeborn', 'faction_flotilla', 'faction_hollow',
    ]),
    portIds: new Set([
      'port_thornwick', 'port_relay_nine', 'port_graveyard', 'port_crucible',
      'port_verity', 'port_scatterpoint', 'port_vigil', 'port_whisper',
    ]),
    achievementFlags: new Set([
      'sera_survived', 'hollow_contact', 'memory_pursued', 'human_artifact_found',
      'existential_confronted', 'derelict_salvaged', 'illuminate_noticed',
      'remnant_trusted', 'flotilla_contract', 'found_derelict_core', 'derelict_data',
    ]),
    resourceNames: new Set(['credits', 'fuel', 'supplies', 'hull', 'integrity', 'morale']),
    scenelets: new Map(),
  };
}

// =============================================================================
// ANALYZER
// =============================================================================

export class SceneAnalyzer {
  private diagnostics: AnalysisDiagnostic[] = [];
  private flagsSet = new Set<string>();
  private flagsChecked = new Set<string>();
  private cardsReferenced = new Set<string>();
  private factionsReferenced = new Set<string>();
  private resourceEffects: ResourceEffectSummary[] = [];
  private totalPassages = 0;
  private totalChoices = 0;

  constructor(private context: AnalysisContext) {}

  /**
   * Analyze a single parsed scene file
   */
  analyzeScene(scene: SceneFile, filename?: string): void {
    const file = filename ?? scene.frontmatter.id;

    // Frontmatter checks
    this.checkFrontmatter(scene, file);

    // Passage analysis
    const passageNames = new Set<string>();
    const passageRefs = new Set<string>();

    for (const passage of scene.passages) {
      this.totalPassages++;
      passageNames.add(passage.name);
      this.analyzePassage(passage, file, passageRefs);
    }

    // Check for unreachable passages
    this.checkUnreachablePassages(passageNames, passageRefs, scene, file);

    // Check for duplicate passage names
    this.checkDuplicatePassages(scene, file);
  }

  /**
   * Analyze a compiled scenelet (post-compilation)
   */
  analyzeCompiled(scenelet: CompiledScenelet, filename?: string): void {
    const file = filename ?? scenelet.id;

    for (const passage of scenelet.passages) {
      if (passage.choices) {
        for (const choice of passage.choices) {
          // Check addCards references
          if (choice.effects.addCards) {
            for (const cardId of choice.effects.addCards) {
              this.cardsReferenced.add(cardId);
              if (!this.context.cardIds.has(cardId)) {
                this.addDiagnostic({
                  severity: 'error',
                  code: 'INVALID_CARD_REF',
                  message: `Unknown card ID: "${cardId}"`,
                  file,
                  suggestion: this.suggestSimilar(cardId, this.context.cardIds),
                });
              }
            }
          }

          // Check reputation faction references
          if (choice.effects.reputation) {
            const faction = choice.effects.reputation.faction;
            this.factionsReferenced.add(faction);
            if (!this.context.factionIds.has(faction)) {
              this.addDiagnostic({
                severity: 'error',
                code: 'INVALID_FACTION_REF',
                message: `Unknown faction ID: "${faction}"`,
                file,
                suggestion: this.suggestSimilar(faction, this.context.factionIds),
              });
            }
          }

          // Check flags
          if (choice.effects.setFlags) {
            for (const flag of Object.keys(choice.effects.setFlags)) {
              this.flagsSet.add(flag);
            }
          }

          // Check requirements for flag checks
          if (choice.requirements?.requiredFlags) {
            for (const flag of choice.requirements.requiredFlags) {
              this.flagsChecked.add(flag);
            }
          }
          if (choice.requirements?.excludedFlags) {
            for (const flag of choice.requirements.excludedFlags) {
              this.flagsChecked.add(flag);
            }
          }
        }
      }
    }

    // Check scenelet-level requirements
    if (scenelet.requirements.requiredFlags) {
      for (const flag of scenelet.requirements.requiredFlags) {
        this.flagsChecked.add(flag);
      }
    }
    if (scenelet.requirements.excludedFlags) {
      for (const flag of scenelet.requirements.excludedFlags) {
        this.flagsChecked.add(flag);
      }
    }
  }

  /**
   * Run cross-scenelet analysis (after all scenes analyzed)
   */
  finalize(): AnalysisResult {
    // Check for orphaned flags (set but never checked)
    const orphanedFlags = new Set<string>();
    for (const flag of this.flagsSet) {
      if (!this.flagsChecked.has(flag) && !this.context.achievementFlags.has(flag)) {
        orphanedFlags.add(flag);
      }
    }

    if (orphanedFlags.size > 0) {
      this.addDiagnostic({
        severity: 'info',
        code: 'ORPHANED_FLAGS',
        message: `Flags set but never checked (excluding achievements): ${[...orphanedFlags].join(', ')}`,
        suggestion: 'Consider using these flags in requirements or removing them',
      });
    }

    // Check for checked but never set flags
    const neverSetFlags = new Set<string>();
    for (const flag of this.flagsChecked) {
      if (!this.flagsSet.has(flag)) {
        neverSetFlags.add(flag);
      }
    }

    if (neverSetFlags.size > 0) {
      this.addDiagnostic({
        severity: 'warning',
        code: 'UNSET_FLAG_CHECK',
        message: `Flags checked but never set: ${[...neverSetFlags].join(', ')}`,
        suggestion: 'These conditions may never be true',
      });
    }

    // Check achievement flag alignment
    for (const achievementFlag of this.context.achievementFlags) {
      if (!this.flagsSet.has(achievementFlag)) {
        this.addDiagnostic({
          severity: 'warning',
          code: 'ACHIEVEMENT_FLAG_MISSING',
          message: `Achievement expects flag "${achievementFlag}" but it's never set in scenes`,
          suggestion: 'Add a scene that sets this flag, or update the achievement condition',
        });
      }
    }

    return {
      diagnostics: this.diagnostics,
      stats: {
        filesAnalyzed: this.context.scenelets.size,
        totalPassages: this.totalPassages,
        totalChoices: this.totalChoices,
        flagsSet: this.flagsSet,
        flagsChecked: this.flagsChecked,
        cardsReferenced: this.cardsReferenced,
        factionsReferenced: this.factionsReferenced,
        resourceEffects: this.resourceEffects,
      },
    };
  }

  private checkFrontmatter(scene: SceneFile, file: string): void {
    const fm = scene.frontmatter;

    // Check for missing ID
    if (!fm.id || fm.id.trim() === '') {
      this.addDiagnostic({
        severity: 'error',
        code: 'MISSING_ID',
        message: 'Scene is missing an ID in frontmatter',
        file,
        line: fm.span.start.line,
      });
    }

    // Check for missing title
    if (!fm.title || fm.title.trim() === '') {
      this.addDiagnostic({
        severity: 'warning',
        code: 'MISSING_TITLE',
        message: 'Scene is missing a title in frontmatter',
        file,
        line: fm.span.start.line,
      });
    }

    // Check weight is reasonable
    if (fm.weight <= 0) {
      this.addDiagnostic({
        severity: 'warning',
        code: 'ZERO_WEIGHT',
        message: 'Scene has zero or negative weight - it will never appear',
        file,
        line: fm.span.start.line,
      });
    }

    // Check cooldown is reasonable
    if (fm.cooldown < 0) {
      this.addDiagnostic({
        severity: 'warning',
        code: 'NEGATIVE_COOLDOWN',
        message: 'Scene has negative cooldown',
        file,
        line: fm.span.start.line,
      });
    }
  }

  private analyzePassage(passage: Passage, file: string, passageRefs: Set<string>): void {
    let hasChoices = false;

    for (const content of passage.content) {
      if (content.type === 'Choice') {
        hasChoices = true;
        this.totalChoices++;
        this.analyzeChoice(content, file, passageRefs);
      } else if (content.type === 'Prose') {
        this.analyzeProse(content.text, file, content.span.start.line);
      }
    }

    // Warn if passage has no choices and isn't a terminal passage
    if (!hasChoices && passage.content.length > 0) {
      this.addDiagnostic({
        severity: 'hint',
        code: 'NO_CHOICES',
        message: `Passage "${passage.name}" has no choices - it will show as a continue prompt`,
        file,
        line: passage.span.start.line,
      });
    }
  }

  private analyzeChoice(choice: Choice, file: string, passageRefs: Set<string>): void {
    // Track navigation targets
    if (choice.target && !choice.target.isEnd) {
      passageRefs.add(choice.target.target);
    }

    // Analyze effects
    for (const effect of choice.effects) {
      this.analyzeEffect(effect, file);
    }

    // Check for empty choice text
    if (!choice.text || choice.text.trim() === '') {
      this.addDiagnostic({
        severity: 'warning',
        code: 'EMPTY_CHOICE',
        message: 'Choice has empty text',
        file,
        line: choice.span.start.line,
      });
    }
  }

  private analyzeEffect(effect: Effect, file: string): void {
    switch (effect.type) {
      case 'AddCardEffect':
        this.cardsReferenced.add(effect.cardId);
        if (!this.context.cardIds.has(effect.cardId)) {
          this.addDiagnostic({
            severity: 'error',
            code: 'INVALID_CARD_REF',
            message: `Unknown card ID: "${effect.cardId}"`,
            file,
            line: effect.span.start.line,
            suggestion: this.suggestSimilar(effect.cardId, this.context.cardIds),
          });
        }
        break;

      case 'FlagEffect':
        this.flagsSet.add(effect.flag);
        break;

      case 'ReputationEffect':
        this.factionsReferenced.add(effect.faction);
        if (!this.context.factionIds.has(effect.faction)) {
          this.addDiagnostic({
            severity: 'error',
            code: 'INVALID_FACTION_REF',
            message: `Unknown faction ID: "${effect.faction}"`,
            file,
            line: effect.span.start.line,
            suggestion: this.suggestSimilar(effect.faction, this.context.factionIds),
          });
        }
        break;

      case 'ResourceEffect':
        // Check for potentially unbalanced effects
        if (effect.resource === 'credits' && effect.operator === '+=' && effect.value > 200) {
          this.addDiagnostic({
            severity: 'info',
            code: 'HIGH_CREDIT_REWARD',
            message: `High credit reward: +${effect.value}`,
            file,
            line: effect.span.start.line,
            suggestion: 'Consider if this reward is balanced with risk/effort',
          });
        }
        break;

      case 'DamageEffect':
        // Check for very high damage
        if (effect.value > 20) {
          this.addDiagnostic({
            severity: 'info',
            code: 'HIGH_DAMAGE',
            message: `High ${effect.target} damage: ${effect.value}`,
            file,
            line: effect.span.start.line,
          });
        }
        break;
    }
  }

  private analyzeProse(text: string, file: string, line: number): void {
    // Check for potentially problematic patterns

    // Biological references (artilects don't eat/sleep/breathe)
    const bioPatterns = [
      /\b(eat|eating|hungry|food|meal|breakfast|lunch|dinner)\b/i,
      /\b(sleep|sleeping|tired|exhausted|rest|nap)\b/i,
      /\b(breathe|breathing|breath|air|oxygen)\b/i,
      /\b(heart|heartbeat|pulse|blood|sweat)\b/i,
    ];

    for (const pattern of bioPatterns) {
      if (pattern.test(text)) {
        this.addDiagnostic({
          severity: 'hint',
          code: 'BIOLOGICAL_REFERENCE',
          message: `Text may contain biological reference inappropriate for artilects`,
          file,
          line,
          suggestion: 'Artilects don\'t eat, sleep, or breathe - consider rephrasing',
        });
        break;
      }
    }

    // Check for exclamation marks (should be rare per style guide)
    const exclamationCount = (text.match(/!/g) || []).length;
    if (exclamationCount > 2) {
      this.addDiagnostic({
        severity: 'hint',
        code: 'EXCESSIVE_EXCLAMATION',
        message: `Text contains ${exclamationCount} exclamation marks`,
        file,
        line,
        suggestion: 'Per style guide, exclamation points should be rare',
      });
    }
  }

  private checkUnreachablePassages(
    passageNames: Set<string>,
    passageRefs: Set<string>,
    scene: SceneFile,
    file: string
  ): void {
    // First passage is always reachable (it's the entry point)
    const firstPassage = scene.passages[0]?.name;
    const reachable = new Set<string>();
    if (firstPassage) {
      reachable.add(firstPassage);
    }

    // Add all referenced passages
    for (const ref of passageRefs) {
      reachable.add(ref);
    }

    // Check for unreachable passages
    for (const passage of scene.passages) {
      if (!reachable.has(passage.name) && passage.name !== firstPassage) {
        this.addDiagnostic({
          severity: 'warning',
          code: 'UNREACHABLE_PASSAGE',
          message: `Passage "${passage.name}" is never referenced and cannot be reached`,
          file,
          line: passage.span.start.line,
        });
      }
    }
  }

  private checkDuplicatePassages(scene: SceneFile, file: string): void {
    const seen = new Map<string, number>();

    for (const passage of scene.passages) {
      if (seen.has(passage.name)) {
        this.addDiagnostic({
          severity: 'error',
          code: 'DUPLICATE_PASSAGE',
          message: `Duplicate passage name: "${passage.name}" (first defined at line ${seen.get(passage.name)})`,
          file,
          line: passage.span.start.line,
        });
      } else {
        seen.set(passage.name, passage.span.start.line);
      }
    }
  }

  private addDiagnostic(diagnostic: AnalysisDiagnostic): void {
    this.diagnostics.push(diagnostic);
  }

  private suggestSimilar(input: string, candidates: Set<string>): string | undefined {
    let bestMatch: string | undefined;
    let bestScore = 0;

    for (const candidate of candidates) {
      const score = this.similarity(input.toLowerCase(), candidate.toLowerCase());
      if (score > bestScore && score > 0.5) {
        bestScore = score;
        bestMatch = candidate;
      }
    }

    return bestMatch ? `Did you mean "${bestMatch}"?` : undefined;
  }

  private similarity(a: string, b: string): number {
    // Simple Jaccard similarity on character bigrams
    const bigramsA = new Set<string>();
    const bigramsB = new Set<string>();

    for (let i = 0; i < a.length - 1; i++) {
      bigramsA.add(a.slice(i, i + 2));
    }
    for (let i = 0; i < b.length - 1; i++) {
      bigramsB.add(b.slice(i, i + 2));
    }

    let intersection = 0;
    for (const bg of bigramsA) {
      if (bigramsB.has(bg)) intersection++;
    }

    const union = bigramsA.size + bigramsB.size - intersection;
    return union === 0 ? 0 : intersection / union;
  }
}

// =============================================================================
// EXPORTS
// =============================================================================

export function createAnalyzer(context?: AnalysisContext): SceneAnalyzer {
  return new SceneAnalyzer(context ?? createBlackwingContext());
}

export function formatDiagnostics(diagnostics: AnalysisDiagnostic[]): string {
  const lines: string[] = [];

  const errors = diagnostics.filter(d => d.severity === 'error');
  const warnings = diagnostics.filter(d => d.severity === 'warning');
  const infos = diagnostics.filter(d => d.severity === 'info');
  const hints = diagnostics.filter(d => d.severity === 'hint');

  for (const diag of diagnostics) {
    const prefix = {
      error: '❌',
      warning: '⚠️ ',
      info: 'ℹ️ ',
      hint: '💡',
    }[diag.severity];

    let line = `${prefix} [${diag.code}]`;
    if (diag.file) {
      line += ` ${diag.file}`;
      if (diag.line) line += `:${diag.line}`;
    }
    line += `: ${diag.message}`;
    lines.push(line);

    if (diag.suggestion) {
      lines.push(`   → ${diag.suggestion}`);
    }
  }

  lines.push('');
  lines.push(`Summary: ${errors.length} error(s), ${warnings.length} warning(s), ${infos.length} info(s), ${hints.length} hint(s)`);

  return lines.join('\n');
}

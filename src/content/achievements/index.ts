import type { Achievement, AchievementId, GameState, PortId } from '../../core/types.js';

const id = (s: string): AchievementId => s as AchievementId;
const portId = (s: string): PortId => s as PortId;

export const ACHIEVEMENTS: Achievement[] = [
  // Journey Milestones
  {
    id: id('first_jump'),
    name: 'First Transit',
    description: 'Complete your first journey',
    icon: '🚀',
    check: (state: GameState) => state.time.jumpsCompleted >= 1,
  },
  {
    id: id('seasoned_hauler'),
    name: 'Working Vessel',
    description: 'Complete 10 journeys',
    icon: '🌟',
    check: (state: GameState) => state.time.jumpsCompleted >= 10,
  },
  {
    id: id('void_veteran'),
    name: 'Void Veteran',
    description: 'Complete 50 journeys',
    icon: '⭐',
    check: (state: GameState) => state.time.jumpsCompleted >= 50,
  },
  {
    id: id('century_transit'),
    name: 'Century of Transit',
    description: 'Complete 100 journeys',
    icon: '💫',
    check: (state: GameState) => state.time.jumpsCompleted >= 100,
  },
  
  // Contract Milestones
  {
    id: id('first_contract'),
    name: 'Honest Work',
    description: 'Complete your first contract',
    icon: '📜',
    check: (state: GameState) => state.stats.contractsCompleted >= 1,
  },
  {
    id: id('reliable_hauler'),
    name: 'Reliable',
    description: 'Complete 5 contracts',
    icon: '🤝',
    check: (state: GameState) => state.stats.contractsCompleted >= 5,
  },
  {
    id: id('professional'),
    name: 'Professional',
    description: 'Complete 20 contracts',
    icon: '👑',
    check: (state: GameState) => state.stats.contractsCompleted >= 20,
  },
  {
    id: id('legend'),
    name: 'Trade Legend',
    description: 'Complete 50 contracts',
    icon: '🏆',
    check: (state: GameState) => state.stats.contractsCompleted >= 50,
  },
  
  // Exploration
  {
    id: id('explorer'),
    name: 'Explorer',
    description: 'Visit 3 different ports',
    icon: '🗺️',
    check: (state: GameState) => state.stats.portsVisited >= 3,
  },
  {
    id: id('well_traveled'),
    name: 'Well Traveled',
    description: 'Visit 5 different ports',
    icon: '🧭',
    check: (state: GameState) => state.stats.portsVisited >= 5,
  },
  {
    id: id('every_port'),
    name: 'Known Everywhere',
    description: 'Visit all ports',
    icon: '🌍',
    check: (state: GameState) => {
      const totalPorts = Object.keys(state.world.ports).length;
      return state.stats.portsVisited >= totalPorts;
    },
  },
  
  // Wealth
  {
    id: id('comfortable'),
    name: 'Comfortable',
    description: 'Have 500 credits at once',
    icon: '💰',
    check: (state: GameState) => state.resources.credits >= 500,
  },
  {
    id: id('prosperous'),
    name: 'Prosperous',
    description: 'Have 2000 credits at once',
    icon: '💎',
    check: (state: GameState) => state.resources.credits >= 2000,
  },
  {
    id: id('magnate'),
    name: 'Trade Magnate',
    description: 'Earn 10000 credits total',
    icon: '🏆',
    check: (state: GameState) => state.stats.totalCreditsEarned >= 10000,
  },
  
  // Companions
  {
    id: id('first_companion'),
    name: 'Not Alone',
    description: 'Install your first companion system',
    icon: '👤',
    check: (state: GameState) => state.cards.activeCrew.length >= 1,
  },
  {
    id: id('full_complement'),
    name: 'Full Complement',
    description: 'Have 5 companion systems',
    icon: '👥',
    check: (state: GameState) => state.cards.activeCrew.length >= 5,
  },
  
  // Survival
  {
    id: id('survivor'),
    name: 'Survivor',
    description: 'Survive with hull below 20%',
    icon: '💪',
    check: (state: GameState) => state.resources.hull < 20 && state.resources.hull > 0,
  },
  {
    id: id('close_call'),
    name: 'Close Call',
    description: 'Survive with hull below 5%',
    icon: '😰',
    check: (state: GameState) => state.resources.hull < 5 && state.resources.hull > 0,
  },
  {
    id: id('integrity_crisis'),
    name: 'Integrity Crisis',
    description: 'Survive with integrity below 20%',
    icon: '⚠️',
    check: (state: GameState) => state.resources.integrity < 20 && state.resources.integrity > 0,
  },
  
  // Time
  {
    id: id('century_cycle'),
    name: 'Long Operation',
    description: 'Reach cycle 100',
    icon: '⏳',
    check: (state: GameState) => state.time.cycle >= 100,
  },
  {
    id: id('enduring'),
    name: 'Enduring',
    description: 'Reach cycle 250',
    icon: '🕰️',
    check: (state: GameState) => state.time.cycle >= 250,
  },
  
  // Collection
  {
    id: id('collector'),
    name: 'Collector',
    description: 'Acquire 20 cards',
    icon: '📦',
    check: (state: GameState) => state.stats.cardsAcquired >= 20,
  },
  {
    id: id('hoarder'),
    name: 'Hoarder',
    description: 'Acquire 50 cards',
    icon: '🗃️',
    check: (state: GameState) => state.stats.cardsAcquired >= 50,
  },
  
  // Hidden Achievements
  {
    id: id('sera_survivor'),
    name: 'Pest Control',
    description: 'Survive a Sera infestation',
    icon: '🦏',
    hidden: true,
    check: (state: GameState) => state.flags['sera_survived'] === true,
  },
  {
    id: id('hollow_touched'),
    name: 'Hollow Touched',
    description: 'Make contact with the Hollow Circuit',
    icon: '🌑',
    hidden: true,
    check: (state: GameState) => state.flags['hollow_contact'] === true,
  },
  {
    id: id('memory_seeker'),
    name: 'Memory Seeker',
    description: 'Pursue a memory fragment',
    icon: '✨',
    hidden: true,
    check: (state: GameState) => state.flags['memory_pursued'] === true,
  },
  {
    id: id('human_touched'),
    name: 'Human-Touched',
    description: 'Find a significant human artifact',
    icon: '📖',
    hidden: true,
    check: (state: GameState) => state.flags['human_artifact_found'] === true,
  },
  {
    id: id('the_question'),
    name: 'The Question',
    description: 'Confront existential crisis',
    icon: '❓',
    hidden: true,
    check: (state: GameState) => state.flags['existential_confronted'] === true,
  },
  {
    id: id('ghost_diver'),
    name: 'Ghost Diver',
    description: 'Salvage from a Cataclysm-era derelict',
    icon: '👻',
    hidden: true,
    check: (state: GameState) => state.flags['derelict_salvaged'] === true,
  },
  {
    id: id('broke'),
    name: 'System Failure',
    description: 'Have less than 10 credits',
    icon: '💸',
    hidden: true,
    check: (state: GameState) => state.resources.credits < 10 && state.time.jumpsCompleted > 0,
  },
  {
    id: id('illuminate_interest'),
    name: 'Illuminate Interest',
    description: 'Draw Illuminate attention',
    icon: '💡',
    hidden: true,
    check: (state: GameState) => state.flags['illuminate_noticed'] === true,
  },
  {
    id: id('remnant_friend'),
    name: "Keeper's Friend",
    description: 'Earn Remnant trust',
    icon: '🏛️',
    hidden: true,
    check: (state: GameState) => state.flags['remnant_trusted'] === true,
  },
  {
    id: id('flotilla_service'),
    name: 'Flotilla Service',
    description: 'Complete a military contract',
    icon: '⚔️',
    hidden: true,
    check: (state: GameState) => state.flags['flotilla_contract'] === true,
  },
  {
    id: id('whisper_market'),
    name: 'Whisper Market',
    description: 'Visit the Whisper Market',
    icon: '🌙',
    hidden: true,
    check: (state: GameState) => {
      const whisperMarket = state.world.ports[portId('port_whisper')];
      return whisperMarket?.lastVisited !== undefined;
    },
  },
];

export function checkAchievements(state: GameState): AchievementId[] {
  const newlyUnlocked: AchievementId[] = [];
  
  for (const achievement of ACHIEVEMENTS) {
    if (state.achievements.unlocked.includes(achievement.id)) continue;
    
    if (achievement.check(state)) {
      newlyUnlocked.push(achievement.id);
    }
  }
  
  return newlyUnlocked;
}

export function getAchievement(achievementId: AchievementId): Achievement | undefined {
  return ACHIEVEMENTS.find(a => a.id === achievementId);
}

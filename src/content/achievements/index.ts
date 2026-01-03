import type { Achievement, AchievementId, GameState } from '../../core/types.js';

const id = (s: string): AchievementId => s as AchievementId;

export const ACHIEVEMENTS: Achievement[] = [
  {
    id: id('first_jump'),
    name: 'First Steps',
    description: 'Complete your first jump',
    icon: '🚀',
    check: (state: GameState) => state.time.jumpsCompleted >= 1,
  },
  {
    id: id('seasoned_hauler'),
    name: 'Seasoned Hauler',
    description: 'Complete 10 jumps',
    icon: '🌟',
    check: (state: GameState) => state.time.jumpsCompleted >= 10,
  },
  {
    id: id('void_veteran'),
    name: 'Void Veteran',
    description: 'Complete 50 jumps',
    icon: '⭐',
    check: (state: GameState) => state.time.jumpsCompleted >= 50,
  },
  {
    id: id('first_contract'),
    name: 'Honest Work',
    description: 'Complete your first contract',
    icon: '📜',
    check: (state: GameState) => state.stats.contractsCompleted >= 1,
  },
  {
    id: id('reliable_captain'),
    name: 'Reliable Captain',
    description: 'Complete 5 contracts',
    icon: '🤝',
    check: (state: GameState) => state.stats.contractsCompleted >= 5,
  },
  {
    id: id('merchant_prince'),
    name: 'Merchant Prince',
    description: 'Complete 20 contracts',
    icon: '👑',
    check: (state: GameState) => state.stats.contractsCompleted >= 20,
  },
  {
    id: id('explorer'),
    name: 'Explorer',
    description: 'Visit 3 different ports',
    icon: '🗺️',
    check: (state: GameState) => state.stats.portsVisited >= 3,
  },
  {
    id: id('cartographer'),
    name: 'Cartographer',
    description: 'Visit all ports',
    icon: '🌍',
    check: (state: GameState) => {
      const totalPorts = Object.keys(state.world.ports).length;
      return state.stats.portsVisited >= totalPorts;
    },
  },
  {
    id: id('wealthy'),
    name: 'Comfortable',
    description: 'Have 500 credits at once',
    icon: '💰',
    check: (state: GameState) => state.resources.credits >= 500,
  },
  {
    id: id('rich'),
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
  {
    id: id('first_crew'),
    name: 'Captain',
    description: 'Hire your first crew member',
    icon: '👤',
    check: (state: GameState) => state.cards.activeCrew.length >= 1,
  },
  {
    id: id('full_crew'),
    name: 'Full House',
    description: 'Have 5 crew members',
    icon: '👥',
    check: (state: GameState) => state.cards.activeCrew.length >= 5,
  },
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
    id: id('cycle_100'),
    name: 'Century',
    description: 'Reach cycle 100',
    icon: '⏳',
    check: (state: GameState) => state.time.cycle >= 100,
  },
  {
    id: id('collector'),
    name: 'Collector',
    description: 'Acquire 20 cards',
    icon: '📦',
    check: (state: GameState) => state.stats.cardsAcquired >= 20,
  },
  {
    id: id('shadow_trader'),
    name: 'Shadow Trader',
    description: 'Visit the Shadow Market',
    icon: '🌑',
    hidden: true,
    check: (state: GameState) => {
      const shadowMarket = state.world.ports['port_shadow_market' as any];
      return shadowMarket?.lastVisited !== undefined;
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

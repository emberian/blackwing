import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import {
  saveGameState,
  loadGameState,
  clearGameState,
  saveMetaState,
  loadMetaState,
  updateMetaAchievements,
  updateMetaLifetime,
  incrementMetaGamesStarted,
  exportSaveData,
  importSaveData,
  clearAllData,
  type MetaState,
} from '../src/core/persistence.js';
import { createInitialState } from '../src/core/init.js';
import type { GameState, AchievementId } from '../src/core/types.js';

const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: vi.fn((key: string) => store[key] ?? null),
    setItem: vi.fn((key: string, value: string) => { store[key] = value; }),
    removeItem: vi.fn((key: string) => { delete store[key]; }),
    clear: vi.fn(() => { store = {}; }),
  };
})();

Object.defineProperty(globalThis, 'localStorage', { value: localStorageMock });

describe('Persistence Module', () => {
  let state: GameState;
  
  beforeEach(() => {
    localStorageMock.clear();
    vi.clearAllMocks();
    state = createInitialState();
  });

  afterEach(() => {
    localStorageMock.clear();
  });

  describe('Game State Persistence', () => {
    it('saves game state to localStorage', () => {
      const result = saveGameState(state);
      
      expect(result).toBe(true);
      expect(localStorageMock.setItem).toHaveBeenCalledWith(
        'blackwing_save',
        JSON.stringify(state)
      );
    });

    it('loads game state from localStorage', () => {
      localStorageMock.setItem('blackwing_save', JSON.stringify(state));
      
      const loaded = loadGameState();
      
      expect(loaded).not.toBeNull();
      expect(loaded?.schemaVersion).toBe(state.schemaVersion);
      expect(loaded?.resources).toEqual(state.resources);
    });

    it('returns null when no save exists', () => {
      const loaded = loadGameState();
      expect(loaded).toBeNull();
    });

    it('returns null for schema version mismatch', () => {
      const oldState = { ...state, schemaVersion: 1 };
      localStorageMock.setItem('blackwing_save', JSON.stringify(oldState));
      
      const loaded = loadGameState();
      expect(loaded).toBeNull();
    });

    it('returns null for corrupted save data', () => {
      localStorageMock.setItem('blackwing_save', 'not valid json{{{');
      
      const loaded = loadGameState();
      expect(loaded).toBeNull();
    });

    it('clears game state from localStorage', () => {
      saveGameState(state);
      clearGameState();
      
      expect(localStorageMock.removeItem).toHaveBeenCalledWith('blackwing_save');
    });
  });

  describe('Meta State Persistence', () => {
    it('saves meta state to localStorage', () => {
      const meta: MetaState = {
        schemaVersion: 1,
        achievements: { unlocked: [], unlockedAt: {} },
        lifetime: {
          gamesStarted: 1,
          gamesCompleted: 0,
          totalJumps: 0,
          totalCreditsEarned: 0,
          totalPortsDiscovered: 0,
        },
      };
      
      const result = saveMetaState(meta);
      
      expect(result).toBe(true);
      expect(localStorageMock.setItem).toHaveBeenCalledWith(
        'blackwing_meta',
        JSON.stringify(meta)
      );
    });

    it('loads meta state from localStorage', () => {
      const meta: MetaState = {
        schemaVersion: 1,
        achievements: { unlocked: [], unlockedAt: {} },
        lifetime: {
          gamesStarted: 5,
          gamesCompleted: 2,
          totalJumps: 100,
          totalCreditsEarned: 5000,
          totalPortsDiscovered: 7,
        },
      };
      localStorageMock.setItem('blackwing_meta', JSON.stringify(meta));
      
      const loaded = loadMetaState();
      
      expect(loaded.lifetime.gamesStarted).toBe(5);
      expect(loaded.lifetime.totalJumps).toBe(100);
    });

    it('returns initial meta state when none exists', () => {
      const loaded = loadMetaState();
      
      expect(loaded.schemaVersion).toBe(1);
      expect(loaded.lifetime.gamesStarted).toBe(1);
      expect(loaded.achievements.unlocked).toEqual([]);
    });

    it('returns initial meta state for corrupted data', () => {
      localStorageMock.setItem('blackwing_meta', 'invalid json');
      
      const loaded = loadMetaState();
      
      expect(loaded.schemaVersion).toBe(1);
      expect(loaded.lifetime.gamesStarted).toBe(1);
    });
  });

  describe('Meta State Updates', () => {
    let baseMeta: MetaState;
    
    beforeEach(() => {
      baseMeta = {
        schemaVersion: 1,
        achievements: { unlocked: [], unlockedAt: {} },
        lifetime: {
          gamesStarted: 1,
          gamesCompleted: 0,
          totalJumps: 0,
          totalCreditsEarned: 0,
          totalPortsDiscovered: 0,
        },
      };
    });

    it('updates meta achievements with new unlocks', () => {
      const newAchievements = ['first_jump', 'trader'] as AchievementId[];
      
      const updated = updateMetaAchievements(baseMeta, newAchievements, 5);
      
      expect(updated.achievements.unlocked).toContain('first_jump');
      expect(updated.achievements.unlocked).toContain('trader');
      expect(updated.achievements.unlockedAt['first_jump' as AchievementId]).toBe(5);
    });

    it('does not duplicate existing achievements', () => {
      baseMeta.achievements.unlocked = ['first_jump' as AchievementId];
      baseMeta.achievements.unlockedAt = { ['first_jump' as AchievementId]: 1 };
      
      const updated = updateMetaAchievements(baseMeta, ['first_jump' as AchievementId], 10);
      
      expect(updated.achievements.unlocked.filter(id => id === 'first_jump').length).toBe(1);
      expect(updated.achievements.unlockedAt['first_jump' as AchievementId]).toBe(1);
    });

    it('returns unchanged meta when no new achievements', () => {
      const updated = updateMetaAchievements(baseMeta, [], 5);
      expect(updated).toBe(baseMeta);
    });

    it('updates lifetime statistics', () => {
      const updated = updateMetaLifetime(baseMeta, {
        totalJumps: 50,
        totalCreditsEarned: 1000,
      });
      
      expect(updated.lifetime.totalJumps).toBe(50);
      expect(updated.lifetime.totalCreditsEarned).toBe(1000);
      expect(updated.lifetime.gamesStarted).toBe(1);
    });

    it('increments games started counter', () => {
      const updated = incrementMetaGamesStarted(baseMeta);
      expect(updated.lifetime.gamesStarted).toBe(2);
    });
  });

  describe('Export/Import Save Data', () => {
    let meta: MetaState;
    
    beforeEach(() => {
      meta = {
        schemaVersion: 1,
        achievements: { unlocked: [], unlockedAt: {} },
        lifetime: {
          gamesStarted: 1,
          gamesCompleted: 0,
          totalJumps: 0,
          totalCreditsEarned: 0,
          totalPortsDiscovered: 0,
        },
      };
    });

    it('exports save data as formatted JSON', () => {
      const exported = exportSaveData(state, meta);
      const parsed = JSON.parse(exported);
      
      expect(parsed.version).toBe(2);
      expect(parsed.timestamp).toBeDefined();
      expect(parsed.gameState).toBeDefined();
      expect(parsed.metaState).toBeDefined();
    });

    it('imports valid save data', () => {
      const exported = exportSaveData(state, meta);
      const imported = importSaveData(exported);
      
      expect('error' in imported).toBe(false);
      if (!('error' in imported)) {
        expect(imported.gameState.schemaVersion).toBe(2);
        expect(imported.metaState.schemaVersion).toBe(1);
      }
    });

    it('rejects invalid JSON', () => {
      const result = importSaveData('not valid json{');
      
      expect('error' in result).toBe(true);
      if ('error' in result) {
        expect(result.error).toContain('parse');
      }
    });

    it('rejects missing required fields', () => {
      const result = importSaveData(JSON.stringify({ version: 2 }));
      
      expect('error' in result).toBe(true);
      if ('error' in result) {
        expect(result.error).toContain('Invalid save file format');
      }
    });

    it('rejects incompatible schema version', () => {
      const oldSave = {
        version: 2,
        timestamp: Date.now(),
        gameState: { ...state, schemaVersion: 1 },
        metaState: meta,
      };
      
      const result = importSaveData(JSON.stringify(oldSave));
      
      expect('error' in result).toBe(true);
      if ('error' in result) {
        expect(result.error).toContain('Incompatible');
      }
    });
  });

  describe('Clear All Data', () => {
    it('clears both save and meta from localStorage', () => {
      saveGameState(state);
      saveMetaState({
        schemaVersion: 1,
        achievements: { unlocked: [], unlockedAt: {} },
        lifetime: {
          gamesStarted: 5,
          gamesCompleted: 2,
          totalJumps: 100,
          totalCreditsEarned: 5000,
          totalPortsDiscovered: 7,
        },
      });
      
      clearAllData();
      
      expect(localStorageMock.removeItem).toHaveBeenCalledWith('blackwing_save');
      expect(localStorageMock.removeItem).toHaveBeenCalledWith('blackwing_meta');
    });
  });
});

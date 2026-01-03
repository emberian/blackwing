import type { GameState, AchievementId, AchievementState } from './types.js';

const SAVE_KEY = 'cargo_hold_save';
const META_KEY = 'cargo_hold_meta';
const SCHEMA_VERSION = 2;

export interface MetaState {
  schemaVersion: number;
  achievements: AchievementState;
  lifetime: {
    gamesStarted: number;
    gamesCompleted: number;
    totalJumps: number;
    totalCreditsEarned: number;
    totalPortsDiscovered: number;
  };
}

function createInitialMetaState(): MetaState {
  return {
    schemaVersion: 1,
    achievements: {
      unlocked: [],
      unlockedAt: {},
    },
    lifetime: {
      gamesStarted: 1,
      gamesCompleted: 0,
      totalJumps: 0,
      totalCreditsEarned: 0,
      totalPortsDiscovered: 0,
    },
  };
}

export interface SaveData {
  version: number;
  timestamp: number;
  gameState: GameState;
  metaState: MetaState;
}

export function saveGameState(state: GameState): boolean {
  try {
    const serialized = JSON.stringify(state);
    localStorage.setItem(SAVE_KEY, serialized);
    return true;
  } catch (e) {
    console.error('Failed to save game:', e);
    return false;
  }
}

export function loadGameState(): GameState | null {
  try {
    const serialized = localStorage.getItem(SAVE_KEY);
    if (!serialized) return null;

    const loaded = JSON.parse(serialized) as GameState;
    
    if (loaded.schemaVersion !== SCHEMA_VERSION) {
      console.warn('Save version mismatch, starting fresh');
      return null;
    }

    return loaded;
  } catch (e) {
    console.error('Failed to load game:', e);
    return null;
  }
}

export function clearGameState(): void {
  localStorage.removeItem(SAVE_KEY);
}

export function saveMetaState(meta: MetaState): boolean {
  try {
    const serialized = JSON.stringify(meta);
    localStorage.setItem(META_KEY, serialized);
    return true;
  } catch (e) {
    console.error('Failed to save meta state:', e);
    return false;
  }
}

export function loadMetaState(): MetaState {
  try {
    const serialized = localStorage.getItem(META_KEY);
    if (!serialized) return createInitialMetaState();

    const loaded = JSON.parse(serialized) as MetaState;
    return loaded;
  } catch (e) {
    console.error('Failed to load meta state:', e);
    return createInitialMetaState();
  }
}

export function updateMetaAchievements(
  meta: MetaState,
  newAchievements: AchievementId[],
  cycle: number
): MetaState {
  if (newAchievements.length === 0) return meta;

  const unlockedAt = { ...meta.achievements.unlockedAt };
  const unlocked = [...meta.achievements.unlocked];

  for (const id of newAchievements) {
    if (!unlocked.includes(id)) {
      unlocked.push(id);
      unlockedAt[id] = cycle;
    }
  }

  return {
    ...meta,
    achievements: { unlocked, unlockedAt },
  };
}

export function updateMetaLifetime(
  meta: MetaState,
  updates: Partial<MetaState['lifetime']>
): MetaState {
  return {
    ...meta,
    lifetime: {
      ...meta.lifetime,
      ...updates,
    },
  };
}

export function incrementMetaGamesStarted(meta: MetaState): MetaState {
  return updateMetaLifetime(meta, {
    gamesStarted: meta.lifetime.gamesStarted + 1,
  });
}

export function exportSaveData(gameState: GameState, metaState: MetaState): string {
  const saveData: SaveData = {
    version: SCHEMA_VERSION,
    timestamp: Date.now(),
    gameState,
    metaState,
  };
  return JSON.stringify(saveData, null, 2);
}

export function importSaveData(jsonString: string): SaveData | { error: string } {
  try {
    const data = JSON.parse(jsonString) as SaveData;
    
    if (!data.version || !data.gameState || !data.metaState) {
      return { error: 'Invalid save file format' };
    }
    
    if (data.gameState.schemaVersion !== SCHEMA_VERSION) {
      return { error: `Incompatible save version (expected ${SCHEMA_VERSION}, got ${data.gameState.schemaVersion})` };
    }
    
    return data;
  } catch (e) {
    return { error: 'Failed to parse save file' };
  }
}

export function downloadSaveFile(gameState: GameState, metaState: MetaState): void {
  const json = exportSaveData(gameState, metaState);
  const blob = new Blob([json], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  
  const a = document.createElement('a');
  a.href = url;
  a.download = `cargo-hold-save-${new Date().toISOString().split('T')[0]}.json`;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

export function createFileInput(onImport: (data: SaveData) => void, onError: (msg: string) => void): void {
  const input = document.createElement('input');
  input.type = 'file';
  input.accept = '.json';
  
  input.onchange = async (e) => {
    const file = (e.target as HTMLInputElement).files?.[0];
    if (!file) return;
    
    try {
      const text = await file.text();
      const result = importSaveData(text);
      
      if ('error' in result) {
        onError(result.error);
      } else {
        onImport(result);
      }
    } catch {
      onError('Failed to read file');
    }
  };
  
  input.click();
}

export function clearAllData(): void {
  localStorage.removeItem(SAVE_KEY);
  localStorage.removeItem(META_KEY);
}

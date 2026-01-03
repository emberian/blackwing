import type { GameState, CardDef, PortState, CardInstance, AchievementId } from '../core/types.js';
import type { GameController } from '../core/controller.js';
import type { TriggeredEvent } from '../core/events.js';
import { ACHIEVEMENTS, getAchievement } from '../content/achievements/index.js';

export type ViewMode = 'narrative' | 'hold' | 'crew' | 'market' | 'travel' | 'chronicle' | 'achievements';

export interface UIState {
  viewMode: ViewMode;
  selectedCardId: string | null;
  achievementToast: AchievementId | null;
  actionToast: string | null;
}

export function createRenderer(
  container: HTMLElement,
  controller: GameController,
  cardDefs: Map<string, CardDef>
) {
  let uiState: UIState = {
    viewMode: 'narrative',
    selectedCardId: null,
    achievementToast: null,
    actionToast: null,
  };

  function setView(mode: ViewMode) {
    uiState = { ...uiState, viewMode: mode };
    render();
  }

  function showAchievementToast(achievementId: AchievementId) {
    uiState = { ...uiState, achievementToast: achievementId };
    render();
    setTimeout(() => {
      uiState = { ...uiState, achievementToast: null };
      render();
    }, 3000);
  }
  
  function showActionToast(message: string) {
    uiState = { ...uiState, actionToast: message };
    render();
    setTimeout(() => {
      uiState = { ...uiState, actionToast: null };
      render();
    }, 2000);
  }

  function render() {
    const state = controller.getState();
    const event = controller.getCurrentEvent();
    const journeyState = controller.getJourneyState();
    const isGameOver = controller.isGameOver();
    
    const newAchievements = controller.getNewAchievements();
    if (newAchievements.length > 0) {
      controller.clearNewAchievements();
      for (const id of newAchievements) {
        showAchievementToast(id);
      }
    }

    if (isGameOver) {
      container.innerHTML = renderGameOver(state);
      attachEventListeners();
      return;
    }

    container.innerHTML = `
      <div class="game-container">
        ${renderStatusBar(state, journeyState)}
        ${uiState.achievementToast ? renderAchievementToast(uiState.achievementToast) : ''}
        ${uiState.actionToast ? renderActionToast(uiState.actionToast) : ''}
        <main class="main-content">
          ${event ? renderEvent(event) : renderMainView(state, journeyState)}
        </main>
        ${!event && !journeyState ? renderNavBar() : ''}
      </div>
    `;

    attachEventListeners();
  }
  
  function renderAchievementToast(achievementId: AchievementId): string {
    const achievement = getAchievement(achievementId);
    if (!achievement) return '';
    
    return `
      <div class="achievement-toast">
        <span class="achievement-icon">${achievement.icon}</span>
        <div class="achievement-info">
          <div class="achievement-label">Achievement Unlocked!</div>
          <div class="achievement-name">${achievement.name}</div>
        </div>
      </div>
    `;
  }
  
  function renderActionToast(message: string): string {
    return `
      <div class="action-toast">
        ${message}
      </div>
    `;
  }

  function renderGameOver(state: GameState): string {
    const reason = state.resources.hull <= 0 
      ? 'Your ship has been destroyed.' 
      : 'Your crew has lost all hope.';
    
    return `
      <div class="game-container">
        <div class="view-gameover">
          <h1>Game Over</h1>
          <p class="gameover-reason">${reason}</p>
          <div class="gameover-stats">
            <p>Jumps completed: ${state.time.jumpsCompleted}</p>
            <p>Credits earned: ${state.stats.totalCreditsEarned}</p>
            <p>Contracts completed: ${state.stats.contractsCompleted}</p>
            <p>Ports visited: ${state.stats.portsVisited}</p>
          </div>
          <button class="btn btn-primary" data-action="restart">Start New Game</button>
        </div>
      </div>
    `;
  }

  function renderStatusBar(state: GameState, journeyState: ReturnType<typeof controller.getJourneyState>): string {
    const port = state.world.ports[state.world.currentLocation];
    const locationText = journeyState 
      ? `Jumping to ${state.world.ports[journeyState.destination]?.name ?? 'Unknown'}`
      : port?.name ?? 'Unknown';

    return `
      <header class="status-bar">
        <div class="status-row">
          <div class="status-resources">
            <span class="resource" title="Credits">&#x26A1; ${Math.floor(state.resources.credits)}</span>
            <span class="resource" title="Fuel">&#x26FD; ${Math.floor(state.resources.fuel)}</span>
            <span class="resource" title="Supplies">&#x1F4E6; ${Math.floor(state.resources.supplies)}</span>
            <span class="resource" title="Hull">&#x1F6E1; ${Math.floor(state.resources.hull)}%</span>
          </div>
          <button class="settings-btn" data-action="settings" title="Settings">&#x2699;</button>
        </div>
        <div class="status-location">
          <span class="location-name">${locationText}</span>
          <span class="cycle-count">Cycle ${state.time.cycle}</span>
        </div>
      </header>
    `;
  }

  function renderNavBar(): string {
    const tabs: { mode: ViewMode; label: string; icon: string }[] = [
      { mode: 'narrative', label: 'Log', icon: '&#x1F4DC;' },
      { mode: 'hold', label: 'Hold', icon: '&#x1F4E6;' },
      { mode: 'crew', label: 'Crew', icon: '&#x1F465;' },
      { mode: 'market', label: 'Trade', icon: '&#x1F4B0;' },
      { mode: 'travel', label: 'Jump', icon: '&#x1F680;' },
      { mode: 'achievements', label: 'Goals', icon: '&#x1F3C6;' },
    ];

    return `
      <nav class="nav-bar">
        ${tabs.map(tab => `
          <button 
            class="nav-btn ${uiState.viewMode === tab.mode ? 'active' : ''}"
            data-view="${tab.mode}"
          >
            <span class="nav-icon">${tab.icon}</span>
            <span class="nav-label">${tab.label}</span>
          </button>
        `).join('')}
      </nav>
    `;
  }

  function renderMainView(state: GameState, journeyState: ReturnType<typeof controller.getJourneyState>): string {
    if (journeyState) {
      return renderJourneyProgress(state, journeyState);
    }

    switch (uiState.viewMode) {
      case 'narrative': return renderNarrative(state);
      case 'hold': return renderHold(state);
      case 'crew': return renderCrew(state);
      case 'market': return renderMarket(state);
      case 'travel': return renderTravel(state);
      case 'chronicle': return renderChronicle(state);
      case 'achievements': return renderAchievements(state);
      default: return renderNarrative(state);
    }
  }

  function renderJourneyProgress(state: GameState, journeyState: NonNullable<ReturnType<typeof controller.getJourneyState>>): string {
    const dest = state.world.ports[journeyState.destination];
    const progress = journeyState.totalEvents > 0 
      ? ((journeyState.totalEvents - journeyState.eventsRemaining) / journeyState.totalEvents) * 100
      : 100;

    return `
      <div class="view-journey">
        <h2>In Transit</h2>
        <p class="journey-dest">Destination: ${dest?.name ?? 'Unknown'}</p>
        <div class="journey-progress">
          <div class="progress-bar">
            <div class="progress-fill" style="width: ${progress}%"></div>
          </div>
          <p class="progress-text">${journeyState.eventsRemaining} events remaining</p>
        </div>
        <p class="journey-flavor">The void stretches. The hold hums.</p>
      </div>
    `;
  }

  function renderNarrative(state: GameState): string {
    const recentChronicle = state.chronicle.slice(-5).reverse();
    const isFirstTime = state.time.cycle === 0 && state.time.jumpsCompleted === 0;
    
    return `
      <div class="view-narrative">
        ${isFirstTime ? `
          <div class="tutorial-hint">
            <strong>Welcome, Captain.</strong> Your journey begins at Haven Prime.
            <br><br>
            <em>Trade</em> cargo between ports to earn credits. <em>Jump</em> to travel.
            Watch your fuel, supplies, and hull. Check <em>Goals</em> for achievements.
          </div>
        ` : ''}
        <div class="narrative-entries">
          ${recentChronicle.map(entry => `
            <article class="chronicle-entry">
              <header class="entry-header">
                <h3>${entry.title}</h3>
                <time>Cycle ${entry.timestamp.cycle}</time>
              </header>
              <p>${entry.text}</p>
            </article>
          `).join('')}
        </div>
        
        <div class="narrative-actions">
          <button class="btn btn-primary" data-action="trigger-event">What catches your attention?</button>
        </div>
      </div>
    `;
  }

  function renderHold(state: GameState): string {
    const cargoInstances = [...state.cards.collection, ...state.cards.deck]
      .map(id => state.cards.instances[id])
      .filter((inst): inst is CardInstance => !!inst)
      .filter(inst => {
        const def = cardDefs.get(inst.cardDefId);
        return def?.type === 'cargo' || def?.type === 'echo';
      });

    return `
      <div class="view-hold">
        <h2>Cargo Hold</h2>
        ${cargoInstances.length === 0 ? `
          <p class="empty-state">The hold is empty. Trade awaits.</p>
        ` : `
          <ul class="card-list">
            ${cargoInstances.map(inst => renderCardItem(inst, state)).join('')}
          </ul>
        `}
      </div>
    `;
  }

  function renderCrew(state: GameState): string {
    const crewInstances = state.cards.activeCrew
      .map(id => state.cards.instances[id])
      .filter((inst): inst is CardInstance => !!inst);

    return `
      <div class="view-crew">
        <h2>Crew Manifest</h2>
        <div class="morale-display">
          <span>Morale:</span>
          <div class="morale-bar">
            <div class="morale-fill" style="width: ${state.resources.morale}%"></div>
          </div>
          <span>${Math.floor(state.resources.morale)}%</span>
        </div>
        ${crewInstances.length === 0 ? `
          <p class="empty-state">No crew aboard. The hold runs on silence.</p>
        ` : `
          <ul class="card-list">
            ${crewInstances.map(inst => renderCrewItem(inst)).join('')}
          </ul>
        `}
      </div>
    `;
  }

  function renderMarket(state: GameState): string {
    const port = state.world.ports[state.world.currentLocation];
    if (!port) return '<div class="view-market"><p>Error: Unknown location</p></div>';

    return `
      <div class="view-market">
        <h2>${port.name} Market</h2>
        <p class="port-desc">${port.description}</p>
        
        <h3>Available Cargo</h3>
        <ul class="market-list">
          ${port.availableCards.map(defId => {
            const def = cardDefs.get(defId);
            if (!def || (def.type !== 'cargo' && def.type !== 'module')) return '';
            const price = Math.ceil((def.baseValue ?? 10) * (port.marketModifiers[defId] ?? 1));
            return `
              <li class="market-item">
                <div class="item-info">
                  <span class="item-name">${def.name}</span>
                  <span class="item-price">${price} cr</span>
                </div>
                <button class="btn btn-small" data-action="buy" data-card="${defId}" 
                  ${state.resources.credits < price ? 'disabled' : ''}>Buy</button>
              </li>
            `;
          }).join('')}
        </ul>

        <h3>Available Crew</h3>
        <ul class="market-list">
          ${port.availableCards.map(defId => {
            const def = cardDefs.get(defId);
            if (!def || def.type !== 'crew') return '';
            const price = def.baseValue ?? 50;
            return `
              <li class="market-item">
                <div class="item-info">
                  <span class="item-name">${def.name}</span>
                  <span class="item-price">${price} cr</span>
                </div>
                <button class="btn btn-small" data-action="hire" data-card="${defId}"
                  ${state.resources.credits < price ? 'disabled' : ''}>Hire</button>
              </li>
            `;
          }).join('')}
        </ul>

        <h3>Services</h3>
        <div class="services">
          <button class="btn" data-action="refuel" ${state.resources.credits < 30 ? 'disabled' : ''}>
            Refuel (+10) - 30 cr
          </button>
          <button class="btn" data-action="resupply" ${state.resources.credits < 20 ? 'disabled' : ''}>
            Resupply (+10) - 20 cr
          </button>
          <button class="btn" data-action="repair" ${state.resources.credits < 50 || state.resources.hull >= 100 ? 'disabled' : ''}>
            Repair (+10) - 50 cr
          </button>
        </div>
      </div>
    `;
  }

  function renderTravel(state: GameState): string {
    const knownPorts = state.world.knownPorts
      .filter(id => id !== state.world.currentLocation)
      .map(id => state.world.ports[id])
      .filter((p): p is PortState => !!p);

    return `
      <div class="view-travel">
        <h2>Jump Navigation</h2>
        <p>Current fuel: ${Math.floor(state.resources.fuel)}</p>
        
        <ul class="destination-list">
          ${knownPorts.map(port => `
            <li class="destination-item">
              <div class="dest-info">
                <span class="dest-name">${port.name}</span>
                <span class="dest-status status-${port.status}">${port.status}</span>
              </div>
              <p class="dest-desc">${port.description}</p>
              <button class="btn btn-primary" data-action="travel" data-dest="${port.id}"
                ${state.resources.fuel < 10 ? 'disabled' : ''}>
                Jump (10 fuel)
              </button>
            </li>
          `).join('')}
        </ul>
      </div>
    `;
  }

  function renderChronicle(state: GameState): string {
    return `
      <div class="view-chronicle">
        <h2>Captain's Chronicle</h2>
        <div class="chronicle-full">
          ${state.chronicle.map(entry => `
            <article class="chronicle-entry">
              <header>
                <h3>${entry.title}</h3>
                <time>Cycle ${entry.timestamp.cycle}</time>
              </header>
              <p>${entry.text}</p>
            </article>
          `).reverse().join('')}
        </div>
      </div>
    `;
  }

  function renderAchievements(state: GameState): string {
    const unlocked = state.achievements.unlocked;
    const unlockedCount = unlocked.length;
    const totalCount = ACHIEVEMENTS.filter(a => !a.hidden).length;
    
    return `
      <div class="view-achievements">
        <h2>Achievements</h2>
        <p class="achievement-progress">${unlockedCount} / ${totalCount} unlocked</p>
        <ul class="achievement-list">
          ${ACHIEVEMENTS.map(achievement => {
            const isUnlocked = unlocked.includes(achievement.id);
            const isHidden = achievement.hidden && !isUnlocked;
            
            if (isHidden) return '';
            
            return `
              <li class="achievement-item ${isUnlocked ? 'unlocked' : 'locked'}">
                <span class="achievement-icon">${isUnlocked ? achievement.icon : '?'}</span>
                <div class="achievement-details">
                  <span class="achievement-name">${achievement.name}</span>
                  <span class="achievement-desc">${achievement.description}</span>
                </div>
              </li>
            `;
          }).join('')}
        </ul>
      </div>
    `;
  }

  function renderEvent(event: TriggeredEvent): string {
    const passage = event.scenelet.passages[event.passageIndex];
    if (!passage) return '<p>Error: Invalid passage</p>';

    return `
      <div class="view-event">
        <h2>${event.scenelet.title}</h2>
        <div class="event-text">
          ${passage.text.split('\n\n').map(p => `<p>${p}</p>`).join('')}
        </div>
        ${passage.choices && passage.choices.length > 0 ? `
          <div class="event-choices">
            ${passage.choices.map((choice, i) => `
              <button class="btn btn-choice" data-action="event-choice" data-choice="${i}">
                ${choice.text}
              </button>
            `).join('')}
          </div>
        ` : `
          <button class="btn btn-primary" data-action="event-dismiss">Continue</button>
        `}
      </div>
    `;
  }

  function renderCardItem(inst: CardInstance, state: GameState): string {
    const def = cardDefs.get(inst.cardDefId);
    if (!def) return '';

    const isEquipped = state.cards.deck.includes(inst.instanceId);

    return `
      <li class="card-item ${isEquipped ? 'equipped' : ''}">
        <div class="card-info">
          <span class="card-name">${def.name}</span>
          <span class="card-condition">${inst.condition}%</span>
        </div>
        <p class="card-desc">${def.description}</p>
        <div class="card-actions">
          <button class="btn btn-small" data-action="sell" data-instance="${inst.instanceId}">Sell</button>
        </div>
      </li>
    `;
  }

  function renderCrewItem(inst: CardInstance): string {
    const def = cardDefs.get(inst.cardDefId);
    if (!def) return '';

    return `
      <li class="card-item crew-item">
        <div class="card-info">
          <span class="card-name">${def.name}</span>
        </div>
        <p class="card-desc">${def.description}</p>
        ${def.flavorText ? `<p class="card-flavor">${def.flavorText}</p>` : ''}
      </li>
    `;
  }

  function attachEventListeners() {
    container.querySelectorAll('[data-view]').forEach(btn => {
      btn.addEventListener('click', (e) => {
        const mode = (e.currentTarget as HTMLElement).dataset.view as ViewMode;
        setView(mode);
      });
    });

    container.querySelectorAll('[data-action]').forEach(btn => {
      btn.addEventListener('click', (e) => {
        const el = e.currentTarget as HTMLElement;
        const action = el.dataset.action;

        switch (action) {
          case 'trigger-event':
            controller.triggerPortEvent();
            render();
            break;
          case 'event-choice':
            const choiceIdx = parseInt(el.dataset.choice ?? '0', 10);
            controller.resolveEventChoice(choiceIdx);
            render();
            break;
          case 'event-dismiss':
            controller.resolveEventChoice(-1);
            render();
            break;
          case 'buy': {
            const result = controller.dispatch({
              type: 'TRADE_BUY',
              payload: { cardDefId: el.dataset.card as any, quantity: 1 }
            });
            if (result.message) showActionToast(result.message);
            render();
            break;
          }
          case 'hire': {
            const result = controller.dispatch({
              type: 'CREW_HIRE',
              payload: { cardDefId: el.dataset.card as any }
            });
            if (result.message) showActionToast(result.message);
            render();
            break;
          }
          case 'sell': {
            const result = controller.dispatch({
              type: 'TRADE_SELL',
              payload: { instanceId: el.dataset.instance as any }
            });
            if (result.message) showActionToast(result.message);
            render();
            break;
          }
          case 'travel': {
            const result = controller.travel(el.dataset.dest as any);
            if (result.message) showActionToast(result.message);
            render();
            break;
          }
          case 'refuel': {
            const result = controller.dispatch({ type: 'REFUEL', payload: { amount: 10 } });
            if (result.message) showActionToast(result.message);
            render();
            break;
          }
          case 'resupply': {
            const result = controller.dispatch({ type: 'RESUPPLY', payload: { amount: 10 } });
            if (result.message) showActionToast(result.message);
            render();
            break;
          }
          case 'repair':
            controller.dispatch({ type: 'REPAIR', payload: { amount: 10 } });
            render();
            break;
          case 'restart':
            controller.reset();
            render();
            break;
          case 'settings':
            if (confirm('Reset game? All progress will be lost.')) {
              controller.reset();
            }
            render();
            break;
        }
      });
    });
  }

  controller.subscribe(() => {
    render();
  });

  return {
    render,
    setView,
  };
}

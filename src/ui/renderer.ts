import type { GameState, CardDef, PortState, CardInstance } from '../core/types.js';
import type { GameController } from '../core/controller.js';
import type { TriggeredEvent } from '../core/events.js';

export type ViewMode = 'narrative' | 'hold' | 'crew' | 'market' | 'travel' | 'chronicle';

export interface UIState {
  viewMode: ViewMode;
  selectedCardId: string | null;
  showingEvent: boolean;
}

export function createRenderer(
  container: HTMLElement,
  controller: GameController,
  cardDefs: Map<string, CardDef>
) {
  let uiState: UIState = {
    viewMode: 'narrative',
    selectedCardId: null,
    showingEvent: false,
  };

  function setView(mode: ViewMode) {
    uiState = { ...uiState, viewMode: mode };
    render();
  }

  function render() {
    const state = controller.getState();
    const event = controller.getCurrentEvent();

    container.innerHTML = `
      <div class="game-container">
        ${renderStatusBar(state)}
        <main class="main-content">
          ${event ? renderEvent(event, state) : renderMainView(state)}
        </main>
        ${renderNavBar()}
      </div>
    `;

    attachEventListeners();
  }

  function renderStatusBar(state: GameState): string {
    const port = state.world.ports[state.world.currentLocation];
    const locationText = state.time.inTransit 
      ? `In transit to ${state.world.ports[state.time.transitDestination!]?.name ?? 'Unknown'}`
      : port?.name ?? 'Unknown';

    return `
      <header class="status-bar">
        <div class="status-resources">
          <span class="resource" title="Credits">&#x26A1; ${Math.floor(state.resources.credits)}</span>
          <span class="resource" title="Fuel">&#x26FD; ${Math.floor(state.resources.fuel)}</span>
          <span class="resource" title="Supplies">&#x1F4E6; ${Math.floor(state.resources.supplies)}</span>
          <span class="resource" title="Hull">&#x1F6E1; ${Math.floor(state.resources.hull)}%</span>
        </div>
        <div class="status-location">
          <span class="location-name">${locationText}</span>
          <span class="era-year">Era ${state.time.era}, Year ${state.time.year}</span>
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

  function renderMainView(state: GameState): string {
    switch (uiState.viewMode) {
      case 'narrative': return renderNarrative(state);
      case 'hold': return renderHold(state);
      case 'crew': return renderCrew(state);
      case 'market': return renderMarket(state);
      case 'travel': return renderTravel(state);
      case 'chronicle': return renderChronicle(state);
      default: return renderNarrative(state);
    }
  }

  function renderNarrative(state: GameState): string {
    const recentChronicle = state.chronicle.slice(-5).reverse();
    
    return `
      <div class="view-narrative">
        <div class="narrative-entries">
          ${recentChronicle.map(entry => `
            <article class="chronicle-entry">
              <header class="entry-header">
                <h3>${entry.title}</h3>
                <time>Era ${entry.timestamp.era}, Year ${entry.timestamp.year}</time>
              </header>
              <p>${entry.text}</p>
            </article>
          `).join('')}
        </div>
        
        <div class="narrative-actions">
          ${state.time.inTransit ? `
            <p class="transit-status">The void stretches. ${getTransitTimeRemaining(state)} years remain.</p>
            <button class="btn btn-primary" data-action="trigger-event">Stir from cryo-doze</button>
          ` : `
            <button class="btn btn-primary" data-action="trigger-event">What catches your attention?</button>
          `}
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
    if (state.time.inTransit) {
      return `
        <div class="view-market">
          <h2>Trade</h2>
          <p class="empty-state">No market in the void. Wait for port.</p>
        </div>
      `;
    }

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
    if (state.time.inTransit) {
      const dest = state.world.ports[state.time.transitDestination!];
      return `
        <div class="view-travel">
          <h2>In Transit</h2>
          <p>Destination: ${dest?.name ?? 'Unknown'}</p>
          <p>${getTransitTimeRemaining(state)} years remaining</p>
          <p class="transit-description">The stars drift past, impossibly slow. Time stretches. The hold hums.</p>
        </div>
      `;
    }

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
                <time>Era ${entry.timestamp.era}, Year ${entry.timestamp.year}</time>
              </header>
              <p>${entry.text}</p>
            </article>
          `).reverse().join('')}
        </div>
      </div>
    `;
  }

  function renderEvent(event: TriggeredEvent, _state: GameState): string {
    const passage = event.scenelet.passages[event.passageIndex];
    if (!passage) return '<p>Error: Invalid passage</p>';

    return `
      <div class="view-event">
        <h2>${event.scenelet.title}</h2>
        <div class="event-text">
          ${passage.text.split('\n\n').map(p => `<p>${p}</p>`).join('')}
        </div>
        ${passage.choices ? `
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
          ${!state.time.inTransit ? `
            <button class="btn btn-small" data-action="sell" data-instance="${inst.instanceId}">Sell</button>
          ` : ''}
        </div>
      </li>
    `;
  }

  function renderCrewItem(inst: CardInstance): string {
    const def = cardDefs.get(inst.cardDefId);
    if (!def) return '';

    const lifespan = def.lifespan ?? 80;
    const ageDisplay = lifespan < 0 ? 'Ageless' : `Age ${inst.age ?? 0}/${lifespan}`;

    return `
      <li class="card-item crew-item">
        <div class="card-info">
          <span class="card-name">${def.name}</span>
          <span class="card-age">${ageDisplay}</span>
        </div>
        <p class="card-desc">${def.description}</p>
        ${def.flavorText ? `<p class="card-flavor">${def.flavorText}</p>` : ''}
      </li>
    `;
  }

  function getTransitTimeRemaining(state: GameState): number {
    if (!state.time.transitArrivesAt) return 0;
    return Math.max(0, state.time.transitArrivesAt.year - state.time.year);
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
            controller.triggerEvent();
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
          case 'buy':
            controller.dispatch({
              type: 'TRADE_BUY',
              payload: { cardDefId: el.dataset.card as any, quantity: 1 }
            });
            render();
            break;
          case 'hire':
            controller.dispatch({
              type: 'CREW_HIRE',
              payload: { cardDefId: el.dataset.card as any }
            });
            render();
            break;
          case 'sell':
            controller.dispatch({
              type: 'TRADE_SELL',
              payload: { instanceId: el.dataset.instance as any }
            });
            render();
            break;
          case 'travel':
            controller.dispatch({
              type: 'TRAVEL',
              payload: { destination: el.dataset.dest as any }
            });
            render();
            break;
          case 'refuel':
            controller.dispatch({ type: 'REFUEL', payload: { amount: 10 } });
            render();
            break;
          case 'resupply':
            controller.dispatch({ type: 'RESUPPLY', payload: { amount: 10 } });
            render();
            break;
          case 'repair':
            controller.dispatch({ type: 'REPAIR', payload: { amount: 10 } });
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

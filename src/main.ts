import { createGameController } from './core/controller.js';
import { createRenderer } from './ui/renderer.js';
import { buildCardDefMap } from './content/cards/index.js';
import { ALL_SCENELETS } from './content/scenelets/index.js';

function main() {
  const cardDefs = buildCardDefMap();
  const controller = createGameController(cardDefs, ALL_SCENELETS);
  
  const loaded = controller.load();
  if (!loaded) {
    console.log('Starting new game');
  } else {
    console.log('Loaded saved game');
  }

  const container = document.getElementById('app');
  if (!container) {
    throw new Error('Could not find #app element');
  }

  const renderer = createRenderer(container, controller, cardDefs);
  renderer.render();
}

document.addEventListener('DOMContentLoaded', main);

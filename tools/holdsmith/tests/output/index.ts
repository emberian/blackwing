export { branching_test } from './branching.js';
export { simple_test } from './simple.js';

import type { Scenelet } from '../../core/types.js';

export const ALL_COMPILED_SCENELETS: Scenelet[] = [branching_test, simple_test];
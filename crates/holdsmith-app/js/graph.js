/**
 * Cytoscape.js setup for CFG visualization.
 */

// Store cytoscape instances by element
const graphs = new WeakMap();

/**
 * Render a CFG graph in the given element.
 * @param {HTMLElement} element - The container element
 * @param {string} graphDataJson - JSON string with nodes and edges
 */
window.renderCfgGraph = function(element, graphDataJson) {
    const data = JSON.parse(graphDataJson);

    // Destroy existing instance
    const existing = graphs.get(element);
    if (existing) {
        existing.destroy();
    }

    // Convert to cytoscape format
    const elements = [
        ...data.nodes.map(n => ({
            group: 'nodes',
            data: n.data,
        })),
        ...data.edges.map(e => ({
            group: 'edges',
            data: e.data,
        })),
    ];

    const cy = cytoscape({
        container: element,
        elements,

        style: [
            {
                selector: 'node',
                style: {
                    'label': 'data(label)',
                    'text-valign': 'center',
                    'text-halign': 'center',
                    'background-color': '#3c3c3c',
                    'color': '#cccccc',
                    'font-size': '11px',
                    'width': '80px',
                    'height': '40px',
                    'shape': 'round-rectangle',
                    'border-width': '1px',
                    'border-color': '#5c5c5c',
                },
            },
            {
                selector: 'node[?is_entry]',
                style: {
                    'border-color': '#89d185',
                    'border-width': '2px',
                },
            },
            {
                selector: 'node[!is_reachable]',
                style: {
                    'background-color': '#4a2020',
                    'border-color': '#f14c4c',
                },
            },
            {
                selector: 'node.highlighted',
                style: {
                    'background-color': '#094771',
                    'border-color': '#0078d4',
                    'border-width': '2px',
                },
            },
            {
                selector: 'edge',
                style: {
                    'label': 'data(label)',
                    'width': 2,
                    'line-color': '#5c5c5c',
                    'target-arrow-color': '#5c5c5c',
                    'target-arrow-shape': 'triangle',
                    'curve-style': 'bezier',
                    'font-size': '9px',
                    'color': '#969696',
                    'text-rotation': 'autorotate',
                    'text-margin-y': -10,
                },
            },
            {
                selector: 'edge[?has_condition]',
                style: {
                    'line-style': 'dashed',
                    'line-color': '#cca700',
                    'target-arrow-color': '#cca700',
                },
            },
        ],

        layout: {
            name: 'dagre',
            rankDir: 'TB',
            nodeSep: 50,
            rankSep: 80,
            padding: 20,
        },

        // Disable zooming/panning for simplicity
        userZoomingEnabled: true,
        userPanningEnabled: true,
        boxSelectionEnabled: false,
    });

    // Try dagre layout, fall back to grid if not available
    try {
        cy.layout({ name: 'dagre', rankDir: 'TB' }).run();
    } catch (e) {
        cy.layout({ name: 'grid' }).run();
    }

    // Add click handler for nodes
    cy.on('tap', 'node', function(evt) {
        const node = evt.target;
        const passageIndex = node.data('passage_index');
        console.log('Clicked passage:', passageIndex);
        // TODO: Dispatch event to Rust to jump to passage
    });

    graphs.set(element, cy);
};

/**
 * Highlight a node in the graph.
 * @param {HTMLElement} element - The container element
 * @param {string} nodeId - The node ID to highlight
 */
window.highlightCfgNode = function(element, nodeId) {
    const cy = graphs.get(element);
    if (!cy) return;

    cy.nodes().removeClass('highlighted');
    cy.$('#' + nodeId).addClass('highlighted');
};

/**
 * Clear all highlighting.
 * @param {HTMLElement} element - The container element
 */
window.clearCfgHighlight = function(element) {
    const cy = graphs.get(element);
    if (!cy) return;

    cy.nodes().removeClass('highlighted');
};

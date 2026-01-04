//! Cytoscape.js bindings for CFG visualization.

#![allow(dead_code)]

use holdsmith_analyzer::SceneCfg;
use wasm_bindgen::prelude::*;
use web_sys::HtmlElement;

#[wasm_bindgen]
extern "C" {
    /// Render a CFG graph using Cytoscape.js.
    #[wasm_bindgen(js_namespace = window, js_name = renderCfgGraph)]
    fn js_render_cfg(element: &HtmlElement, graph_data: &str);

    /// Highlight a node in the CFG graph.
    #[wasm_bindgen(js_namespace = window, js_name = highlightCfgNode)]
    fn js_highlight_node(element: &HtmlElement, node_id: &str);

    /// Clear CFG graph highlighting.
    #[wasm_bindgen(js_namespace = window, js_name = clearCfgHighlight)]
    fn js_clear_highlight(element: &HtmlElement);
}

/// Render a CFG in the given element.
pub fn render_cfg(element: &HtmlElement, cfg: &SceneCfg) {
    // Convert CFG to Cytoscape format
    let graph_data = cfg_to_cytoscape(cfg);
    let json = serde_json::to_string(&graph_data).unwrap_or_else(|_| "{}".to_string());
    js_render_cfg(element, &json);
}

/// Highlight a passage node in the CFG.
#[allow(dead_code)]
pub fn highlight_node(element: &HtmlElement, passage_index: usize) {
    js_highlight_node(element, &format!("p{}", passage_index));
}

/// Clear any highlighting.
#[allow(dead_code)]
pub fn clear_highlight(element: &HtmlElement) {
    js_clear_highlight(element);
}

/// Convert a SceneCfg to Cytoscape.js elements format.
fn cfg_to_cytoscape(cfg: &SceneCfg) -> CytoscapeData {
    use holdsmith_analyzer::{CfgTarget, NodeId};

    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    // Add nodes from CFG
    for node in &cfg.nodes {
        nodes.push(CytoscapeNode {
            data: NodeData {
                id: format!("p{}", node.passage_index),
                label: format!("Passage {}", node.passage_index),
                passage_index: node.passage_index,
                is_entry: NodeId(node.passage_index) == cfg.entry,
                is_exit: node.is_exit,
                is_reachable: true, // TODO: Get from analysis
            },
        });
    }

    // Add edges from CFG
    for edge in &cfg.edges {
        if let CfgTarget::Node(target) = edge.to {
            edges.push(CytoscapeEdge {
                data: EdgeData {
                    id: format!("e{}-{}-{}", edge.from.0, edge.choice_index, target.0),
                    source: format!("p{}", edge.from.0),
                    target: format!("p{}", target.0),
                    label: edge.text.to_string(),
                    has_condition: !edge.condition_source.is_empty(),
                },
            });
        }
    }

    CytoscapeData { nodes, edges }
}

#[derive(serde::Serialize)]
struct CytoscapeData {
    nodes: Vec<CytoscapeNode>,
    edges: Vec<CytoscapeEdge>,
}

#[derive(serde::Serialize)]
struct CytoscapeNode {
    data: NodeData,
}

#[derive(serde::Serialize)]
struct NodeData {
    id: String,
    label: String,
    passage_index: usize,
    is_entry: bool,
    is_exit: bool,
    is_reachable: bool,
}

#[derive(serde::Serialize)]
struct CytoscapeEdge {
    data: EdgeData,
}

#[derive(serde::Serialize)]
struct EdgeData {
    id: String,
    source: String,
    target: String,
    label: String,
    has_condition: bool,
}

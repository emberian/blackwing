use engine_core::{ContextId, GameState, Scene, TagProvider};
use engine_script::ScriptExecutor;

use crate::{ContentRegistry, Rng};

/// Context type for scene selection (journey vs port)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventContext {
    Journey,
    Port,
}

impl EventContext {
    pub fn matches(&self, scene_context: Option<&ContextId>) -> bool {
        match scene_context {
            None => true, // No context restriction means scene can appear anywhere
            Some(ctx) => {
                let ctx_str = ctx.as_str();
                match self {
                    EventContext::Journey => ctx_str == "journey" || ctx_str == "any",
                    EventContext::Port => ctx_str == "port" || ctx_str == "any",
                }
            }
        }
    }
}

/// Result of scene selection
pub struct SelectedScene<'a> {
    pub scene: &'a Scene,
    pub passage_index: usize,
}

/// Select a scene from eligible candidates using weighted random selection.
///
/// Returns None if no scenes are eligible.
pub fn select_scene<'a>(
    registry: &'a dyn ContentRegistry,
    state: &GameState,
    tags: &impl TagProvider,
    context: EventContext,
    rng: &mut Rng,
) -> Option<SelectedScene<'a>> {
    let eligible: Vec<&Scene> = registry
        .scenes()
        .filter(|scene| is_scene_eligible(scene, state, tags, context))
        .collect();

    if eligible.is_empty() {
        return None;
    }

    let weights: Vec<u32> = eligible.iter().map(|s| s.weight).collect();
    let selected_idx = rng.weighted_choice(&eligible, &weights)?;

    Some(SelectedScene {
        scene: eligible[selected_idx],
        passage_index: 0,
    })
}

/// Check if a scene meets all requirements to appear
pub fn is_scene_eligible(
    scene: &Scene,
    state: &GameState,
    tags: &impl TagProvider,
    context: EventContext,
) -> bool {
    // Check context matches
    if !context.matches(scene.context.as_ref()) {
        return false;
    }

    // Check cooldown
    if state.is_scene_on_cooldown(&scene.id, scene.cooldown) {
        return false;
    }

    // Check Rhai requirements (empty string = always eligible)
    if scene.rhai_requirements.is_empty() {
        return true;
    }

    let executor = ScriptExecutor::new();
    executor
        .eval_condition(&scene.rhai_requirements, state, tags)
        .unwrap_or(false) // If script errors, scene is not eligible
}

/// Get all eligible scenes for the current state (useful for debugging/analysis)
pub fn get_eligible_scenes<'a>(
    registry: &'a dyn ContentRegistry,
    state: &GameState,
    tags: &impl TagProvider,
    context: EventContext,
) -> Vec<&'a Scene> {
    registry
        .scenes()
        .filter(|scene| is_scene_eligible(scene, state, tags, context))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_matching() {
        let journey_ctx = Some(ContextId::new("journey"));
        let port_ctx = Some(ContextId::new("port"));
        let any_ctx = Some(ContextId::new("any"));

        // Journey context
        assert!(EventContext::Journey.matches(journey_ctx.as_ref()));
        assert!(!EventContext::Journey.matches(port_ctx.as_ref()));
        assert!(EventContext::Journey.matches(any_ctx.as_ref()));
        assert!(EventContext::Journey.matches(None));

        // Port context
        assert!(!EventContext::Port.matches(journey_ctx.as_ref()));
        assert!(EventContext::Port.matches(port_ctx.as_ref()));
        assert!(EventContext::Port.matches(any_ctx.as_ref()));
        assert!(EventContext::Port.matches(None));
    }
}

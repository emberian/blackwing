use engine_core::{CardDef, CardId, Scene, SceneId};

pub trait ContentRegistry: Send + Sync {
    fn get_scene(&self, id: &SceneId) -> Option<&Scene>;
    fn get_card(&self, id: &CardId) -> Option<&CardDef>;
    fn scenes(&self) -> Box<dyn Iterator<Item = &Scene> + '_>;
    fn cards(&self) -> Box<dyn Iterator<Item = &CardDef> + '_>;
    fn scene_count(&self) -> usize;
    fn card_count(&self) -> usize;
}

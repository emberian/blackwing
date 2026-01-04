use rustc_hash::FxHashSet;

use engine_core::{CardDef, GameState, TagCategoryId, TagId, TagProvider};

use crate::ContentRegistry;

pub struct DeckTagProvider<'a> {
    tags: FxHashSet<(TagCategoryId, TagId)>,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> DeckTagProvider<'a> {
    pub fn from_state(state: &'a GameState, registry: &'a dyn ContentRegistry) -> Self {
        let mut tags = FxHashSet::default();

        for card_id in state.deck_card_ids() {
            if let Some(card_def) = registry.get_card(card_id) {
                Self::collect_tags_from_card(&mut tags, card_def);
            }
        }

        for card_id in state.equipped_module_card_ids() {
            if let Some(card_def) = registry.get_card(card_id) {
                Self::collect_tags_from_card(&mut tags, card_def);
            }
        }

        Self {
            tags,
            _phantom: std::marker::PhantomData,
        }
    }

    fn collect_tags_from_card(tags: &mut FxHashSet<(TagCategoryId, TagId)>, card: &CardDef) {
        for tag in &card.tags {
            tags.insert((TagCategoryId::new(&*card.card_type), tag.clone()));
        }

        for tag in &card.effects.grants_tags {
            tags.insert((TagCategoryId::new("ship"), tag.clone()));
        }
    }
}

impl TagProvider for DeckTagProvider<'_> {
    fn has_tag(&self, category: &TagCategoryId, tag: &TagId) -> bool {
        self.tags.contains(&(category.clone(), tag.clone()))
    }
}

pub struct EmptyTagProvider;

impl TagProvider for EmptyTagProvider {
    fn has_tag(&self, _category: &TagCategoryId, _tag: &TagId) -> bool {
        false
    }
}

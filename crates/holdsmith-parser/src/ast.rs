use smol_str::SmolStr;

pub type Span = std::ops::Range<usize>;

pub trait Spanned {
    fn span(&self) -> Span;
}

#[derive(Debug, Clone)]
pub struct SceneFile {
    pub frontmatter: Frontmatter,
    pub passages: Vec<Passage>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Frontmatter {
    pub id: SmolStr,
    pub title: SmolStr,
    pub tags: Vec<SmolStr>,
    pub context: Context,
    pub weight: u32,
    pub cooldown: u32,
    pub requires: Option<Requirements>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Context {
    Journey,
    Port,
    #[default]
    Any,
}

#[derive(Debug, Clone, Default)]
pub struct Requirements {
    pub ship_tags: Vec<SmolStr>,
    pub crew_tags: Vec<SmolStr>,
    pub cargo_tags: Vec<SmolStr>,
    pub min_resources: Vec<ResourceCheck>,
    pub max_resources: Vec<ResourceCheck>,
    pub required_flags: Vec<SmolStr>,
    pub excluded_flags: Vec<SmolStr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ResourceCheck {
    pub resource: SmolStr,
    pub value: i64,
}

#[derive(Debug, Clone)]
pub struct Passage {
    pub name: SmolStr,
    pub content: Vec<PassageContent>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum PassageContent {
    Prose(Prose),
    Choice(Choice),
    RhaiBlock(RhaiBlock),
}

/// Inline Rhai script block in passage content
#[derive(Debug, Clone)]
pub struct RhaiBlock {
    pub source: SmolStr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Prose {
    pub text: SmolStr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Choice {
    pub text: SmolStr,
    pub condition: Option<Condition>,
    pub effects: Vec<Effect>,
    pub target: Option<NavigationTarget>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct NavigationTarget {
    pub target: SmolStr,
    pub is_end: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Condition {
    pub clauses: Vec<ConditionClause>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ConditionClause {
    Tag(TagCondition),
    Resource(ResourceCondition),
    Flag(FlagCondition),
}

#[derive(Debug, Clone)]
pub struct TagCondition {
    pub source: TagSource,
    pub tag: SmolStr,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagSource {
    Crew,
    Ship,
    Cargo,
}

#[derive(Debug, Clone)]
pub struct ResourceCondition {
    pub resource: SmolStr,
    pub operator: CompareOp,
    pub value: i64,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    Ge,
    Le,
    Gt,
    Lt,
    Eq,
    Ne,
}

#[derive(Debug, Clone)]
pub struct FlagCondition {
    pub flag: SmolStr,
    pub negated: bool,
    pub operator: Option<CompareOp>,
    pub value: Option<FlagValue>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum FlagValue {
    Bool(bool),
    Int(i64),
    String(SmolStr),
}

#[derive(Debug, Clone)]
pub enum Effect {
    Resource(ResourceEffect),
    Flag(FlagEffect),
    AddCard(AddCardEffect),
    RemoveCards(RemoveCardsEffect),
    Chronicle(ChronicleEffect),
    Damage(DamageEffect),
    Reputation(ReputationEffect),
    Script(ScriptEffect),
}

#[derive(Debug, Clone)]
pub struct ScriptEffect {
    pub source: SmolStr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ResourceEffect {
    pub resource: SmolStr,
    pub operator: AssignOp,
    pub value: i64,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    Add,
    Sub,
    Set,
}

#[derive(Debug, Clone)]
pub struct FlagEffect {
    pub flag: SmolStr,
    pub value: FlagValue,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AddCardEffect {
    pub card_id: SmolStr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct RemoveCardsEffect {
    pub pattern: SmolStr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ChronicleEffect {
    pub title: SmolStr,
    pub text: SmolStr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct DamageEffect {
    pub target: SmolStr,
    pub value: i64,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ReputationEffect {
    pub faction: SmolStr,
    pub operator: AssignOp,
    pub value: i64,
    pub span: Span,
}

impl Spanned for SceneFile {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl Spanned for Passage {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl Spanned for Choice {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

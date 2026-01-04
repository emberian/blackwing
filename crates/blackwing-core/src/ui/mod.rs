//! Declarative UI types that systems can emit.
//!
//! Systems can influence the UI in two ways:
//! 1. **Imperative**: Emit `SetUiRoot` or `ShowModal` effects with `UiNode` trees
//! 2. **Reactive**: Set world state that UI binds to via `Bound` nodes
//!
//! # Example
//!
//! ```ignore
//! // In a Rhai system:
//! set_ui(vstack([
//!     passage("Welcome, traveler!"),
//!     choices([
//!         #{ text: "Trade", action: "start_trade" },
//!         #{ text: "Leave", action: "end_dialogue" },
//!     ])
//! ]));
//! ```

mod node;

pub use node::{UiChoice, UiNode, UiResource};

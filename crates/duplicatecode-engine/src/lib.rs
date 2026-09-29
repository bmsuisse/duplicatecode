//! duplicatecode engine: static (LLM-free) duplicate/similar code detection.

pub mod diff;
pub mod fingerprint;
pub mod index;
pub mod lang;
pub mod naming;
pub mod similarity;
pub mod units;

pub use index::{find_matches, Corpus, load_units, units_from_file, Match, MatchOptions};
pub use lang::Lang;
pub use units::{extract_units, Unit};

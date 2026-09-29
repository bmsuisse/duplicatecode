//! duplicatecode engine: static (LLM-free) duplicate/similar code detection.

pub mod diff;
pub mod fingerprint;
pub mod index;
pub mod lang;
pub mod mutate;
pub mod naming;
pub mod similarity;
pub mod units;

pub use index::{find_matches, Corpus, load_file_units, load_units, load_units_with, units_from_file, walk_files, Match, MatchOptions};
pub use lang::Lang;
pub use units::{extract_file_unit, extract_units, Unit};

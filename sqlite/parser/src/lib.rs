pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod token;

/// Owned key for identifier maps and sets.
///
/// The stored text is ASCII-case-folded. Borrow a map keyed by this type with
/// [`IdentKeyStr`], usually through [`ast::Name::as_key_str`].
pub type IdentKey = identstr::Key;
/// Borrowed, allocation-free view used to compare or look up identifiers.
///
/// [`IdentKeyStr::as_str`](identstr::KeyStr::as_str) returns the original text;
/// equality, ordering, and hashing apply SQLite's ASCII case-insensitive rules.
pub type IdentKeyStr = identstr::KeyStr;
pub use parser::MAX_EXPR_DEPTH;

type Result<T, E = error::Error> = std::result::Result<T, E>;

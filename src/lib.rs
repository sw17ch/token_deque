//! A double-ended queue (deque) that allows internal nodes to be
//! removed in addition to the front and the back of the list.
//!
//! Internally, the deque uses a `Vec`, and tracks next, previous,
//! front, and back elements by index.
//!
//! As items are removed from the deque, their memory in the `Vec` is
//! put on an internal free list. This free list is used when items
//! are inserted into the list before the internal `Vec` is expanded.

mod cursor;
mod deque;
mod iterators;
mod slot;
mod token;

pub use crate::cursor::{Cursor, CursorMut};
pub use crate::deque::Deque;
pub use crate::iterators::{DrainBack, DrainFront, IterBack, IterFront};
pub use crate::token::Token;

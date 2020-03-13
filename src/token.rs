/// A token representing an item in the `List`. It can be used to try
/// and remove the item from the list, or try to get the value of the
/// item in the list. It contains a generation number that prevents
/// the wrong item (that may have come to inhabit the same location)
/// from being removed.
///
/// Tokens can be stored in other data structures, and do not have
/// lifetime bindings to the list that created them. Furthermore, they
/// can safely be serialized as they do not contain pointers.
///
/// While the type system allows it, using a `Token` with a list other
/// than the one that created it will result in (likely) unexpected
/// behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub(crate) ix: usize,
    pub(crate) generation: usize,
}

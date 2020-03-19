use crate::iterators::{DrainBack, DrainFront, IterBack, IterFront};
use crate::slot::Slot;
use crate::token::Token;
use std::fmt;
use std::iter::FromIterator;
use std::usize;

/// A deque that supports removing of nodes not in front or back
/// position, but also nodes in front and back position.
pub struct Deque<T> {
    // Index of the first element on the free list. MAX when the
    // free-list is empty.
    free_list: usize,
    // The index of the front of the deque. MAX when the deque is empty.
    pub(crate) front: usize,
    // The index of the back of the deque. MAX when the deque is empty.
    pub(crate) back: usize,
    // The next generation number.
    next_generation: usize,
    // The number of slots currently used by entries.
    len_used: usize,
    // The number of slots currently on the free list.
    len_free: usize,
    // The memory used to back the LRU structure.
    pub(crate) slots: Vec<Slot<T>>,
}

impl<T> fmt::Debug for Deque<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_list().entries(self.iter_front()).finish()
    }
}

impl<T> Default for Deque<T> {
    fn default() -> Self {
        Self {
            free_list: usize::MAX,
            front: usize::MAX,
            back: usize::MAX,
            next_generation: 0,
            len_used: 0,
            len_free: 0,
            slots: Vec::new(),
        }
    }
}

impl<T> Deque<T> {
    /// Creates an empty `Deque`. No allocations are performed until
    /// values are added.
    ///
    /// # Examples
    ///
    /// ```
    /// use token_deque::Deque;
    ///
    /// let deque: Deque<u32> = Deque::new();
    /// ```
    pub fn new() -> Deque<T> {
        Default::default()
    }

    /// Create a new `Deque` instance with a freelist at least
    /// `capacity` elements deep.
    ///
    /// # Examples
    ///
    /// ```
    /// use token_deque::Deque;
    ///
    /// let deque: Deque<u32> = Deque::with_capacity(16);
    /// ```
    pub fn with_capacity(capacity: usize) -> Deque<T> {
        let mut vec = Vec::with_capacity(capacity);

        let mut next = usize::MAX;
        for i in 0..capacity {
            vec.push(Slot::new_free(next));
            next = i;
        }

        Deque {
            free_list: next,
            front: usize::MAX,
            back: usize::MAX,
            next_generation: 0,
            len_used: 0,
            len_free: capacity,
            slots: vec,
        }
    }

    /// Reserves capacity for at least `additional` more elements to
    /// be inserted into the given `Deque`. Note: this only expands
    /// the size of the underlying `Vec`. It does not add the reserved
    /// elements to the free list.
    ///
    /// # Examples
    ///
    /// ```
    /// use token_deque::Deque;
    ///
    /// let mut l: Deque<u32> = Deque::new();
    /// l.reserve(16);
    /// ```
    pub fn reserve(&mut self, additional: usize) {
        self.slots.reserve(additional)
    }

    /// Returns how many items could be held without resizing the
    /// internal vector. Note: this is not necesarily `len() + len_freelist()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use token_deque::Deque;
    ///
    /// let d: Deque<u8> = Deque::with_capacity(16);
    /// assert_eq!(16, d.capacity());
    /// ```
    pub fn capacity(&self) -> usize {
        self.slots.capacity()
    }

    /// The number of items in the deque.
    ///
    /// # Examples
    ///
    /// ```
    /// use token_deque::Deque;
    ///
    /// let mut d: Deque<u8> = Deque::new();
    ///
    /// d.push_front(1);
    /// d.push_back(2);
    /// assert_eq!(2, d.len());
    ///
    /// d.pop_front();
    /// assert_eq!(1, d.len());
    /// ```
    pub fn len(&self) -> usize {
        self.len_used
    }

    /// True when the deque is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use token_deque::Deque;
    ///
    /// let mut d: Deque<u8> = Deque::new();
    ///
    /// assert!(d.is_empty());
    ///
    /// d.push_front(1);
    /// assert!(!d.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        0 == self.len_used
    }

    /// The number of entries on the deque's freelist.
    ///
    /// # Examples
    ///
    /// ```
    /// use token_deque::Deque;
    ///
    /// let mut d: Deque<u8> = Deque::new();
    ///
    /// assert_eq!(0, d.len_freelist());
    ///
    /// d.push_front(1);
    /// assert_eq!(0, d.len_freelist());
    ///
    /// d.pop_front();
    /// assert_eq!(1, d.len_freelist());
    ///
    /// d.push_front(2);
    /// assert_eq!(0, d.len_freelist());
    /// ```
    pub fn len_freelist(&self) -> usize {
        self.len_free
    }

    /// Insert `data` into the front of the deque.
    ///
    /// # Examples
    ///
    /// ```
    /// use token_deque::Deque;
    ///
    /// let mut l = Deque::new();
    /// let tok = l.push_front(10);
    ///
    /// assert_eq!(Some(&10), l.get(&tok));
    /// assert_eq!(Some(10), l.remove(&tok));
    /// ```
    pub fn push_front(&mut self, data: T) -> Token {
        let (new_ix, new_generation) = self.allocate(usize::MAX, self.front, data);

        // Update the old front of the deque so that it points to the
        // new front we just inserted.
        if usize::MAX != self.front {
            self.slots[self.front]
                .get_used_mut()
                .expect("self.slots[self.front] should always be a used slot")
                .set_front(new_ix);
        }
        // Repoint the front of the deque at the new front we just
        // inserted.
        self.front = new_ix;

        // If the back was not yet set, set it to the front.
        if usize::MAX == self.back {
            self.back = new_ix;
        }

        Token {
            ix: new_ix,
            generation: new_generation,
        }
    }

    /// Insert `data` into the back of the deque. Returns a token that
    /// can be used to retrieve the data again using `get()` or
    /// `remove()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use token_deque::Deque;
    ///
    /// let mut l = Deque::new();
    /// let tok = l.push_back(10);
    ///
    /// assert_eq!(Some(&10), l.get(&tok));
    /// assert_eq!(Some(10), l.remove(&tok));
    /// ```
    pub fn push_back(&mut self, data: T) -> Token {
        let (new_ix, new_generation) = self.allocate(self.back, usize::MAX, data);

        // Update the old back of the deque so that it points to the
        // new back we just inserted.
        if usize::MAX != self.back {
            self.slots[self.back]
                .get_used_mut()
                .expect("self.slots[self.back] should always be a used slot")
                .set_back(new_ix);
        }
        // Repoint the back of the deque at the new back we just
        // inserted.
        self.back = new_ix;

        // If the front was not yet set, set it to the back.
        if usize::MAX == self.front {
            self.front = new_ix;
        }

        Token {
            ix: new_ix,
            generation: new_generation,
        }
    }

    /// Remove the front of the deque and return it. If the deque is
    /// empty, `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use token_deque::Deque;
    ///
    /// let mut l = Deque::new();
    /// l.push_back(10);
    /// l.push_back(20);
    ///
    /// assert_eq!(Some(10), l.pop_front());
    /// assert_eq!(Some(20), l.pop_front());
    /// assert_eq!(None, l.pop_front());
    /// ```
    pub fn pop_front(&mut self) -> Option<T> {
        if usize::MAX != self.front {
            Some(self.remove_unchecked(self.front))
        } else {
            None
        }
    }

    /// Remove the back of the deque and return it. If the deque is
    /// empty, `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use token_deque::Deque;
    ///
    /// let mut l = Deque::new();
    /// l.push_front(10);
    /// l.push_front(20);
    ///
    /// assert_eq!(Some(10), l.pop_back());
    /// assert_eq!(Some(20), l.pop_back());
    /// assert_eq!(None, l.pop_back());
    /// ```
    pub fn pop_back(&mut self) -> Option<T> {
        if usize::MAX != self.back {
            Some(self.remove_unchecked(self.back))
        } else {
            None
        }
    }

    /// Get the front value of the deque. If the deque is empty, `None`
    /// is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use token_deque::Deque;
    ///
    /// let mut l = Deque::new();
    /// let tok = l.push_back(10);
    ///
    /// assert_eq!(Some(&10), l.get_front());
    /// ```
    pub fn get_front(&self) -> Option<&T> {
        if usize::MAX != self.front {
            Some(
                self.slots[self.front]
                    .get_used()
                    .expect("self.slots[self.front] should always be a used slot")
                    .data(),
            )
        } else {
            None
        }
    }

    /// Get the front value of the deque as a mutable reference. If the
    /// deque is empty, `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use token_deque::Deque;
    ///
    /// let mut l = Deque::new();
    /// let tok = l.push_back(10);
    ///
    /// l.get_front_mut().map(|i| *i += 10);
    ///
    /// assert_eq!(Some(&20), l.get_front());
    /// ```
    pub fn get_front_mut(&mut self) -> Option<&mut T> {
        if usize::MAX != self.front {
            Some(
                self.slots[self.front]
                    .get_used_mut()
                    .expect("self.slots[self.front] should always be a used slot")
                    .data_mut(),
            )
        } else {
            None
        }
    }

    /// Get the back of the deque. If the deque is empty, `None` is
    /// returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use token_deque::Deque;
    ///
    /// let mut l = Deque::new();
    /// let tok = l.push_front(10);
    ///
    /// assert_eq!(Some(&10), l.get_back());
    /// ```
    pub fn get_back(&self) -> Option<&T> {
        if usize::MAX != self.back {
            Some(
                self.slots[self.back]
                    .get_used()
                    .expect("self.slots[self.back] should always be a used slot")
                    .data(),
            )
        } else {
            None
        }
    }

    /// Get the back of the deque as a mutable reference. If the deque
    /// is empty, `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use token_deque::Deque;
    ///
    /// let mut l = Deque::new();
    /// let tok = l.push_front(10);
    ///
    /// l.get_back_mut().map(|i| *i += 10);
    ///
    /// assert_eq!(Some(&20), l.get_back());
    /// ```
    pub fn get_back_mut(&mut self) -> Option<&mut T> {
        if usize::MAX != self.back {
            Some(
                self.slots[self.back]
                    .get_used_mut()
                    .expect("self.slots[self.back] should always be a used slot")
                    .data_mut(),
            )
        } else {
            None
        }
    }

    /// Get a reference to the item associated with `token`. If the
    /// item has been removed, then `None` will be returned.
    ///
    /// # Examples
    ///
    /// A valid token results in `Some(&T)`.
    ///
    /// ```
    /// use token_deque::Deque;
    ///
    /// let mut l = Deque::new();
    /// let tok = l.push_front(10);
    ///
    /// assert_eq!(Some(&10), l.get(&tok));
    /// ```
    ///
    /// An invalid token results in `None`.
    ///
    /// ```
    /// use token_deque::Deque;
    ///
    /// let mut l = Deque::new();
    /// let tok = l.push_front(10);
    ///
    /// // This removes the 10 we just pushed which invalidates the
    /// // tok returned by push_front.
    /// l.pop_front();
    ///
    /// assert_eq!(None, l.get(&tok));
    /// ```
    pub fn get(&self, token: &Token) -> Option<&T> {
        let Token { ix, generation } = token;

        self.slots
            .get(*ix)
            .and_then(|s| s.get_used())
            .and_then(|u| u.as_generation(*generation))
            .map(|u| u.data())
    }

    /// Get a mutable reference to the item associted with `token`. If
    /// the item has been removed, then `None` will be returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use token_deque::Deque;
    ///
    /// let mut l = Deque::new();
    /// let tok = l.push_front(10);
    ///
    /// l.get_mut(&tok).map(|i| *i += 10);
    ///
    /// assert_eq!(Some(&20), l.get(&tok));
    /// ```
    pub fn get_mut(&mut self, token: &Token) -> Option<&mut T> {
        let Token { ix, generation } = token;

        self.slots
            .get_mut(*ix)
            .and_then(|s| s.get_used_mut())
            .and_then(|u| u.as_generation_mut(*generation))
            .map(|u| u.data_mut())
    }

    /// Remove the item associated with the specified token from the
    /// deque. If the item has already been removed, `None` is
    /// returned. This consumes the token.
    ///
    /// # Examples
    ///
    /// ```
    /// use token_deque::Deque;
    ///
    /// let mut l = Deque::new();
    /// let tok = l.push_front(10);
    ///
    /// assert_eq!(Some(10), l.remove(&tok));
    /// assert_eq!(None, l.remove(&tok));
    /// ```
    pub fn remove(&mut self, token: &Token) -> Option<T> {
        let Token { ix, generation } = token;

        self.slots
            .get(*ix)
            .and_then(|s| s.get_used())
            .and_then(|v| v.as_generation(*generation).map(|_| ix))
            .map(|ix| self.remove_unchecked(*ix))
    }

    /// Create an iterator over the deque starting from the front.
    ///
    /// # Examples
    ///
    /// ```
    /// use token_deque::Deque;
    ///
    /// let mut d: Deque<u8> = Deque::new();
    ///
    /// d.push_back(1);
    /// d.push_back(2);
    /// d.push_back(3);
    ///
    /// let v: Vec<&u8> = d.iter_front().collect();
    /// assert_eq!(vec![&1, &2, &3], v);
    /// ```
    pub fn iter_front(&self) -> IterFront<T> {
        IterFront::new(self, self.front)
    }

    /// A draining iterator starting from the front position. All
    /// drained slots are moved onto the free list.
    ///
    /// # Examples
    ///
    /// ```
    /// use token_deque::Deque;
    ///
    /// let mut d: Deque<u8> = Deque::new();
    ///
    /// d.push_back(1);
    /// d.push_back(2);
    /// d.push_back(3);
    ///
    /// let v: Vec<u8> = d.drain_front().collect();
    /// assert_eq!(vec![1, 2, 3], v);
    /// assert_eq!(3, d.len_freelist());
    /// ```
    pub fn drain_front(&mut self) -> DrainFront<T> {
        DrainFront::new(self, self.front)
    }

    /// Create an iterator over the deque starting from the back.
    ///
    /// # Examples
    ///
    /// ```
    /// use token_deque::Deque;
    ///
    /// let mut d: Deque<u8> = Deque::new();
    ///
    /// d.push_back(1);
    /// d.push_back(2);
    /// d.push_back(3);
    ///
    /// let v: Vec<&u8> = d.iter_back().collect();
    /// assert_eq!(vec![&3, &2, &1], v);
    /// ```
    pub fn iter_back(&self) -> IterBack<T> {
        IterBack::new(self, self.back)
    }

    /// A draining iterator starting from the back position. All
    /// drained slots are moved onto the free list.
    ///
    /// # Examples
    ///
    /// ```
    /// use token_deque::Deque;
    ///
    /// let mut d: Deque<u8> = Deque::new();
    ///
    /// d.push_back(1);
    /// d.push_back(2);
    /// d.push_back(3);
    ///
    /// let v: Vec<u8> = d.drain_back().collect();
    /// assert_eq!(vec![3, 2, 1], v);
    /// assert_eq!(3, d.len_freelist());
    /// ```
    pub fn drain_back(&mut self) -> DrainBack<T> {
        DrainBack::new(self, self.back)
    }

    fn remove_unchecked(&mut self, ix: usize) -> T {
        let (front, data, back) = self
            .free(ix)
            .into_used()
            .expect("self.slots[ix] must be used in order to remove it")
            .take();

        if self.front == ix {
            debug_assert_eq!(usize::MAX, front);
            self.front = back;
        } else {
            debug_assert_ne!(usize::MAX, front);
            self.slots[front]
                .get_used_mut()
                .expect("self.slots[front] should always be a used slot")
                .set_back(back);
        }

        if self.back == ix {
            debug_assert_eq!(usize::MAX, back);
            self.back = front;
        } else {
            debug_assert_ne!(usize::MAX, back);
            self.slots[back]
                .get_used_mut()
                .expect("self.slots[back] should always be a used slot")
                .set_front(front);
        }

        data
    }

    pub(crate) fn allocate(&mut self, front: usize, back: usize, data: T) -> (usize, usize) {
        // Assuming a 64 bit usize and that we could add a new item to
        // the deque 10 billion times per second, it would take ~58
        // years for the generation to overflow. After that point, the
        // token that is constructed from the generation could be used
        // to remove or get an incorrect object from the deque if the
        // object at that index had the same generation 58 years
        // prior.
        //
        // We do a checked-add in order to save future developers from
        // having to hunt down this rare problem in ancient code
        // bases. Instead, we give them a once-in-a-lifetime panic.
        let generation = self.next_generation;
        self.next_generation = self
            .next_generation
            .checked_add(1)
            .expect("58 years have passed, or generations have become corrupted");

        self.len_used += 1;

        let s = Slot::new_used(front, back, generation, data);

        let ix = if usize::MAX == self.free_list {
            self.slots.push(s);
            self.slots.len() - 1
        } else {
            let ix = self.free_list;
            self.free_list = self.slots[ix]
                .get_free()
                .expect("self.slots[self.free_list] is expected to be free")
                .next();
            self.slots[ix] = s;
            self.len_free -= 1;
            ix
        };

        (ix, generation)
    }

    pub(crate) fn free(&mut self, ix: usize) -> Slot<T> {
        debug_assert!(self.slots[ix].get_used().is_some());

        self.len_used -= 1;

        let mut v = Slot::new_free(self.free_list);
        std::mem::swap(&mut v, &mut self.slots[ix]);
        self.free_list = ix;
        self.len_free += 1;
        v
    }
}

impl<T> FromIterator<T> for Deque<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut l = Self::new();
        for i in iter {
            l.push_back(i);
        }
        l
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn push_get_works() {
        let mut l = Deque::new();
        let t10 = l.push_front(10u8);

        let r = l.get(&t10);
        assert_eq!(Some(&10), r);
        let r = l.get_front();
        assert_eq!(Some(&10), r);
        let r = l.get_back();
        assert_eq!(Some(&10), r);

        let mut l = Deque::new();
        let t11 = l.push_back(11u8);

        let r = l.get(&t11);
        assert_eq!(Some(&11), r);
        let r = l.get_front();
        assert_eq!(Some(&11), r);
        let r = l.get_back();
        assert_eq!(Some(&11), r);
    }

    #[test]
    fn push_remove_works() {
        let mut l = Deque::new();
        let t10 = l.push_front(10u8);

        let r = l.remove(&t10);
        assert_eq!(Some(10), r);
    }

    #[test]
    fn push_remove_front_get_is_none() {
        let mut l = Deque::new();
        let t10 = l.push_front(10u8);

        let r = l.get_front();
        assert_eq!(Some(&10), r);

        let r = l.pop_front();
        assert_eq!(Some(10), r);

        let r = l.get(&t10);
        assert_eq!(None, r);
    }

    #[test]
    fn push_remove_back_get_is_none() {
        let mut l = Deque::new();
        let t10 = l.push_front(10u8);

        let r = l.get_back();
        assert_eq!(Some(&10), r);

        let r = l.pop_back();
        assert_eq!(Some(10), r);

        let r = l.get(&t10);
        assert_eq!(None, r);
    }

    #[test]
    fn push_back_works() {
        let mut l = Deque::new();
        let t10 = l.push_back(10u8);
        let t11 = l.push_back(11u8);

        let r = l.get(&t10);
        assert_eq!(Some(&10), r);
        let r = l.get(&t11);
        assert_eq!(Some(&11), r);

        let r = l.get_front();
        assert_eq!(Some(&10), r);
        let r = l.get_back();
        assert_eq!(Some(&11), r);
    }

    #[test]
    fn counts_work_as_expected() {
        let mut l = Deque::new();
        l.push_front(10u8);
        l.push_front(11u8);
        assert_eq!(2, l.len());
        assert_eq!(0, l.len_freelist());

        l.pop_back();
        assert_eq!(1, l.len());
        assert_eq!(1, l.len_freelist());

        l.pop_back();
        assert_eq!(0, l.len());
        assert_eq!(2, l.len_freelist());

        l.push_front(12u8);
        assert_eq!(1, l.len());
        assert_eq!(1, l.len_freelist());

        l.push_front(13u8);
        assert_eq!(2, l.len());
        assert_eq!(0, l.len_freelist());
    }

    #[test]
    fn get_mut_allows_values_to_be_replaced() {
        let mut l = Deque::new();
        l.push_front(10u8);
        let t = l.push_front(11u8);
        l.push_front(12u8);

        l.get_mut(&t).map(|v| *v = 20);

        let r = l.pop_back();
        assert_eq!(Some(10), r);
        let r = l.pop_back();
        assert_eq!(Some(20), r);
        let r = l.pop_back();
        assert_eq!(Some(12), r);
    }

    #[test]
    fn can_be_created_from_iterator() {
        let mut l = Deque::from_iter((0..5).into_iter());

        let r = l.pop_front();
        assert_eq!(Some(0), r);
        let r = l.pop_front();
        assert_eq!(Some(1), r);
        let r = l.pop_front();
        assert_eq!(Some(2), r);
        let r = l.pop_front();
        assert_eq!(Some(3), r);
        let r = l.pop_front();
        assert_eq!(Some(4), r);
    }

    #[test]
    fn with_capacity_preallocates_free_list() {
        let mut l = Deque::with_capacity(3);
        assert_eq!(3, l.len_freelist());
        assert_eq!(0, l.len());

        l.push_front(());
        assert_eq!(2, l.len_freelist());
        assert_eq!(1, l.len());

        // The underlying capacity should not have changed.
        assert_eq!(3, l.capacity());

        l.push_front(());
        l.push_front(());
        l.push_front(());

        assert_eq!(0, l.len_freelist());
        assert_eq!(4, l.len());

        // The underlying capacity should have expanded to handle 4
        // items.
        assert!(3 < l.capacity());
    }

    #[test]
    fn get_front_mut_allows_front_to_change_value() {
        let mut l = Deque::new();
        l.push_front(10u8);

        l.get_front_mut().map(|r| *r = 100);

        assert_eq!(Some(&100), l.get_front());
    }

    #[test]
    fn get_back_mut_allows_back_to_change_value() {
        let mut l = Deque::new();
        l.push_back(10u8);

        l.get_back_mut().map(|r| *r = 100);

        assert_eq!(Some(&100), l.get_front());
    }

    #[test]
    fn empty_list() {
        let mut l: Deque<u8> = Deque::new();
        let t = l.push_front(1);
        l.pop_front();

        assert!(l.is_empty());

        assert_eq!(None, l.get_front());
        assert_eq!(None, l.get_front_mut());

        assert_eq!(None, l.get_back());
        assert_eq!(None, l.get_back_mut());

        assert_eq!(None, l.pop_front());
        assert_eq!(None, l.pop_back());

        assert_eq!(None, l.get(&t));
    }

    #[test]
    fn generation_protects_against_getting_wrong_item() {
        let mut l: Deque<u8> = Deque::new();

        // Push in a value, and make sure we can see it.
        let t0 = l.push_front(1);
        assert_eq!(Some(&1), l.get(&t0));
        assert_eq!(Some(&mut 1), l.get_mut(&t0));

        // Pop off the value, and make sure we can no longer see it.
        l.pop_front();
        assert_eq!(None, l.get(&t0));
        assert_eq!(None, l.get_mut(&t0));

        // Push in a new value, and make sure we can see it.
        let t1 = l.push_front(2);
        assert_eq!(Some(&2), l.get(&t1));
        assert_eq!(Some(&mut 2), l.get_mut(&t1));

        // Check the original token, and make sure it's still None.
        assert_eq!(None, l.get(&t0));
        assert_eq!(None, l.get_mut(&t0));
    }

    #[test]
    fn reserve_increases_capacity() {
        let mut l: Deque<u8> = Deque::new();
        l.push_front(1);

        let cap = l.capacity();
        let res = cap + 16;

        l.reserve(res);

        assert!(l.capacity() >= res);
    }

    #[test]
    fn debug_string() {
        let mut l: Deque<u8> = Deque::new();

        l.push_back(1);
        l.push_back(2);
        l.push_back(3);

        assert_eq!("[1, 2, 3]", format!("{:?}", l));
    }

    #[test]
    fn default_works() {
        let mut l: Deque<u8> = Default::default();

        l.push_back(1);
        l.push_back(2);
        l.push_back(3);

        assert_eq!(vec![1, 2, 3], l.drain_front().collect::<Vec<u8>>());
    }
}

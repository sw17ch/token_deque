use crate::deque::Deque;
use std::usize;

/// An iterator over the deque starting from the front. It is
/// constructed from the [`iter_front`] method on `Deque`.
///
/// [`iter_front`]: struct.Deque.html#method.iter_front
pub struct IterFront<'l, T> {
    target: &'l Deque<T>,
    next_index: usize,
}

impl<'l, T> IterFront<'l, T> {
    pub(crate) fn new(target: &'l Deque<T>, next_index: usize) -> Self {
        Self { target, next_index }
    }
}

impl<'l, T> Iterator for IterFront<'l, T> {
    type Item = &'l T;

    fn next(&mut self) -> Option<Self::Item> {
        if usize::MAX != self.next_index {
            let r = self.target.slots[self.next_index]
                .get_used()
                .expect("self.target.slots[self.next_index] is expected to be used");
            self.next_index = r.back();
            Some(r.data())
        } else {
            None
        }
    }
}

/// An iterator over the deque starting from the back. It is
/// constructed from the [`iter_back`] method on `Deque`.
///
/// [`iter_back`]: struct.Deque.html#method.iter_back
pub struct IterBack<'l, T> {
    target: &'l Deque<T>,
    next_index: usize,
}

impl<'l, T> IterBack<'l, T> {
    pub(crate) fn new(target: &'l Deque<T>, next_index: usize) -> Self {
        Self { target, next_index }
    }
}

impl<'l, T> Iterator for IterBack<'l, T> {
    type Item = &'l T;

    fn next(&mut self) -> Option<Self::Item> {
        if usize::MAX != self.next_index {
            let r = &self.target.slots[self.next_index]
                .get_used()
                .expect("self.target.slots[self.next_index] is expected to be used");
            self.next_index = r.front();
            Some(r.data())
        } else {
            None
        }
    }
}

/// A draining iterator over the deque starting from the front. It is
/// constructed from the [`drain_front`] method on `Deque`.
///
/// [`drain_front`]: struct.Deque.html#method.drain_front
pub struct DrainFront<'l, T> {
    target: &'l mut Deque<T>,
    next_index: usize,
}

impl<'l, T> DrainFront<'l, T> {
    pub(crate) fn new(target: &'l mut Deque<T>, next_index: usize) -> Self {
        Self { target, next_index }
    }
}

impl<'l, T> Iterator for DrainFront<'l, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if usize::MAX != self.next_index {
            let r = self.target.free(self.next_index);
            let (_, value, back) = r
                .into_used()
                .expect("self.target.slots[self.next_index] is expected to be used")
                .take();
            self.next_index = back;
            Some(value)
        } else {
            None
        }
    }
}

/// A draining iterator over the deque starting from the front. It is
/// constructed from the [`drain_back`] method on `Deque`.
///
/// [`drain_back`]: struct.Deque.html#method.drain_back
pub struct DrainBack<'l, T> {
    target: &'l mut Deque<T>,
    next_index: usize,
}

impl<'l, T> DrainBack<'l, T> {
    pub(crate) fn new(target: &'l mut Deque<T>, next_index: usize) -> Self {
        Self { target, next_index }
    }
}

impl<'l, T> Iterator for DrainBack<'l, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if usize::MAX != self.next_index {
            let r = self.target.free(self.next_index);
            let (front, value, _) = r
                .into_used()
                .expect("self.target.slots[self.next_index] is expected to be used")
                .take();
            self.next_index = front;
            Some(value)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn filter_can_find_items() {
        let mut l = Deque::new();
        l.push_front(10u8);
        l.push_front(11u8);
        l.push_front(12u8);

        assert_eq!(Some(&10), l.iter_front().filter(|i| **i == 10).next());
        assert_eq!(Some(&11), l.iter_front().filter(|i| **i == 11).next());
        assert_eq!(Some(&12), l.iter_front().filter(|i| **i == 12).next());
        assert_eq!(None, l.iter_front().filter(|i| **i == 13).next());
    }

    #[test]
    fn iterator_filter_can_find_duplicates() {
        let mut l = Deque::new();
        l.push_back(10u8);
        l.push_back(11u8);
        l.push_back(12u8);
        l.push_back(13u8);
        l.push_back(14u8);

        let mut s = l.iter_front().filter(|i| 0 == *i % 2);

        assert_eq!(Some(&10), s.next());
        assert_eq!(Some(&12), s.next());
        assert_eq!(Some(&14), s.next());
        assert_eq!(None, s.next());

        let mut s = l.iter_back().filter(|i| 0 == *i % 2);

        assert_eq!(Some(&14), s.next());
        assert_eq!(Some(&12), s.next());
        assert_eq!(Some(&10), s.next());
        assert_eq!(None, s.next());
    }

    #[test]
    fn iters_finds_everything() {
        let mut l = Deque::new();
        l.push_front(10u8);
        let tok = l.push_front(11u8);
        l.push_front(12u8);

        assert_eq!(vec![&12, &11, &10], l.iter_front().collect::<Vec<&u8>>());
        assert_eq!(vec![&10, &11, &12], l.iter_back().collect::<Vec<&u8>>());

        l.remove(&tok);

        assert_eq!(vec![&12, &10], l.iter_front().collect::<Vec<&u8>>());
        assert_eq!(vec![&10, &12], l.iter_back().collect::<Vec<&u8>>());
    }

    #[test]
    fn drains_find_everything_and_leave_slots_free() {
        let mut l = Deque::new();
        l.push_front(10u8);
        l.push_front(11u8);
        l.push_front(12u8);

        assert_eq!(0, l.len_freelist());
        assert_eq!(vec![12, 11, 10], l.drain_front().collect::<Vec<u8>>());
        assert_eq!(3, l.len_freelist());

        let mut l = Deque::new();
        l.push_front(10u8);
        l.push_front(11u8);
        l.push_front(12u8);

        assert_eq!(0, l.len_freelist());
        assert_eq!(vec![10, 11, 12], l.drain_back().collect::<Vec<u8>>());
        assert_eq!(3, l.len_freelist());
    }
}

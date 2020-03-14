use crate::deque::Deque;
use crate::token::Token;
use std::usize;

/// A movable cursor over a `Deque`. It is constructed from the
/// [`cursor`] method on `Deque`.
///
/// [`cursor`]: struct.Deque.html#method.cursor
pub struct Cursor<'l, T> {
    target: &'l Deque<T>,
    focus: usize,
}

impl<'l, T> Cursor<'l, T> {
    pub(crate) fn new(target: &'l Deque<T>, focus: usize) -> Self {
        Self { target, focus }
    }

    /// Return a reference to the value focused by the cursor.
    pub fn get(&self) -> &T {
        self.target.slots[self.focus].get_used().unwrap().data()
    }

    /// Return the token referring to the current cursor focus.
    pub fn get_token(&self) -> Token {
        Token {
            ix: self.focus,
            generation: self.target.slots[self.focus]
                .get_used()
                .unwrap()
                .generation(),
        }
    }

    /// Attempts to move the cursor towards the front of the deque. If
    /// successful, returns a reference to the new focus. If
    /// unsuccessful, `None` is returned and the focus does not change
    /// focus.
    pub fn move_front(&mut self) -> Option<&T> {
        match self.target.slots[self.focus].get_used().unwrap().front() {
            usize::MAX => None,
            front => {
                self.focus = front;
                Some(self.target.slots[front].get_used().unwrap().data())
            }
        }
    }

    /// Attempts to move the cursor towards the back of the deque. If
    /// successful, returns a reference to the new focus. If
    /// unsuccessful, `None` is returned and the focus does not change
    /// focus.
    pub fn move_back(&mut self) -> Option<&T> {
        match self.target.slots[self.focus].get_used().unwrap().back() {
            usize::MAX => None,
            back => {
                self.focus = back;
                Some(self.target.slots[back].get_used().unwrap().data())
            }
        }
    }
}

/// A movable cursor over a `Deque` that provides mutable access to the
/// items in the deque. It is constructed from the [`cursor_mut`] method on
/// `Deque`.
///
/// [`cursor_mut`]: struct.Deque.html#method.cursor_mut
pub struct CursorMut<'l, T> {
    target: &'l mut Deque<T>,
    focus: usize,
}

impl<'l, T> CursorMut<'l, T> {
    pub(crate) fn new(target: &'l mut Deque<T>, focus: usize) -> Self {
        Self { target, focus }
    }

    /// Return a mutable reference to the value focused by the cursor.
    pub fn get(&mut self) -> &mut T {
        self.target.slots[self.focus]
            .get_used_mut()
            .unwrap()
            .data_mut()
    }

    /// Return the token referring to the current cursor focus.
    pub fn get_token(&self) -> Token {
        Token {
            ix: self.focus,
            generation: self.target.slots[self.focus]
                .get_used()
                .unwrap()
                .generation(),
        }
    }

    /// Attempts to move the cursor towards the front of the deque. If
    /// successful, returns a mutable reference to the new focus. If
    /// unsuccessful, `None` is returned and the focus does not change
    /// focus.
    pub fn move_front(&mut self) -> Option<&mut T> {
        match self.target.slots[self.focus].get_used().unwrap().front() {
            usize::MAX => None,
            front => {
                self.focus = front;
                Some(self.target.slots[front].get_used_mut().unwrap().data_mut())
            }
        }
    }

    /// Attempts to move the cursor towards the back of the deque. If
    /// successful, returns a mutable reference to the new focus. If
    /// unsuccessful, `None` is returned and the focus does not change
    /// focus.
    pub fn move_back(&mut self) -> Option<&mut T> {
        match self.target.slots[self.focus].get_used().unwrap().back() {
            usize::MAX => None,
            back => {
                self.focus = back;
                Some(self.target.slots[back].get_used_mut().unwrap().data_mut())
            }
        }
    }

    /// Push the item into the deque before the focus of the
    /// cursor. The cursor remains unmoved.
    pub fn push_front(&mut self, data: T) -> Token {
        match self.target.slots[self.focus].get_used().unwrap().front() {
            usize::MAX => {
                debug_assert_eq!(self.target.front, self.focus);
                let (new_ix, new_generation) = self.target.allocate(usize::MAX, self.focus, data);
                self.target.slots[self.focus]
                    .get_used_mut()
                    .unwrap()
                    .set_front(new_ix);
                self.target.front = new_ix;

                Token {
                    ix: new_ix,
                    generation: new_generation,
                }
            }
            front => {
                let (new_ix, new_generation) = self.target.allocate(front, self.focus, data);
                self.target.slots[self.focus]
                    .get_used_mut()
                    .unwrap()
                    .set_front(new_ix);
                self.target.slots[front]
                    .get_used_mut()
                    .unwrap()
                    .set_back(new_ix);

                Token {
                    ix: new_ix,
                    generation: new_generation,
                }
            }
        }
    }

    /// Push the item into the deque after the focus of the cursor. The
    /// cursor remains unmoved.
    pub fn push_back(&mut self, data: T) -> Token {
        match self.target.slots[self.focus].get_used().unwrap().back() {
            usize::MAX => {
                debug_assert_eq!(self.target.back, self.focus);
                let (new_ix, new_generation) = self.target.allocate(self.focus, usize::MAX, data);
                self.target.slots[self.focus]
                    .get_used_mut()
                    .unwrap()
                    .set_back(new_ix);
                self.target.back = new_ix;

                Token {
                    ix: new_ix,
                    generation: new_generation,
                }
            }
            back => {
                let (new_ix, new_generation) = self.target.allocate(self.focus, back, data);
                self.target.slots[self.focus]
                    .get_used_mut()
                    .unwrap()
                    .set_back(new_ix);
                self.target.slots[back]
                    .get_used_mut()
                    .unwrap()
                    .set_front(new_ix);

                Token {
                    ix: new_ix,
                    generation: new_generation,
                }
            }
        }
    }

    /// If the focus is not the front of the deque, remove the item
    /// before the focus and return it.
    pub fn remove_front(&mut self) -> Option<T> {
        match self.target.slots[self.focus].get_used().unwrap().front() {
            usize::MAX => None,
            front => {
                let (ffront, v, fback) = self.target.free(front).into_used().unwrap().take();
                debug_assert_eq!(self.focus, fback);

                // Update our new front to be our old front's front.
                self.target.slots[self.focus]
                    .get_used_mut()
                    .unwrap()
                    .set_front(ffront);

                // If our old front was not the front of the deque, set
                // our old front's front's back to us.
                if ffront != usize::MAX {
                    self.target.slots[ffront]
                        .get_used_mut()
                        .unwrap()
                        .set_back(self.focus);
                }

                Some(v)
            }
        }
    }

    /// If the focus is not the back of the deque, remove the item
    /// before the focus and return it.
    pub fn remove_back(&mut self) -> Option<T> {
        match self.target.slots[self.focus].get_used().unwrap().back() {
            usize::MAX => None,
            back => {
                let (bfront, v, bback) = self.target.free(back).into_used().unwrap().take();
                debug_assert_eq!(self.focus, bfront);

                // Update our new back to be our old back's back.
                self.target.slots[self.focus]
                    .get_used_mut()
                    .unwrap()
                    .set_back(bback);

                // If our old back was not the back of the deque, set
                // our old back's back's front to us.
                if bback != usize::MAX {
                    self.target.slots[bback]
                        .get_used_mut()
                        .unwrap()
                        .set_front(self.focus);
                }

                Some(v)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cursor_can_navigate() {
        let mut l = Deque::new();
        l.push_back(1u8);
        let t = l.push_back(2u8);
        l.push_back(3u8);

        let mut c = l.cursor(&t).unwrap();
        assert_eq!(&2, c.get());

        assert_eq!(Some(&1), c.move_front());
        assert_eq!(&1, c.get());

        assert_eq!(None, c.move_front());
        assert_eq!(&1, c.get());

        assert_eq!(Some(&2), c.move_back());
        assert_eq!(&2, c.get());

        assert_eq!(Some(&3), c.move_back());
        assert_eq!(&3, c.get());

        assert_eq!(None, c.move_back());
        assert_eq!(&3, c.get());
    }

    #[test]
    fn cursor_can_return_a_token() {
        let mut l = Deque::new();
        l.push_back(1u8);
        let t = l.push_back(2u8);
        l.push_back(3u8);

        let mut c = l.cursor(&t).unwrap();
        assert_eq!(&2, c.get());

        c.move_front();
        let t = c.get_token();
        assert_eq!(Some(&1), l.get(&t));
    }

    #[test]
    fn cursormut_can_navigate() {
        let mut l = Deque::new();
        l.push_back(1u8);
        let t = l.push_back(2u8);
        l.push_back(3u8);

        let mut c = l.cursor_mut(&t).unwrap();
        assert_eq!(&2, c.get());

        assert_eq!(Some(&mut 1), c.move_front());
        assert_eq!(&1, c.get());

        assert_eq!(None, c.move_front());
        assert_eq!(&1, c.get());

        assert_eq!(Some(&mut 2), c.move_back());
        assert_eq!(&2, c.get());

        assert_eq!(Some(&mut 3), c.move_back());
        assert_eq!(&3, c.get());

        assert_eq!(None, c.move_back());
        assert_eq!(&3, c.get());
    }

    #[test]
    fn cursormut_can_return_a_token() {
        let mut l = Deque::new();
        l.push_back(1u8);
        let t = l.push_back(2u8);
        l.push_back(3u8);

        let mut c = l.cursor_mut(&t).unwrap();
        assert_eq!(&2, c.get());

        c.move_back();
        let t = c.get_token();
        assert_eq!(Some(&3), l.get(&t));
    }

    #[test]
    fn cursormut_push_front_and_push_back() {
        let mut l = Deque::new();
        l.push_back(1u8);
        let t = l.push_back(2u8);
        l.push_back(3u8);

        let mut c = l.cursor_mut(&t).unwrap();
        assert_eq!(&2, c.get());

        c.push_front(10);
        c.push_back(20);

        assert_eq!(
            vec![&1, &10, &2, &20, &3],
            l.iter_front().collect::<Vec<&u8>>()
        );
    }

    #[test]
    fn cursormut_remove_front_and_remove_back() {
        let mut l = Deque::new();
        l.push_back(1u8);
        l.push_back(2u8);
        let t = l.push_back(3u8);
        l.push_back(4u8);
        l.push_back(5u8);

        let mut c = l.cursor_mut(&t).unwrap();
        assert_eq!(&3, c.get());

        assert_eq!(Some(2), c.remove_front());
        assert_eq!(Some(1), c.remove_front());
        assert_eq!(None, c.remove_front());
        assert_eq!(Some(4), c.remove_back());
        assert_eq!(Some(5), c.remove_back());
        assert_eq!(None, c.remove_back());
    }

    #[test]
    fn cursor_front() {
        let mut l = Deque::new();
        l.push_back(1);
        l.push_back(2);
        l.push_back(3);

        let mut c = l.cursor_front().unwrap();
        assert_eq!(&1, c.get());
        c.move_back();
        assert_eq!(&2, c.get());
        c.move_back();
        assert_eq!(&3, c.get());

        assert_eq!(None, c.move_back());
        assert_eq!(&3, c.get());
    }

    #[test]
    fn cursor_front_mut() {
        let mut l = Deque::new();
        l.push_front(1);

        let mut c = l.cursor_front_mut().unwrap();

        c.push_front(10);
        c.push_back(20);

        assert_eq!(vec![&10, &1, &20], l.iter_front().collect::<Vec<&u8>>());
    }

    #[test]
    fn cursor_back() {
        let mut l = Deque::new();
        l.push_back(1);
        l.push_back(2);
        l.push_back(3);

        let mut c = l.cursor_back().unwrap();
        assert_eq!(&3, c.get());
        c.move_front();
        assert_eq!(&2, c.get());
        c.move_front();
        assert_eq!(&1, c.get());

        assert_eq!(None, c.move_front());
        assert_eq!(&1, c.get());
    }

    #[test]
    fn cursor_back_mut() {
        let mut l = Deque::new();
        l.push_front(1);

        let mut c = l.cursor_back_mut().unwrap();

        c.push_back(20);
        c.push_front(10);

        assert_eq!(vec![&10, &1, &20], l.iter_front().collect::<Vec<&u8>>());
    }
}

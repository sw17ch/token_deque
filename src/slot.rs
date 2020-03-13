pub(crate) struct Free(FreeInner);
pub(crate) struct FreeInner {
    // The next free slot.
    next: usize,
}

impl Free {
    fn new(next: usize) -> Free {
        Free(FreeInner { next })
    }

    pub(crate) fn next(&self) -> usize {
        self.0.next
    }
}

pub(crate) struct Used<T>(UsedInner<T>);
struct UsedInner<T> {
    // The index of the slot before this slot.
    front: usize,
    // The index of the slot after this slot.
    back: usize,
    // The generation ID for this slot.
    generation: usize,
    // The contained data.
    data: T,
}

impl<T> Used<T> {
    fn new(front: usize, back: usize, generation: usize, data: T) -> Used<T> {
        Used(UsedInner {
            front,
            back,
            generation,
            data,
        })
    }

    pub(crate) fn front(&self) -> usize {
        self.0.front
    }

    pub(crate) fn set_front(&mut self, new_front: usize) {
        self.0.front = new_front;
    }

    pub(crate) fn back(&self) -> usize {
        self.0.back
    }

    pub(crate) fn set_back(&mut self, new_back: usize) {
        self.0.back = new_back;
    }

    pub(crate) fn take(self) -> (usize, T, usize) {
        let Used(UsedInner {
            front, back, data, ..
        }) = self;
        (front, data, back)
    }

    pub(crate) fn as_generation(&self, g: usize) -> Option<&Used<T>> {
        if self.0.generation == g {
            Some(self)
        } else {
            None
        }
    }

    pub(crate) fn as_generation_mut(&mut self, g: usize) -> Option<&mut Used<T>> {
        if self.0.generation == g {
            Some(self)
        } else {
            None
        }
    }

    pub(crate) fn data(&self) -> &T {
        &self.0.data
    }

    pub(crate) fn data_mut(&mut self) -> &mut T {
        &mut self.0.data
    }
}

pub(crate) enum Slot<T> {
    Free(Free),
    Used(Used<T>),
}

impl<T> Slot<T> {
    pub(crate) fn new_free(next: usize) -> Slot<T> {
        Slot::Free(Free::new(next))
    }

    pub(crate) fn new_used(front: usize, back: usize, generation: usize, data: T) -> Slot<T> {
        Slot::Used(Used::new(front, back, generation, data))
    }

    pub(crate) fn get_used(&self) -> Option<&Used<T>> {
        if let Slot::Used(used) = self {
            Some(used)
        } else {
            None
        }
    }

    pub(crate) fn get_used_mut(&mut self) -> Option<&mut Used<T>> {
        if let Slot::Used(used) = self {
            Some(used)
        } else {
            None
        }
    }

    pub(crate) fn get_free(&self) -> Option<&Free> {
        if let Slot::Free(free) = self {
            Some(free)
        } else {
            None
        }
    }

    pub(crate) fn into_used(self) -> Option<Used<T>> {
        if let Slot::Used(used) = self {
            Some(used)
        } else {
            None
        }
    }
}

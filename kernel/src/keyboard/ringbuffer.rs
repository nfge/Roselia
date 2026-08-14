use core::mem::MaybeUninit;

pub struct RingBuffer<T, const N: usize> {
    buf: [MaybeUninit<T>; N],
    head: usize,
    tail: usize,
    full: bool,
}

impl<T: Copy, const N: usize> RingBuffer<T, N> {
    pub const fn new() -> Self {
        Self {
            buf: [MaybeUninit::uninit(); N],
            head: 0,
            tail: 0,
            full: false,
        }
    }

    pub fn push(&mut self, val: T) {
        self.buf[self.head] = MaybeUninit::new(val);

        self.head = (self.head + 1) % N;

        if self.full {
            self.tail = (self.tail + 1) % N;
        }

        self.full = self.head == self.tail;
    }

    pub fn pop(&mut self) -> Option<T> {
        if !self.full && self.head == self.tail {
            return None;
        }
        let val = unsafe { self.buf[self.tail].assume_init() };

        self.tail = (self.tail + 1) % N;
        self.full = false;

        Some(val)
    }
}
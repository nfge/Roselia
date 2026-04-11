pub struct RingBuffer {
    buf: [u8; 64],
    head: usize,
    tail: usize,
    full: bool,
}

impl RingBuffer {
    pub const fn new() -> Self {
        Self {
            buf: [0; 64],
            head: 0,
            tail: 0,
            full: false,
        }
    }
    pub fn push(&mut self, val: u8) {
        self.buf[self.head] = val;

        self.head = (self.head + 1) % 64;

        if self.full {
            self.tail = (self.tail + 1) % 64;
        }

        self.full = self.head == self.tail;
    }
    pub fn pop(&mut self) -> Option<u8> {
        if !self.full && self.head == self.tail {
            return None;
        }

        let val = self.buf[self.tail];

        self.tail = (self.tail + 1) % 64;

        self.full = false;

        Some(val)
    }
}

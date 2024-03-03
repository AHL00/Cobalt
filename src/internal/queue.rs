use std::ops::{Index, IndexMut};

pub struct SizedQueue<T: Copy, const N: usize> {
    data: [Option<T>; N],
    size: usize,
    pointer: usize,
    filled: usize,
}

impl<T: Copy, const N: usize> SizedQueue<T, N> {
    pub fn new() -> Self {
        Self {
            data: [None; N],
            size: N,
            pointer: 0,
            filled: 0,
        }
    }

    pub fn enqueue(&mut self, item: T) {
        if self.filled == self.size {
            // Remove the oldest item
            self.dequeue();
        }

        self.data[self.pointer] = Some(item);
        self.pointer = (self.pointer + 1) % self.size;
        self.filled += 1;
    }

    pub fn dequeue(&mut self) -> Option<T> {
        if self.filled == 0 {
            return None;
        }

        let index = (self.pointer + self.size - self.filled) % self.size;
        let item = self.data[index];
        self.data[index] = None;
        self.filled -= 1;

        item
    }

    pub fn get(&self, index: usize) -> Option<T> {
        if index >= self.filled {
            return None;
        }

        let index = (self.pointer + self.size - self.filled + index) % self.size;
        self.data[index]
    }

    pub fn len(&self) -> usize {
        self.filled
    }

    pub fn last(&self) -> Option<T> {
        if self.filled == 0 {
            return None;
        }

        let index = (self.pointer + self.size - 1) % self.size;
        self.data[index]
    }

    pub const fn capacity(&self) -> usize {
        N
    }
}

impl<T: Copy, const N: usize> Index<usize> for SizedQueue<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.filled {
            panic!("Index out of bounds");
        }

        let index = (self.pointer + self.size - self.filled + index) % self.size;
        self.data[index].as_ref().unwrap()
    }
}

pub struct SizedQueueIterator<T: Copy, const N: usize> {
    queue: SizedQueue<T, N>,
    index: usize,
}

impl<T: Copy, const N: usize> Iterator for SizedQueueIterator<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.queue.filled {
            return None;
        }

        let item = self.queue.get(self.index);
        self.index += 1;

        item
    }
}

impl<T: Copy, const N: usize> IntoIterator for SizedQueue<T, N> {
    type Item = T;
    type IntoIter = SizedQueueIterator<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        SizedQueueIterator {
            queue: self,
            index: 0,
        }
    }
}

impl<T: Copy, const N: usize> IndexMut<usize> for SizedQueue<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index >= self.filled {
            panic!("Index out of bounds");
        }

        let index = (self.pointer + self.size - self.filled + index) % self.size;
        self.data[index].as_mut().unwrap()
    }
}

//! Module for temperature readings buffer

use std::ops::Index;

/// Stack allocated fixed-length buffer with only push and into Vec operations
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FixedBuffer<T, const N: usize> {
    items: [T; N],
    index: usize,
}

impl<T, const N: usize> FixedBuffer<T, N> {
    /// Push a new value to the buffer, it will appear first in the result
    pub fn push(&mut self, value: T) {
        self.items[self.index] = value;
        self.index = if self.index == 0 {
            N - 1
        } else {
            self.index - 1
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= N {
            return None;
        }
        Some(&self.items[(self.index + 1 + index) % N])
    }
}
impl<T: Default + Copy, const N: usize> Default for FixedBuffer<T, N> {
    /// Create new fixed buffer, filled with default for generic param `T`
    fn default() -> Self {
        Self {
            items: [Default::default(); N],
            index: N - 1,
        }
    }
}
impl<T, const N: usize> Index<usize> for FixedBuffer<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("index out of bounds")
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FixedBufferIter<'a, T, const N: usize> {
    buffer: &'a FixedBuffer<T, N>,
    current: usize,
}
impl<'a, T: 'a, const N: usize> Iterator for FixedBufferIter<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        // map index to start at self.buffer.index
        let length = self.buffer.items.len();
        if self.current >= length {
            return None;
        }
        let res = self.buffer.get(self.current);
        self.current += 1;
        res
    }
}
impl<'a, T, const N: usize> IntoIterator for &'a FixedBuffer<T, N> {
    type Item = &'a T;
    type IntoIter = FixedBufferIter<'a, T, N>;

    fn into_iter(self) -> Self::IntoIter {
        FixedBufferIter {
            buffer: &self,
            current: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FixedBufferIntoIter<T, const N: usize> {
    buffer: FixedBuffer<T, N>,
    current: usize,
}
impl<T: Copy, const N: usize> Iterator for FixedBufferIntoIter<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        // map index to start at self.buffer.index
        let length = self.buffer.items.len();
        if self.current >= length {
            return None;
        }
        let res = self.buffer.get(self.current);
        self.current += 1;
        res.copied()
    }
}
impl<T: Copy, const N: usize> IntoIterator for FixedBuffer<T, N> {
    type Item = T;
    type IntoIter = FixedBufferIntoIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        FixedBufferIntoIter {
            buffer: self,
            current: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_buffer() {
        let mut buf: FixedBuffer<Option<u32>, 5> = FixedBuffer::default();
        buf.push(Some(5));
        buf.push(Some(10));
        buf.push(Some(15));
        let items: Vec<_> = buf.into_iter().collect();
        assert_eq!(vec![Some(15), Some(10), Some(5), None, None], items);
        buf.push(Some(20));
        buf.push(Some(25));
        buf.push(Some(30));
        buf.push(Some(35));
        let items: Vec<_> = buf.into_iter().collect();
        assert_eq!(
            vec![Some(35), Some(30), Some(25), Some(20), Some(15)],
            items
        );
    }

    #[test]
    fn test_indexing() {
        let mut buf: FixedBuffer<Option<u32>, 5> = FixedBuffer::default();
        buf.push(Some(5));
        buf.push(Some(10));
        buf.push(Some(15));
        assert_eq!(buf[0], Some(15));
        assert_eq!(buf[1], Some(10));
        assert_eq!(buf[2], Some(5));
        assert_eq!(buf.get(5), None);
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_index_out_of_bounds() {
        let mut buf: FixedBuffer<Option<u32>, 5> = FixedBuffer::default();
        buf.push(Some(5));
        buf.push(Some(10));
        buf.push(Some(15));
        println!("{:?}", buf[5]);
    }
}

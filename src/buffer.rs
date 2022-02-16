//! Module for temperature readings buffer

/// Stack allocated fixed-length buffer with only push and into Vec operations
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FixedBuffer<T, const N: usize> {
    items: [T; N],
    index: usize,
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
impl<T: Clone, const N: usize> From<FixedBuffer<T, N>> for Vec<T> {
    fn from(slf: FixedBuffer<T, N>) -> Self {
        let (first, second) = slf.items.split_at(slf.index + 1);
        [second, first].concat()
    }
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
        let items: Vec<_> = buf.into();
        assert_eq!(vec![Some(15), Some(10), Some(5), None, None], items);
        buf.push(Some(20));
        buf.push(Some(25));
        buf.push(Some(30));
        buf.push(Some(35));
        let items: Vec<_> = buf.into();
        assert_eq!(
            vec![Some(35), Some(30), Some(25), Some(20), Some(15)],
            items
        );
    }
}

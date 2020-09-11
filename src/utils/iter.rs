use std::collections::VecDeque;
use std::iter::Fuse;

// I got inspired by the peek_nth from the iterator in itertools
pub struct PeekNth<I>
where
    I: Iterator,
    I::Item: Clone,
{
    iter: Fuse<I>,
    buf: VecDeque<I::Item>,
    pub current: Option<I::Item>,
}

pub fn peek_nth<I>(iterable: I) -> PeekNth<I::IntoIter>
where
    I: IntoIterator,
    I::Item: Clone,
{
    PeekNth {
        iter: iterable.into_iter().fuse(),
        buf: VecDeque::new(),
        current: None,
    }
}

impl<I> PeekNth<I>
where
    I: Iterator,
    I::Item: Clone,
{
    pub fn peek_nth(&mut self, n: usize) -> Option<&I::Item> {
        let items_to_buffer = (n + 1).saturating_sub(self.buf.len());
        self.buf.extend(self.iter.by_ref().take(items_to_buffer));
        self.buf.get(n)
    }

    pub fn peek(&mut self) -> Option<&I::Item> {
        self.peek_nth(0)
    }
}

impl<I> Iterator for PeekNth<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.buf.pop_front().or_else(|| self.iter.next());
        // TODO: Find out if it can be implemented without this clone
        self.current = current.clone();
        current
    }
}

#[cfg(test)]
mod test {
    use super::peek_nth as into_peek_nth;
    use super::*;

    /// Returns items sequentially
    #[test]
    fn iter() {
        let items = vec![1, 2, 3];
        let mut iter = into_peek_nth(items.clone().into_iter());
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));

        let mut iter_1 = into_peek_nth(items.clone().into_iter());
        let mut iter_2 = into_peek_nth(items.clone().into_iter());
        assert_eq!(iter_1.next(), iter_2.next());
        assert_eq!(iter_1.next(), iter_2.next());
        assert_eq!(iter_1.next(), iter_2.next());
    }

    /// Peeks the nth value of the iterator
    #[test]
    fn peek_nth() {
        let items = vec![1, 2, 3];
        let mut iter = into_peek_nth(items.clone().into_iter());

        assert_eq!(iter.peek_nth(0), Some(&1));
        assert_eq!(iter.peek_nth(1), Some(&2));
        assert_eq!(iter.peek_nth(2), Some(&3));
        assert_eq!(iter.peek_nth(4), None);
    }

    /// Returns current token
    #[test]
    fn current() {
        let items = vec![1, 2, 3];
        let mut iter = into_peek_nth(items.into_iter());
        assert_eq!(iter.current, None);
        iter.next();
        assert_eq!(iter.current, Some(1));
        iter.next();
        assert_eq!(iter.current, Some(2));
        iter.next();
        assert_eq!(iter.current, Some(3));
        iter.next();
    }
}

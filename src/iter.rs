use super::{Entry, IndexList};

impl<T> IntoIterator for IndexList<T> {
    type Item = T;
    type IntoIter = ForwardIntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let next_index = self.head;

        ForwardIntoIter {
            list: self,
            next_index,
        }
    }
}

/// Forwards iterator that consumes the list
pub struct ForwardIntoIter<T> {
    list: IndexList<T>,
    next_index: Option<usize>,
}

impl<T> Iterator for ForwardIntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let next_index = self.next_index?;

        let entry = std::mem::replace(
            &mut self.list.contents[next_index],
            Entry::Free { next_free: None },
        );

        match entry {
            Entry::Free { .. } => panic!("Corrupted list: Corrupt into iter"),
            Entry::Occupied(value) => {
                self.next_index = value.next;

                Some(value.item)
            }
        }
    }
}

/// Forwards iterator with reference to a list
/// Returns references to the elements in the list
pub struct ForwardIter<'a, T> {
    list: &'a IndexList<T>,
    next_index: Option<usize>,
}

impl<'a, T> ForwardIter<'a, T> {
    pub fn new(list: &'a IndexList<T>, next_index: Option<usize>) -> Self {
        ForwardIter { list, next_index }
    }
}

impl<'a, T> Iterator for ForwardIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let next_index = self.next_index?;

        match &self.list.contents[next_index] {
            Entry::Free { .. } => panic!("Corrupted list: Next in iterator"),
            Entry::Occupied(value) => {
                self.next_index = value.next;

                Some(&value.item)
            }
        }
    }
}

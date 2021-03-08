pub mod iter;
use iter::ForwardIter;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct IndexList<T> {
    contents: Vec<Entry<T>>,
    generation: usize,
    next_free: Option<usize>,
    head: Option<usize>,
    tail: Option<usize>,
}

#[derive(Debug)]
enum Entry<T> {
    Free { next_free: Option<usize> },
    Occupied(OccupiedEntry<T>),
}

#[derive(Debug)]
struct OccupiedEntry<T> {
    item: T,
    generation: usize,
    next: Option<usize>,
    prev: Option<usize>,
}
/// Index used to access the information in the list
/// The Generation data is used to validate that the used
/// index does corresponds to the stored entry
pub struct Index<T> {
    index: usize,
    generation: usize,
    _marker: PhantomData<T>,
}

impl<T> Index<T> {
    fn new(index: usize, generation: usize) -> Self {
        Index {
            index,
            generation,
            _marker: PhantomData,
        }
    }
}

impl<T> Default for IndexList<T> {
    fn default() -> Self {
        IndexList {
            contents: Default::default(),
            generation: Default::default(),
            next_free: Default::default(),
            head: Default::default(),
            tail: Default::default(),
        }
    }
}

impl<T> IndexList<T> {
    /// Creates an empty Index list with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// When you know how many elements will be stored in
    /// list then it makes sense to allocate that space in the vector
    pub fn new_with_capacity(size: usize) -> Self {
        IndexList {
            contents: Vec::with_capacity(size),
            generation: Default::default(),
            next_free: Default::default(),
            head: Default::default(),
            tail: Default::default(),
        }
    }

    // Extracting a reference to the head of the list
    pub fn head(&self) -> Option<&T> {
        let index = self.head?;

        self.contents.get(index).and_then(|entry| match entry {
            Entry::Free { .. } => None,
            Entry::Occupied(value) => Some(&value.item),
        })
    }

    // Extracting a mut reference to the head of the list
    pub fn head_mut(&mut self) -> Option<&mut T> {
        let index = self.head?;

        self.contents.get_mut(index).and_then(|entry| match entry {
            Entry::Free { .. } => None,
            Entry::Occupied(value) => Some(&mut value.item),
        })
    }

    // Extracting a reference to the head of the list
    pub fn tail(&self) -> Option<&T> {
        let index = self.tail?;

        self.contents.get(index).and_then(|entry| match entry {
            Entry::Free { .. } => None,
            Entry::Occupied(value) => Some(&value.item),
        })
    }

    // Extracting a mut reference to the head of the list
    pub fn tail_mut(&mut self) -> Option<&mut T> {
        let index = self.tail?;

        self.contents.get_mut(index).and_then(|entry| match entry {
            Entry::Free { .. } => None,
            Entry::Occupied(value) => Some(&mut value.item),
        })
    }

    // Get a reference to an item based on an index
    pub fn get(&self, index: &Index<T>) -> Option<&T> {
        self.contents
            .get(index.index)
            .and_then(|entry| match entry {
                Entry::Free { .. } => None,
                Entry::Occupied(value) => {
                    if value.generation == index.generation {
                        Some(&value.item)
                    } else {
                        None
                    }
                }
            })
    }

    // Get a mutable reference to an item based on an index
    pub fn get_mut(&mut self, index: &Index<T>) -> Option<&mut T> {
        self.contents
            .get_mut(index.index)
            .and_then(|entry| match entry {
                Entry::Free { .. } => None,
                Entry::Occupied(value) => {
                    if value.generation == index.generation {
                        Some(&mut value.item)
                    } else {
                        None
                    }
                }
            })
    }

    /// Push back an element to the list. This function increases
    /// the size of the list and moves the tail
    pub fn push_back(&mut self, item: T) -> Index<T> {
        // If there is no head in the list, then the entry that will
        // be appended wont have a next or previous index.
        // Otherwise, the previous index for the appended element will
        // be the actual tail
        let new_entry = match self.head {
            None => Entry::Occupied(OccupiedEntry {
                item,
                generation: self.generation,
                next: None,
                prev: None,
            }),
            Some(..) => Entry::Occupied(OccupiedEntry {
                item,
                generation: self.generation,
                next: None,
                prev: self.tail,
            }),
        };

        // Appending the new entry to the vector
        let index = self.push(new_entry);

        // Updating the next index for the item that was the tail
        if let Some(tail_index) = self.tail {
            match &mut self.contents[tail_index] {
                Entry::Free { .. } => panic!("Corrupted list: Free tail"),
                Entry::Occupied(entry) => entry.next = Some(index),
            }
        }

        // If there is no head then the head for the list becomes this new index
        if self.head.is_none() {
            self.head = Some(index);
        }

        self.tail = Some(index);

        Index::new(index, self.generation)
    }

    /// Push forward an element to the list. This function increases
    /// the size of the list and moves the head
    pub fn push_front(&mut self, item: T) -> Index<T> {
        // If there is no head in the list, then the entry that will
        // be appended wont have a next or previous index.
        // Otherwise, the previous index for the appended element will
        // be the actual tail
        let new_entry = match self.head {
            None => Entry::Occupied(OccupiedEntry {
                item,
                generation: self.generation,
                next: None,
                prev: None,
            }),
            Some(..) => Entry::Occupied(OccupiedEntry {
                item,
                generation: self.generation,
                next: self.head,
                prev: None,
            }),
        };

        // Appending the new entry to the vector
        let index = self.push(new_entry);

        // Updating the next index for the item that was the tail
        if let Some(tail_index) = self.tail {
            match &mut self.contents[tail_index] {
                Entry::Free { .. } => panic!("Corrupted list: Free tail"),
                Entry::Occupied(entry) => entry.prev = Some(index),
            }
        }

        // If there is no tail then the tail for the list becomes this new index
        if self.tail.is_none() {
            self.tail = Some(index);
        }

        self.head = Some(index);

        Index::new(index, self.generation)
    }

    /// Private function used to insert elements to the vector
    /// The insertion depends on free slots in the vector
    fn push(&mut self, new_entry: Entry<T>) -> usize {
        // The index is based on the next_free slot. If there is a next_free
        // index then we can store the element in the vector using the index
        // If there is no next slot, then the new element is added by pushing
        // an element to the vector.
        match self.next_free {
            None => {
                let index = self.contents.len();
                self.contents.push(new_entry);
                index
            }
            Some(index) => {
                match self.contents[index] {
                    Entry::Free { next_free } => self.next_free = next_free,
                    Entry::Occupied(..) => panic!("Corrupted list: Occupied next_free"),
                }
                self.contents[index] = new_entry;
                index
            }
        }
    }

    /// remove element based on key
    pub fn remove(&mut self, index: &Index<T>) -> Option<T> {
        // Check if the list has a head or tail
        let head = self.head?;
        let tail = self.tail?;

        // Check if the element exists in the vector
        let entry = self.contents.get(index.index)?;

        // And then we check if the generation of the key is equal to
        // the generation in the index. If there is no match then return None
        // Also, if the selected index is one marked as Free then None is returned
        let (prev, next) = match entry {
            Entry::Free { .. } => return None,
            Entry::Occupied(value) => {
                if value.generation != index.generation {
                    return None;
                }

                (value.prev, value.next)
            }
        };

        // Changing the next value for the previous item
        if let Some(prev_index) = prev {
            if let Some(Entry::Occupied(value)) = self.contents.get_mut(prev_index) {
                value.next = next;

                if index.index == tail {
                    self.tail = Some(prev_index);
                }
            }
        }

        // Changing the previous value for the next item
        if let Some(next_index) = next {
            if let Some(Entry::Occupied(value)) = self.contents.get_mut(next_index) {
                value.prev = prev;

                if index.index == head {
                    self.head = Some(next_index)
                }
            }
        }

        // After all the indexes have been assigned to the previous and next items
        // the item from the list is removed from the list and it is returned
        let removed = std::mem::replace(
            &mut self.contents[index.index],
            Entry::Free {
                next_free: self.next_free,
            },
        );
        self.next_free = Some(index.index);
        self.generation += 1;

        match removed {
            Entry::Occupied(value) => Some(value.item),
            Entry::Free { .. } => panic!("Corrupted list: Free value in removed"),
        }
    }

    /// Reference iterator. It doesn't consume the list and returns a reference to
    /// the elements in the list
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        ForwardIter::new(self, self.head)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        let test_list: IndexList<usize> = IndexList::new();
        assert_eq!(test_list.contents.len(), 0);
        assert_eq!(test_list.generation, 0);
        assert_eq!(test_list.next_free, None);
        assert_eq!(test_list.head, None);
        assert_eq!(test_list.tail, None);
    }

    #[test]
    fn test_with_capacity() {
        let test_list: IndexList<usize> = IndexList::new_with_capacity(10);
        assert_eq!(test_list.contents.capacity(), 10);
        assert_eq!(test_list.generation, 0);
        assert_eq!(test_list.next_free, None);
        assert_eq!(test_list.head, None);
        assert_eq!(test_list.tail, None);
    }

    #[test]
    fn test_push_back_item() {
        let mut test_list: IndexList<usize> = IndexList::new();
        assert_eq!(test_list.head(), None);
        assert_eq!(test_list.tail(), None);

        test_list.push_back(100);
        assert_eq!(test_list.head(), Some(&100));

        test_list.push_back(200);
        test_list.push_back(300);
        test_list.push_back(400);

        assert_eq!(test_list.head(), Some(&100));
        assert_eq!(test_list.tail(), Some(&400));

        assert_eq!(test_list.head_mut(), Some(&mut 100));
        assert_eq!(test_list.tail_mut(), Some(&mut 400));
    }

    #[test]
    fn test_push_forward_item() {
        let mut test_list: IndexList<usize> = IndexList::new();
        assert_eq!(test_list.head(), None);
        assert_eq!(test_list.tail(), None);

        test_list.push_front(100);
        assert_eq!(test_list.head(), Some(&100));

        test_list.push_front(200);
        test_list.push_front(300);
        test_list.push_front(400);

        assert_eq!(test_list.head(), Some(&400));
        assert_eq!(test_list.tail(), Some(&100));
    }
    #[test]
    fn test_key_values() {
        let mut test_list: IndexList<usize> = IndexList::new();

        let key_100 = test_list.push_back(100);
        let key_200 = test_list.push_back(200);
        let key_300 = test_list.push_back(300);
        let key_400 = test_list.push_back(400);

        assert_eq!(test_list.get(&key_100), Some(&100));
        assert_eq!(test_list.get(&key_200), Some(&200));
        assert_eq!(test_list.get(&key_300), Some(&300));
        assert_eq!(test_list.get(&key_400), Some(&400));

        assert_eq!(test_list.get_mut(&key_100), Some(&mut 100));
        assert_eq!(test_list.get_mut(&key_200), Some(&mut 200));
        assert_eq!(test_list.get_mut(&key_300), Some(&mut 300));
        assert_eq!(test_list.get_mut(&key_400), Some(&mut 400));
    }

    #[test]
    fn test_remove() {
        let mut test_list: IndexList<usize> = IndexList::new();

        let key_100 = test_list.push_back(100);
        test_list.push_back(200);
        let key_300 = test_list.push_back(300);
        test_list.push_back(400);
        test_list.push_back(500);

        let val_300 = test_list.remove(&key_300);

        assert_eq!(val_300, Some(300));
        assert_eq!(test_list.get(&key_300), None);
        assert_eq!(test_list.generation, 1);

        // Testing that it does use the available space
        let key_900 = test_list.push_back(900);
        assert_eq!(key_900.generation, 1);
        assert_eq!(test_list.get(&key_900), Some(&900));
        assert_eq!(test_list.tail(), Some(&900));

        // Testing that the head is reassigned
        let val_100 = test_list.remove(&key_100);
        assert_eq!(val_100, Some(100));
        assert_eq!(test_list.generation, 2);
        assert_eq!(test_list.head(), Some(&200));

        // Testing that the tail is reassigned
        test_list.remove(&key_900);
        assert_eq!(test_list.generation, 3);
        assert_eq!(test_list.tail(), Some(&500));
    }

    #[test]
    fn test_iter() {
        let mut test_list: IndexList<usize> = IndexList::new();

        test_list.push_back(100);
        test_list.push_back(200);
        let key = test_list.push_back(300);
        test_list.push_back(400);
        test_list.push_back(500);

        let mut iter = test_list.iter();
        assert_eq!(iter.next(), Some(&100));
        assert_eq!(iter.next(), Some(&200));
        assert_eq!(iter.next(), Some(&300));
        assert_eq!(iter.next(), Some(&400));
        assert_eq!(iter.next(), Some(&500));
        assert_eq!(iter.next(), None);
        drop(iter);

        test_list.remove(&key);
        let mut iter = test_list.iter();
        assert_eq!(iter.next(), Some(&100));
        assert_eq!(iter.next(), Some(&200));
        assert_eq!(iter.next(), Some(&400));
        assert_eq!(iter.next(), Some(&500));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_into_iter() {
        let mut test_list: IndexList<usize> = IndexList::new();

        test_list.push_back(100);
        test_list.push_back(200);
        test_list.push_back(300);
        test_list.push_back(400);

        let mut iter = test_list.into_iter();
        assert_eq!(iter.next(), Some(100));
        assert_eq!(iter.next(), Some(200));
        assert_eq!(iter.next(), Some(300));
        assert_eq!(iter.next(), Some(400));
        assert_eq!(iter.next(), None);
    }
}

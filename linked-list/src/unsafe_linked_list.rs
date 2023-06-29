use std::marker::PhantomData;
use std::ptr::NonNull;

/**
run tests:
MIRIFLAGS="-Zmiri-tag-raw-pointers" cargo  +nightly-2023-06-18 miri test
 */

pub struct UnsafeLinkedList<T> {
    first: Link<T>,
    last: Link<T>,
    len: usize,
    _boo: PhantomData<T>,
}

pub struct Iter<'a, T> {
    first: Link<T>,
    last: Link<T>,
    len: usize,
    _boo: PhantomData<T>,
}

type Link<T> = Option<NonNull<Node<T>>>;

struct Node<T> {
    next: Link<T>,
    prev: Link<T>,
    value: T,
}

impl<T> UnsafeLinkedList<T> {
    pub fn new() -> Self {
        Self {
            first: None,
            last: None,
            len: 0,
            _boo: PhantomData,
        }
    }

    pub fn push(&mut self, value: T) {
        unsafe {
            let new = NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                next: None,
                prev: None,
                value,
            })));
            if let Some(old) = self.first {
                (*old.as_ptr()).next = Some(new);
                (*new.as_ptr()).prev = Some(old);
            } else {
                self.last = Some(new);
            }
            self.first = Some(new);
            self.len += 1;
        }
    }

    pub fn pop_first(&mut self) -> Option<T> {
        unsafe {
            self.first.map(|node| {
                let boxed_node = Box::from_raw(node.as_ptr());
                let res = boxed_node.value;

                self.first = boxed_node.prev;
                if let Some(new) = self.first {
                    (*new.as_ptr()).next = None;
                } else {
                    debug_assert_eq!(1, self.len);
                    self.last = None;
                }

                self.len -= 1;
                res
            })
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl<T> UnsafeLinkedList<T> {
    pub fn first(&self) -> Option<&T> {
        unsafe { Some(&(*self.first?.as_ptr()).value) }
    }

    pub fn first_mut(&mut self) -> Option<&mut T> {
        unsafe {
            self.first.map(|node| &mut ((*node.as_ptr()).value))
        }
    }
}

impl<'a, T> IntoIterator for &'a UnsafeLinkedList<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            unsafe {
                self.first.map(|node| unsafe {
                    self.first = (*node.as_ptr()).next;
                    self.len -= 1;
                    &(*node.as_ptr()).value
                })
            }
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.last.map(|node| unsafe {
                self.last = (*node.as_ptr()).next;
                self.len -=1;
                &(*node.as_ptr()).value
            })
        } else {
            None
        }
    }
}

impl<T> UnsafeLinkedList<T> {
    pub fn iter(&self) -> Iter<T> {
        Iter {
            first: self.first,
            last: self.last,
            len: self.len,
            _boo: PhantomData,
        }
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<T> Drop for UnsafeLinkedList<T> {
    fn drop(&mut self) {
        while Some(_) = self.pop_first() {}
    }
}

#[cfg(test)]
mod test {
    use super::UnsafeLinkedList;

    #[test]
    fn test_basic_front() {
        let mut list = UnsafeLinkedList::new();

        // Try to break an empty list
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_first(), None);
        assert_eq!(list.len(), 0);

        // Try to break a one item list
        list.push(10);
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_first(), Some(10));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_first(), None);
        assert_eq!(list.len(), 0);

        // Mess around
        list.push(10);
        assert_eq!(list.len(), 1);
        list.push(20);
        assert_eq!(list.len(), 2);
        list.push(30);
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_first(), Some(30));
        assert_eq!(list.len(), 2);
        list.push(40);
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_first(), Some(40));
        assert_eq!(list.len(), 2);
        assert_eq!(list.pop_first(), Some(20));
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_first(), Some(10));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_first(), None);
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_first(), None);
        assert_eq!(list.len(), 0);
    }
}
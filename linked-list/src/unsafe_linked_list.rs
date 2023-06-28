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
                debug_assert!(self.first.is_none());
                debug_assert!(self.last.is_none());
                debug_assert_eq!(0, self.len);
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
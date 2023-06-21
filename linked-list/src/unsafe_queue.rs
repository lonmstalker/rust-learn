use std::ptr::null_mut;

/**
run tests:
MIRIFLAGS="-Zmiri-tag-raw-pointers" cargo  +nightly-2023-06-18 miri test
*/
pub struct UnsafeQueue<T> {
    head: Link<T>,
    tail: *mut Node<T>,
}

struct Node<T> {
    value: T,
    next: Link<T>,
}

type Link<T> = *mut Node<T>;

impl<T> UnsafeQueue<T> {
    pub fn new() -> Self {
        UnsafeQueue { head: null_mut(), tail: null_mut() }
    }

    pub fn push(&mut self, value: T) {
        let mut new_tail = Box::into_raw(Box::new(Node {
            value,
            next: null_mut(),
        }));

        if !self.tail.is_null() {
            unsafe {
                (*self.tail).next = new_tail;
            }
        } else {
            self.head = new_tail;
        }

        self.tail = new_tail;
    }

    pub fn pop(&mut self) -> Option<T> {
        unsafe {
            if self.head.is_null() {
                None
            } else {
                let head = Box::from_raw(self.head);
                self.head = head.next;

                if self.head.is_null() {
                    self.tail = null_mut()
                }

                Some(head.value)
            }
        }
    }
}

impl<T> Drop for UnsafeQueue<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop() {}
    }
}

#[cfg(test)]
mod test {
    use super::UnsafeQueue;

    #[test]
    fn basics() {
        let mut list = UnsafeQueue::new();

        // Check empty list behaves right
        assert_eq!(list.pop(), None);

        // Populate list
        list.push(1);
        list.push(2);
        list.push(3);

        // Check normal removal
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push(4);
        list.push(5);

        // Check normal removal
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), None);

        // Check the exhaustion case fixed the pointer right
        list.push(6);
        list.push(7);

        // Check normal removal
        assert_eq!(list.pop(), Some(6));
        assert_eq!(list.pop(), Some(7));
        assert_eq!(list.pop(), None);
    }
}
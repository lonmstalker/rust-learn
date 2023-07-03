use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

struct Deque<T> {
    first: Link<T>,
    last: Link<T>,
}

struct Node<T> {
    value: T,
    next: Link<T>,
    prev: Link<T>,
}

type Link<T> = Option<Rc<RefCell<Node<T>>>>;

impl<T> Deque<T> {
    pub fn new() -> Self {
        Deque {
            first: None,
            last: None,
        }
    }

    pub fn push_front(&mut self, value: T) {
        let new_first = Node::new(value);
        match self.first.take() {
            None => {
                self.last = Some(new_first.clone());
                self.first = Some(new_first);
            }
            Some(old_first) => {
                old_first.borrow_mut().prev = Some(new_first.clone());
                new_first.borrow_mut().next = Some(old_first);
                self.first = Some(new_first);
            }
        }
    }

    pub fn push_back(&mut self, value: T) {
        let new_first = Node::new(value);
        match self.last.take() {
            None => {
                self.first = Some(new_first.clone());
                self.last = Some(new_first);
            }
            Some(old_first) => {
                old_first.borrow_mut().next = Some(new_first.clone());
                new_first.borrow_mut().prev = Some(old_first);
                self.last = Some(new_first);
            }
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.first.take().map(|old_node| {
            match old_node.borrow_mut().next.take() {
                None => {
                    self.last.take();
                }
                Some(next_node) => {
                    next_node.borrow_mut().prev.take();
                    self.first = Some(next_node)
                }
            }
            Rc::try_unwrap(old_node).ok().unwrap().into_inner().value
        })
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.last.take().map(|old_node| {
            match old_node.borrow_mut().prev.take() {
                None => {
                    self.first.take();
                }
                Some(next_node) => {
                    next_node.borrow_mut().next.take();
                    self.last = Some(next_node)
                }
            }
            Rc::try_unwrap(old_node).ok().unwrap().into_inner().value
        })
    }

    pub fn peek_front(&self) -> Option<Ref<T>> {
        self.first
            .as_ref()
            .map(|node| Ref::map(node.borrow(), |rf| &rf.value))
    }

    pub fn peek_front_mut(&self) -> Option<RefMut<T>> {
        self.first
            .as_ref()
            .map(|node| RefMut::map(node.borrow_mut(), |rf| &mut rf.value))
    }

    pub fn peek_back(&self) -> Option<Ref<T>> {
        self.last
            .as_ref()
            .map(|node| Ref::map(node.borrow(), |rf| &rf.value))
    }

    pub fn peek_back_mut(&self) -> Option<RefMut<T>> {
        self.last
            .as_ref()
            .map(|node| RefMut::map(node.borrow_mut(), |rf| &mut rf.value))
    }
}

impl<T> Node<T> {
    pub fn new(value: T) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            value,
            next: None,
            prev: None,
        }))
    }
}

impl<T> Drop for Deque<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

#[cfg(test)]
mod test {
    use crate::safe_deque::Deque;

    #[test]
    fn basics() {
        let mut list = Deque::new();

        // Check empty list behaves right
        assert_eq!(list.pop_front(), None);

        // Populate list
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        // Check normal removal
        assert_eq!(list.pop_front(), Some(3));
        assert_eq!(list.pop_front(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push_front(4);
        list.push_front(5);

        // Check normal removal
        assert_eq!(list.pop_front(), Some(5));
        assert_eq!(list.pop_front(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), None);

        // ---- back -----

        // Check empty list behaves right
        assert_eq!(list.pop_back(), None);

        // Populate list
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        // Check normal removal
        assert_eq!(list.pop_back(), Some(3));
        assert_eq!(list.pop_back(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push_back(4);
        list.push_back(5);

        // Check normal removal
        assert_eq!(list.pop_back(), Some(5));
        assert_eq!(list.pop_back(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop_back(), Some(1));
        assert_eq!(list.pop_back(), None);
    }

    #[test]
    fn peek() {
        let mut list = Deque::new();
        assert!(list.peek_front().is_none());
        assert!(list.peek_back().is_none());
        assert!(list.peek_front_mut().is_none());
        assert!(list.peek_back_mut().is_none());

        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        assert_eq!(&*list.peek_front().unwrap(), &3);
        assert_eq!(&mut *list.peek_front_mut().unwrap(), &mut 3);
        assert_eq!(&*list.peek_back().unwrap(), &1);
        assert_eq!(&mut *list.peek_back_mut().unwrap(), &mut 1);
    }
}

use std::cell::RefCell;
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
        Deque { first: None, last: None }
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

    pub fn pop_front(&mut self) -> Option<T> {
        self.first.take().map(|old_node| {
            match old_node.borrow_mut().next.take() {
                None => { self.last.take(); }
                Some(next_node) => {
                    next_node.borrow_mut().prev.take();
                    self.first = Some(next_node)
                }
            }
            Rc::try_unwrap(old_node).ok().unwrap().into_inner().value
        })
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
    }
}
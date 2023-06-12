
// https://rust-unofficial.github.io/too-many-lists/index.html
pub struct LinkedList<T> {
    head: Link<T>,
}

struct LinkedNode<T> {
    value: T,
    next: Link<T>,
}

type Link<T> = Option<Box<LinkedNode<T>>>;

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        LinkedList { head: None }
    }

    pub fn push(&mut self, value: T) {
        let node = LinkedNode {
            value,
            next: self.head.take(),
        };
        self.head = Some(Box::new(node))
    }

    pub fn pop(&mut self) -> Option<T> {
        self.head.take()
            .map(|node| {
                self.head = node.next;
                node.value
            })
    }

    pub fn peek(&self) -> Option<&T>{
        self.head.as_ref()
            .map(|node| &node.value)
    }

    pub fn peek_mut(&mut self) -> Option<&mut T>{
        self.head.as_mut()
            .map(|node| &mut node.value)
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        let mut cur_link = self.head.take();
        while let Some(_) = cur_link {
            cur_link = self.head.take();
        }
    }
}

#[cfg(test)]
mod test {
    use crate::linked_list::LinkedList;

    #[test]
    fn basics() {
        let mut list = LinkedList::new();

        // Check empty list behaves right
        assert_eq!(list.pop(), None);

        // Populate list
        list.push(1);
        list.push(2);
        list.push(3);

        // Check normal removal
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push(4);
        list.push(5);

        // Check normal removal
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), None);
    }

    #[test]
    fn peek() {
        let mut list = LinkedList::new();
        assert_eq!(list.peek(), None);
        assert_eq!(list.peek_mut(), None);
        list.push(1); list.push(2); list.push(3);

        assert_eq!(list.peek(), Some(&3));
        assert_eq!(list.peek_mut(), Some(&mut 3));
    }
}
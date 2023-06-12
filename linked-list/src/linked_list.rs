pub mod collection {
    use std::mem;

    struct LinkedList<T> {
        head: LinkedNode<T>,
    }

    enum LinkedNode<T> {
        Empty,
        Node { value: T, next: Box<LinkedNode<T>> },
    }

    impl<T> LinkedList<T> {
        pub fn new() -> Self {
            LinkedList { head: LinkedNode::Empty }
        }

        pub fn push(&mut self, value: T) {
            let node = Box::new(
                LinkedNode::Node { value, next: LinkedNode::Empty }
            );
            self.head = node
        }
    }
}
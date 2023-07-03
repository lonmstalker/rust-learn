use std::rc::Rc;

pub struct ImmutableList<T> {
    head: Option<Rc<Node<T>>>,
}

struct Node<T> {
    value: T,
    next: Option<Rc<Node<T>>>,
}

struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<T> ImmutableList<T> {
    pub fn new() -> Self {
        ImmutableList { head: None }
    }
}

impl<T> ImmutableList<T> {
    pub fn prepend(&self, value: T) -> Self {
        let node = Node {
            value,
            next: self.head.clone(),
        };
        ImmutableList {
            head: Some(Rc::new(node)),
        }
    }

    pub fn drop_last(&self) -> Self {
        ImmutableList {
            head: self.head.as_ref().and_then(|node| node.next.clone()),
        }
    }

    pub fn first(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.value)
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            next: self.head.as_deref(),
        }
    }
}

impl<T> Drop for ImmutableList<T> {
    fn drop(&mut self) {
        let mut head = self.head.take();
        while let Some(node) = head {
            if let Ok(mut node) = Rc::try_unwrap(node) {
                head = node.next.take();
            } else {
                break;
            }
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        self.next.map(|node| {
            self.next = node.next.as_deref();
            &node.value
        })
    }
}

#[cfg(test)]
mod test {
    use super::ImmutableList;

    #[test]
    fn basics() {
        let list = ImmutableList::new();
        assert_eq!(list.first(), None);

        let list = list.prepend(1).prepend(2).prepend(3);
        assert_eq!(list.first(), Some(&3));

        let list = list.drop_last();
        assert_eq!(list.first(), Some(&2));

        let list = list.drop_last();
        assert_eq!(list.first(), Some(&1));

        let list = list.drop_last();
        assert_eq!(list.first(), None);

        // Make sure empty tail works
        let list = list.drop_last();
        assert_eq!(list.first(), None);
    }

    #[test]
    fn iter() {
        let list = ImmutableList::new().prepend(1).prepend(2).prepend(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
    }
}

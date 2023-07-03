use std::cmp::Ordering;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::{fmt, mem};

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
    _boo: PhantomData<&'a T>,
}

pub struct IterMut<'a, T> {
    first: Link<T>,
    last: Link<T>,
    len: usize,
    _boo: PhantomData<&'a mut T>,
}

pub struct IntoIter<T> {
    list: UnsafeLinkedList<T>,
}

pub struct CursorMut<'a, T> {
    cur: Link<T>,
    list: &'a mut UnsafeLinkedList<T>,
    index: Option<usize>,
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
    pub fn cursor_mut(&mut self) -> CursorMut<T> {
        CursorMut {
            list: self,
            cur: None,
            index: None,
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

    pub fn push_back(&mut self, value: T) {
        unsafe {
            let new = NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                next: None,
                prev: None,
                value,
            })));
            if let Some(old) = self.last {
                (*old.as_ptr()).prev = Some(new);
                (*new.as_ptr()).next = Some(old);
            } else {
                self.first = Some(new);
            }
            self.last = Some(new);
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
                    self.last = None;
                }

                self.len -= 1;
                res
            })
        }
    }

    pub fn pop_back(&mut self) -> Option<T> {
        unsafe {
            self.last.map(|node| {
                let boxed_node = Box::from_raw(node.as_ptr());
                let res = boxed_node.value;

                self.last = boxed_node.next;
                if let Some(new) = self.last {
                    (*new.as_ptr()).prev = None;
                } else {
                    self.first = None;
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
        unsafe { self.first.map(|node| &mut ((*node.as_ptr()).value)) }
    }

    pub fn back(&self) -> Option<&T> {
        unsafe { self.last.map(|node| &(*node.as_ptr()).value) }
    }

    pub fn back_mut(&mut self) -> Option<&mut T> {
        unsafe { self.last.map(|node| &mut (*node.as_ptr()).value) }
    }
}

impl<'a, T> IntoIterator for &'a UnsafeLinkedList<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> IntoIterator for UnsafeLinkedList<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a mut UnsafeLinkedList<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            unsafe {
                self.first.map(|node| unsafe {
                    self.len -= 1;
                    self.first = (*node.as_ptr()).prev;
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

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop_first()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.list.len, Some(self.list.len))
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.first.map(|node| unsafe {
                self.len -= 1;
                self.first = (*node.as_ptr()).prev;
                &mut (*node.as_ptr()).value
            })
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.list.pop_back()
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.last.map(|node| unsafe {
                self.len -= 1;
                self.last = (*node.as_ptr()).next;
                &(*node.as_ptr()).value
            })
        } else {
            None
        }
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.last.map(|node| unsafe {
                self.len -= 1;
                self.last = (*node.as_ptr()).next;
                &mut (*node.as_ptr()).value
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

    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter { list: self }
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            first: self.first,
            last: self.last,
            len: self.len,
            _boo: PhantomData,
        }
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.list.len
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<T> UnsafeLinkedList<T> {
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        while self.pop_first().is_some() {}
    }
}

impl<T> Drop for UnsafeLinkedList<T> {
    fn drop(&mut self) {
        self.clear()
    }
}

impl<T> Default for UnsafeLinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> Clone for UnsafeLinkedList<T> {
    fn clone(&self) -> Self {
        let mut new_list = Self::new();
        for elem in self {
            new_list.push_back(elem.clone());
        }
        new_list
    }
}

impl<T> Extend<T> for UnsafeLinkedList<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push_back(item);
        }
    }
}

impl<T> FromIterator<T> for UnsafeLinkedList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = Self::new();
        list.extend(iter);
        list
    }
}

impl<T: Debug> Debug for UnsafeLinkedList<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

impl<T: PartialEq> PartialEq for UnsafeLinkedList<T> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other)
    }

    fn ne(&self, other: &Self) -> bool {
        self.len() != other.len() || self.iter().ne(other)
    }
}

impl<T: Eq> Eq for UnsafeLinkedList<T> {}

impl<T: PartialOrd> PartialOrd for UnsafeLinkedList<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl<T: Ord> Ord for UnsafeLinkedList<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.iter().cmp(other)
    }
}

impl<T: Hash> Hash for UnsafeLinkedList<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.len().hash(state);
        for item in self {
            item.hash(state);
        }
    }
}

impl<'a, T> CursorMut<'a, T> {
    pub fn index(&self) -> Option<usize> {
        self.index
    }

    pub fn move_next(&mut self) {
        if let Some(cur) = self.cur {
            unsafe {
                self.cur = (*cur.as_ptr()).prev;
                if self.cur.is_some() {
                    *self.index.as_mut().unwrap() += 1;
                } else {
                    self.index = None;
                }
            }
        } else if !self.list.is_empty() {
            self.cur = self.list.first;
            self.index = Some(0);
        }
    }

    pub fn move_back(&mut self) {
        if let Some(cur) = self.cur {
            unsafe {
                self.cur = (*cur.as_ptr()).next;
                if self.cur.is_some() {
                    *self.index.as_mut().unwrap() -= 1;
                } else {
                    self.index = None;
                }
            }
        } else if !self.list.is_empty() {
            self.cur = self.list.last;
            self.index = Some(self.list.len - 1);
        }
    }

    pub fn current(&mut self) -> Option<&mut T> {
        unsafe { self.cur.map(|node| &mut (*node.as_ptr()).value) }
    }

    pub fn peek_next(&mut self) -> Option<&mut T> {
        unsafe {
            let next = if let Some(cur) = self.cur {
                (*cur.as_ptr()).prev
            } else {
                self.list.first
            };

            next.map(|node| &mut (*node.as_ptr()).value)
        }
    }

    pub fn peek_back(&mut self) -> Option<&mut T> {
        unsafe {
            let next = if let Some(cur) = self.cur {
                (*cur.as_ptr()).next
            } else {
                self.list.last
            };

            next.map(|node| &mut (*node.as_ptr()).value)
        }
    }

    pub fn split_before(&mut self) -> UnsafeLinkedList<T> {
        if let Some(cur) = self.cur {
            unsafe {
                let old_len = self.list.len;
                let old_idx = self.index.unwrap();
                let prev = (*cur.as_ptr()).next.take();
                let new_len = old_len - old_idx;

                if let Some(prev) = prev {
                    (*prev.as_ptr()).prev = None;
                }

                self.index = Some(0);
                self.list.len = new_len;
                self.list.first = self.cur;

                UnsafeLinkedList {
                    first: self.list.first,
                    last: prev,
                    len: old_len - new_len,
                    _boo: PhantomData,
                }
            }
        } else {
            mem::replace(self.list, UnsafeLinkedList::new())
        }
    }

    pub fn split_after(&mut self) -> UnsafeLinkedList<T> {
        if let Some(cur) = self.cur {
            unsafe {
                let old_len = self.list.len;
                let old_idx = self.index.unwrap();
                let prev = (*cur.as_ptr()).prev.take();
                let new_len = old_len - old_idx;

                if let Some(prev) = prev {
                    (*prev.as_ptr()).next = None;
                }

                self.index = Some(0);
                self.list.len = new_len;
                self.list.last = self.cur;

                UnsafeLinkedList {
                    first: self.list.last,
                    last: prev,
                    len: old_len - new_len,
                    _boo: PhantomData,
                }
            }
        } else {
            mem::replace(self.list, UnsafeLinkedList::new())
        }
    }

    pub fn splice_before(&mut self, mut input: UnsafeLinkedList<T>) {
        unsafe {
            if input.is_empty() {
                return;
            }
            if let Some(cur) = self.cur {
                let in_front = input.first.take().unwrap();
                let in_back = input.last.take().unwrap();

                if let Some(prev) = (*cur.as_ptr()).next {
                    (*prev.as_ptr()).prev = Some(in_front);
                    (*in_front.as_ptr()).next = Some(prev);
                    (*cur.as_ptr()).next = Some(in_back);
                    (*in_back.as_ptr()).prev = Some(cur);
                } else {
                    (*cur.as_ptr()).next = Some(in_back);
                    (*in_back.as_ptr()).prev = Some(cur);
                    self.list.first = Some(in_front);
                }
                *self.index.as_mut().unwrap() += input.len;
            } else if let Some(back) = self.list.last {
                let in_front = input.first.take().unwrap();
                let in_back = input.last.take().unwrap();

                (*back.as_ptr()).prev = Some(in_front);
                (*in_front.as_ptr()).next = Some(back);
                self.list.last = Some(in_back);
            } else {
                mem::swap(self.list, &mut input);
            }

            self.list.len += input.len;
            input.len = 0;
        }
    }

    pub fn splice_after(&mut self, mut input: UnsafeLinkedList<T>) {
        unsafe {
            if input.is_empty() {
                return;
            }
            if let Some(cur) = self.cur {
                let in_front = input.first.take().unwrap();
                let in_back = input.last.take().unwrap();

                if let Some(next) = (*cur.as_ptr()).prev {
                    (*next.as_ptr()).next = Some(in_back);
                    (*in_back.as_ptr()).prev = Some(next);
                    (*cur.as_ptr()).prev = Some(in_front);
                    (*in_front.as_ptr()).next = Some(cur);
                } else {
                    (*cur.as_ptr()).prev = Some(in_front);
                    (*in_front.as_ptr()).next = Some(cur);
                    self.list.last = Some(in_back);
                }
            } else if let Some(front) = self.list.first {
                let in_front = input.first.take().unwrap();
                let in_back = input.last.take().unwrap();

                (*front.as_ptr()).next = Some(in_back);
                (*in_back.as_ptr()).prev = Some(front);
                self.list.first = Some(in_front);
            } else {
                mem::swap(self.list, &mut input);
            }

            self.list.len += input.len;
            input.len = 0;
        }
    }
}

#[cfg(test)]
mod test {
    use super::UnsafeLinkedList;

    fn generate_test() -> UnsafeLinkedList<i32> {
        list_from(&[0, 1, 2, 3, 4, 5, 6])
    }

    fn list_from<T: Clone>(data: &[T]) -> UnsafeLinkedList<T> {
        data.iter().map(|v| (*v).clone()).collect()
    }

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

    #[test]
    fn test_basic() {
        let mut m = UnsafeLinkedList::new();
        assert_eq!(m.pop_first(), None);
        assert_eq!(m.pop_back(), None);
        assert_eq!(m.pop_first(), None);
        m.push(1);
        assert_eq!(m.pop_first(), Some(1));
        m.push_back(2);
        m.push_back(3);
        assert_eq!(m.len(), 2);
        assert_eq!(m.pop_first(), Some(2));
        assert_eq!(m.pop_first(), Some(3));
        assert_eq!(m.len(), 0);
        assert_eq!(m.pop_first(), None);
        m.push_back(1);
        m.push_back(3);
        m.push_back(5);
        m.push_back(7);
        assert_eq!(m.pop_first(), Some(1));

        let mut n = UnsafeLinkedList::new();
        n.push(2);
        n.push(3);
        {
            assert_eq!(n.first().unwrap(), &3);
            let x = n.first_mut().unwrap();
            assert_eq!(*x, 3);
            *x = 0;
        }
        {
            assert_eq!(n.back().unwrap(), &2);
            let y = n.back_mut().unwrap();
            assert_eq!(*y, 2);
            *y = 1;
        }
        assert_eq!(n.pop_first(), Some(0));
        assert_eq!(n.pop_first(), Some(1));
    }

    #[test]
    fn test_iterator() {
        let m = generate_test();
        for (i, elt) in m.iter().enumerate() {
            assert_eq!(i as i32, *elt);
        }
        let mut n = UnsafeLinkedList::new();
        assert_eq!(n.iter().next(), None);
        n.push(4);
        let mut it = n.iter();
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(it.next().unwrap(), &4);
        assert_eq!(it.size_hint(), (0, Some(0)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_iterator_double_end() {
        let mut n = UnsafeLinkedList::new();
        assert_eq!(n.iter().next(), None);
        n.push(4);
        n.push(5);
        n.push(6);
        let mut it = n.iter();
        assert_eq!(it.size_hint(), (3, Some(3)));
        assert_eq!(it.next().unwrap(), &6);
        assert_eq!(it.size_hint(), (2, Some(2)));
        assert_eq!(it.next_back().unwrap(), &4);
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(it.next_back().unwrap(), &5);
        assert_eq!(it.next_back(), None);
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_rev_iter() {
        let m = generate_test();
        for (i, elt) in m.iter().rev().enumerate() {
            assert_eq!(6 - i as i32, *elt);
        }
        let mut n = UnsafeLinkedList::new();
        assert_eq!(n.iter().rev().next(), None);
        n.push(4);
        let mut it = n.iter().rev();
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(it.next().unwrap(), &4);
        assert_eq!(it.size_hint(), (0, Some(0)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_mut_iter() {
        let mut m = generate_test();
        let mut len = m.len();
        for (i, elt) in m.iter_mut().enumerate() {
            assert_eq!(i as i32, *elt);
            len -= 1;
        }
        assert_eq!(len, 0);
        let mut n = UnsafeLinkedList::new();
        assert!(n.iter_mut().next().is_none());
        n.push(4);
        n.push_back(5);
        let mut it = n.iter_mut();
        assert_eq!(it.size_hint(), (2, Some(2)));
        assert!(it.next().is_some());
        assert!(it.next().is_some());
        assert_eq!(it.size_hint(), (0, Some(0)));
        assert!(it.next().is_none());
    }

    #[test]
    fn test_iterator_mut_double_end() {
        let mut n = UnsafeLinkedList::new();
        assert!(n.iter_mut().next_back().is_none());
        n.push(4);
        n.push(5);
        n.push(6);
        let mut it = n.iter_mut();
        assert_eq!(it.size_hint(), (3, Some(3)));
        assert_eq!(*it.next().unwrap(), 6);
        assert_eq!(it.size_hint(), (2, Some(2)));
        assert_eq!(*it.next_back().unwrap(), 4);
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(*it.next_back().unwrap(), 5);
        assert!(it.next_back().is_none());
        assert!(it.next().is_none());
    }

    #[test]
    fn test_eq() {
        let mut n: UnsafeLinkedList<u8> = list_from(&[]);
        let mut m = list_from(&[]);
        assert_eq!(n, m);
        n.push(1);
        assert_ne!(n, m);
        m.push_back(1);
        assert_eq!(n, m);

        let n = list_from(&[2, 3, 4]);
        let m = list_from(&[1, 2, 3]);
        assert_ne!(n, m);
    }

    #[test]
    fn test_ord() {
        let n = list_from(&[]);
        let m = list_from(&[1, 2, 3]);
        assert!(n < m);
        assert!(m > n);
        assert!(n <= n);
        assert!(n >= n);
    }

    #[test]
    fn test_ord_nan() {
        let nan = 0.0f64 / 0.0;
        let n = list_from(&[nan]);
        let m = list_from(&[nan]);
        assert!(!(n < m));
        assert!(!(n > m));
        assert!(!(n <= m));
        assert!(!(n >= m));

        let n = list_from(&[nan]);
        let one = list_from(&[1.0f64]);
        assert!(!(n < one));
        assert!(!(n > one));
        assert!(!(n <= one));
        assert!(!(n >= one));

        let u = list_from(&[1.0f64, 2.0, nan]);
        let v = list_from(&[1.0f64, 2.0, 3.0]);
        assert!(!(u < v));
        assert!(!(u > v));
        assert!(!(u <= v));
        assert!(!(u >= v));

        let s = list_from(&[1.0f64, 2.0, 4.0, 2.0]);
        let t = list_from(&[1.0f64, 2.0, 3.0, 2.0]);
        assert!(!(s < t));
        assert!(s > one);
        assert!(!(s <= one));
        assert!(s >= one);
    }

    #[test]
    fn test_debug() {
        let list: UnsafeLinkedList<i32> = (0..10).collect();
        assert_eq!(format!("{:?}", list), "[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]");

        let list: UnsafeLinkedList<&str> = vec!["just", "one", "test", "more"]
            .iter()
            .copied()
            .collect();
        assert_eq!(format!("{:?}", list), r#"["just", "one", "test", "more"]"#);
    }

    #[test]
    fn test_hashmap() {
        // Check that HashMap works with this as a key

        let list1: UnsafeLinkedList<i32> = (0..10).collect();
        let list2: UnsafeLinkedList<i32> = (1..11).collect();
        let mut map = std::collections::HashMap::new();

        assert_eq!(map.insert(list1.clone(), "list1"), None);
        assert_eq!(map.insert(list2.clone(), "list2"), None);

        assert_eq!(map.len(), 2);

        assert_eq!(map.get(&list1), Some(&"list1"));
        assert_eq!(map.get(&list2), Some(&"list2"));

        assert_eq!(map.remove(&list1), Some("list1"));
        assert_eq!(map.remove(&list2), Some("list2"));

        assert!(map.is_empty());
    }

    #[test]
    fn test_cursor_move_peek() {
        let mut m: UnsafeLinkedList<u32> = UnsafeLinkedList::new();
        m.extend([1, 2, 3, 4, 5, 6]);
        let mut cursor = m.cursor_mut();
        cursor.move_next();
        assert_eq!(cursor.current(), Some(&mut 1));
        assert_eq!(cursor.peek_next(), Some(&mut 2));
        assert_eq!(cursor.peek_back(), None);
        assert_eq!(cursor.index(), Some(0));
        cursor.move_back();
        assert_eq!(cursor.current(), None);
        assert_eq!(cursor.peek_next(), Some(&mut 1));
        assert_eq!(cursor.peek_back(), Some(&mut 6));
        assert_eq!(cursor.index(), None);
        cursor.move_next();
        cursor.move_next();
        assert_eq!(cursor.current(), Some(&mut 2));
        assert_eq!(cursor.peek_next(), Some(&mut 3));
        assert_eq!(cursor.peek_back(), Some(&mut 1));
        assert_eq!(cursor.index(), Some(1));

        let mut cursor = m.cursor_mut();
        cursor.move_back();
        assert_eq!(cursor.current(), Some(&mut 6));
        assert_eq!(cursor.peek_next(), None);
        assert_eq!(cursor.peek_back(), Some(&mut 5));
        assert_eq!(cursor.index(), Some(5));
        cursor.move_next();
        assert_eq!(cursor.current(), None);
        assert_eq!(cursor.peek_next(), Some(&mut 1));
        assert_eq!(cursor.peek_back(), Some(&mut 6));
        assert_eq!(cursor.index(), None);
        cursor.move_back();
        cursor.move_back();
        assert_eq!(cursor.current(), Some(&mut 5));
        assert_eq!(cursor.peek_next(), Some(&mut 6));
        assert_eq!(cursor.peek_back(), Some(&mut 4));
        assert_eq!(cursor.index(), Some(4));
    }

    #[test]
    fn test_cursor_mut_insert() {
        let mut m: UnsafeLinkedList<u32> = UnsafeLinkedList::new();
        m.extend([1, 2, 3, 4, 5, 6]);
        let mut cursor = m.cursor_mut();
        cursor.move_next();
        cursor.splice_before(Some(7).into_iter().collect());
        cursor.splice_after(Some(8).into_iter().collect());
        // check_links(&m);
        assert_eq!(
            m.iter().cloned().collect::<Vec<_>>(),
            &[7, 1, 8, 2, 3, 4, 5, 6]
        );
        let mut cursor = m.cursor_mut();
        cursor.move_next();
        cursor.move_back();
        cursor.splice_before(Some(9).into_iter().collect());
        cursor.splice_after(Some(10).into_iter().collect());
        check_links(&m);
        assert_eq!(
            m.iter().cloned().collect::<Vec<_>>(),
            &[10, 7, 1, 8, 2, 3, 4, 5, 6, 9]
        );

        /* remove_current not impl'd
        let mut cursor = m.cursor_mut();
        cursor.move_next();
        cursor.move_prev();
        assert_eq!(cursor.remove_current(), None);
        cursor.move_next();
        cursor.move_next();
        assert_eq!(cursor.remove_current(), Some(7));
        cursor.move_prev();
        cursor.move_prev();
        cursor.move_prev();
        assert_eq!(cursor.remove_current(), Some(9));
        cursor.move_next();
        assert_eq!(cursor.remove_current(), Some(10));
        check_links(&m);
        assert_eq!(m.iter().cloned().collect::<Vec<_>>(), &[1, 8, 2, 3, 4, 5, 6]);
        */

        let mut a: UnsafeLinkedList<u32> = UnsafeLinkedList::new();
        a.extend([1, 8, 2, 3, 4, 5, 6]);
        let mut cursor = a.cursor_mut();
        cursor.move_next();
        let mut p: UnsafeLinkedList<u32> = UnsafeLinkedList::new();
        p.extend([100, 101, 102, 103]);
        let mut q: UnsafeLinkedList<u32> = UnsafeLinkedList::new();
        q.extend([200, 201, 202, 203]);
        cursor.splice_after(p);
        cursor.splice_before(q);
        check_links(&m);
        assert_eq!(
            a.iter().cloned().collect::<Vec<_>>(),
            &[200, 201, 202, 203, 1, 100, 101, 102, 103, 8, 2, 3, 4, 5, 6]
        );
        let mut cursor = a.cursor_mut();
        cursor.move_next();
        cursor.move_back();
        let tmp = cursor.split_before();
        assert_eq!(a.clon.into_iter().collect::<Vec<_>>(), &[]);
        m = tmp;
        let mut cursor = a.cursor_mut();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        let tmp = cursor.split_after();
        assert_eq!(
            tmp.into_iter().collect::<Vec<_>>(),
            &[102, 103, 8, 2, 3, 4, 5, 6]
        );
        check_links(&m);
        assert_eq!(
            a.iter().cloned().collect::<Vec<_>>(),
            &[200, 201, 202, 203, 1, 100, 101]
        );
    }

    fn check_links<T: Eq + std::fmt::Debug>(list: &UnsafeLinkedList<T>) {
        let from_front: Vec<_> = list.iter().collect();
        let from_back: Vec<_> = list.iter().rev().collect();
        let re_reved: Vec<_> = from_back.into_iter().rev().collect();

        assert_eq!(from_front, re_reved);
    }
}

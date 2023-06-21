use crate::linked_list::LinkedList;

mod linked_list;
mod safe_deque;
mod immutable_linked_list;
mod unsafe_queue;

fn main() {
    let mut list = LinkedList::new();
    list.push(12);
    print!("{:?}", list.pop());
}

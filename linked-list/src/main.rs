use crate::linked_list::LinkedList;
use crate::immutable_linked_list::ImmutableList;

mod linked_list;
mod immutable_linked_list;

fn main() {
    let mut list = LinkedList::new();
    list.push(12);
    print!("{:?}", list.pop());
}

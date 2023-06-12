use crate::linked_list::LinkedList;

mod linked_list;

fn main() {
    let mut list = LinkedList::new();
    list.push(12);
    print!("{:?}", list.pop());
}
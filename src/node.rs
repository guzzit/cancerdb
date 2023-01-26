use crate::{dal::Dal, freelist::PageNumber};


struct Node {
    dal: Dal,
    items: Vec<Item>,
    child_nodes: Vec<PageNumber>,
}

impl Node {
    fn new(dal: Dal, items: Vec<Item>) -> Self {
        Node {
            dal,
            items,
            child_nodes: Vec::new(),
        }
    }

    fn is_leaf(&self) -> bool {
        self.child_nodes.is_empty()
    }
}

struct Item {
    key: Box<[u8]>,
    value: Box<[u8]>,
}

impl Item {
    fn new(key: Box<[u8]>, value: Box<[u8]>) -> Self {
        Item{
            key,
            value,
        }
    }
}
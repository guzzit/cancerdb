use std::{io::{self, ErrorKind}, cmp::Ordering};

use crate::{freelist::PageNumber, dal::Dal, node::{Item, Node}};


struct Collection {
    name: Box<[u8]>,
    root: Option<PageNumber>,
}

impl Collection {
    fn new(name: Box<[u8]>, root: PageNumber) -> Self {
        Collection { 
            name, 
            root:None, 
        }
    }

    fn find(&self, dal: &mut Dal, key: Box<[u8]>) -> Result<Option<Item>, io::Error> {
        let mut node = dal.get_node(self.root.ok_or_else(|| ErrorKind::InvalidData)?)?;

        let key = node.find_key(&key, dal)?;

        Ok(key)
    }

    fn put(&mut self, dal: &mut Dal, key: Box<[u8]>, value: Box<[u8]>) -> Result<(), io::Error> {
        let item = Item::new(key, value);

        let mut root:Node = match self.root {
            None => {
                let mut node = dal.new_node()?;
                node.items.push(item);
                dal.write_node(&node)?;
                self.root = Some(node.get_page_number());
                return Ok(());
            }
        Some(page_number) => {
            let root = dal.get_node(page_number)?;
            root
        },
        };

        let ancestor_nodes = Vec::new();
        let (node_to_insert, insertion_index) = root.find_node(&key, dal, &ancestor_nodes)?.ok_or_else(|| ErrorKind::Other)?;
        //let a = root.find_key(&key, dal);
        if !node_to_insert.is_leaf() && node_to_insert.items.get(insertion_index).is_some_and(|i| i.get_key().cmp(&key) == Ordering::Equal) == true {
            node_to_insert.items[insertion_index] = item;
        }
        else {
            node_to_insert.add_item(insertion_index, item);
        }

        dal.write_node(&node_to_insert)?;

        Ok(())
    }
}
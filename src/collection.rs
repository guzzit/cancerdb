use std::{io::{self, ErrorKind}, cmp::Ordering};

use crate::{freelist::PageNumber, dal::Dal, node::{Item, Node}};


pub struct Collection {
    name: Box<[u8]>,
    root: Option<PageNumber>,
}

impl Collection {
    pub fn new(name: Box<[u8]>) -> Self {
        Collection { 
            name, 
            root:None, 
        }
    }

    pub fn find(&self, dal: &mut Dal, key: Box<[u8]>) -> Result<Option<Item>, io::Error> {
        let mut node = dal.get_node(self.root.ok_or_else(|| ErrorKind::InvalidData)?)?;

        let key = node.find_key(&key, dal)?;

        Ok(key)
    }

    pub fn put(&mut self, dal: &mut Dal, key: Box<[u8]>, value: Box<[u8]>) -> Result<(), io::Error> {
        let item = Item::new(key.clone(), value);

        let mut root:Node = match self.root {
            None => {
                let mut node = dal.new_node()?;
                node.items.push(item);
                //need to store collection page number as well
                dal.write_node(&node)?;
                self.root = Some(node.get_page_number());
                return Ok(());
            }
        Some(page_number) => {
            let root = dal.get_node(page_number)?;
            root
        },
        };

        let mut ancestor_page_numbers = Vec::new();
        let (mut node_to_insert, insertion_index) = root.find_node(&key, dal, &mut ancestor_page_numbers)?.ok_or_else(|| ErrorKind::Other)?;
        //let a = root.find_key(&key, dal);
        //if !node_to_insert.is_leaf() && node_to_insert.items.get(insertion_index).is_some_and(|i| i.get_key().cmp(&key) == Ordering::Equal) == true {
        if  node_to_insert.items.get(insertion_index).is_some_and(|i| i.get_key().cmp(&key) == Ordering::Equal) == true {
            node_to_insert.items[insertion_index] = item;
        }
        else {
            node_to_insert.add_item(insertion_index, item)?;
        }

        dal.write_node(&node_to_insert)?;

        let ancestor_nodes = dal.get_nodes(ancestor_page_numbers)?;

        let mut iterator = ancestor_nodes.iter().rev().enumerate();
        iterator.next();

        for (index, ancestor) in iterator {
            let child_node_index = index + 1;
            if ancestor_nodes.get(child_node_index).is_none() {
                break;
            }
            let mut parent_node = ancestor.clone();
            
            let mut child_node = ancestor_nodes.get(child_node_index).ok_or_else(|| ErrorKind::Other)?.clone();

            if child_node.is_overpopulated(dal)? {
                parent_node.split_child(&mut child_node, child_node_index, dal)?;
            }
        }

        let mut root_node = ancestor_nodes.first().ok_or_else(|| ErrorKind::Other)?.clone();

        if root_node.is_overpopulated(dal)? {
            let mut new_root = Node::build(root_node.get_page_number())?;
            

            root_node.set_page_number(dal);
            new_root.child_nodes.push(root_node.get_page_number());

            new_root.split_child(&mut root_node, 0, dal)?;
            self.root = Some(new_root.get_page_number());

            //dal.write_node(&new_root);

        }

        Ok(())
    }
}
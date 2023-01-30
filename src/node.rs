use std::{io::{self, ErrorKind}, collections::HashMap};

use crate::{dal::Dal, freelist::PageNumber, constants::{BYTES_IN_U16, BYTES_IN_U64}};


struct Node {
    dal: Dal,
    items: Vec<Item>,
    //items: HashMap<Box<[u8]>, ItemValue>,
    page_number: PageNumber,
    child_nodes: Vec<PageNumber>,
}

type ItemValue = (Box<[u8]>, Option<PageNumber>);

impl Node {
    fn new(dal: Dal, page_number: PageNumber) -> Self {
        Node {
            dal,
            items: Vec::new(),
            page_number,
            child_nodes: Vec::new(),
        }
    }

    fn is_leaf(&self) -> bool {
        self.child_nodes.is_empty()
    }

    pub fn serialize<const A: usize>(&self, arr: &mut[u8; A]) -> Result<(), io::Error> {

        //is there a nee to write this when the child_nodes_length is already being written?
        // match self.is_leaf() {
        //     true => self.u16_to_bytes(&mut arr[..BYTES_IN_U16], 1),
        //     false => self.u16_to_bytes(&mut arr[..BYTES_IN_U16], 0),
        // }

        let mut child_nodes_length = u16::try_from(self.child_nodes.len()).map_err(|_| ErrorKind::InvalidData)?;
        self.u16_to_bytes(&mut arr[..BYTES_IN_U16], child_nodes_length);

        let mut right_pos = A - 1;

        //
        let mut starting_index = BYTES_IN_U16;
        for child_node in self.child_nodes.iter() {
            let ending_index :usize= starting_index + BYTES_IN_U64;
            self.u64_to_bytes(&mut arr[starting_index..ending_index], child_node.clone());
            starting_index = starting_index.saturating_add(8);
        }

        //

        let mut starting_index = BYTES_IN_U16;
        for (index, item) in self.items.iter().enumerate() {
            if self.is_leaf() {
                let ending_index :usize= starting_index + BYTES_IN_U64;
                let child_node_page_number = self.child_nodes.get(index);

                if let Some(page_number) = child_node_page_number{
                    self.u64_to_bytes(&mut arr[starting_index..ending_index], page_number.clone());
                    starting_index = starting_index.saturating_add(BYTES_IN_U64);
                }
                
            }

            let offset = right_pos.saturating_sub(item.key.len()).saturating_sub(item.value.len());
           // self.u16_to_bytes(&mut arr[..BYTES_IN_U16], offset);

        }

        
        Ok(())
    }

    fn u16_to_bytes(&self, arr: &mut[u8], num:u16) {
        let num :[u8; BYTES_IN_U16]= num.to_le_bytes();
        arr[..BYTES_IN_U16].copy_from_slice(&num);
    }

    fn u64_to_bytes(&self, arr: &mut[u8], num:u64) {
        let num :[u8; BYTES_IN_U64]= num.to_le_bytes();
        arr[..BYTES_IN_U64].copy_from_slice(&num);
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
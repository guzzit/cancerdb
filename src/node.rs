use std::{io::{self, ErrorKind}, collections::HashMap};

use crate::{dal::Dal, freelist::PageNumber, constants::{BYTES_IN_U16, BYTES_IN_U64}};


struct Node {
    dal: Dal,
    items: Vec<Item>,
    //items: HashMap<Box<[u8]>, ItemValue>,
    page_number: PageNumber,
    child_nodes: Vec<PageNumber>,
}

//type ItemValue = (Box<[u8]>, Option<PageNumber>);

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

        let is_leaf = self.is_leaf(); 

        let is_leaf_to_num = match self.is_leaf() {
            true => 1,
            false => 0,
        };

        self.u16_to_bytes(&mut arr[..BYTES_IN_U16], is_leaf_to_num);

        //add checks to ensure A is greater than all these sizes

        let child_nodes_length = u16::try_from(self.items.len()).map_err(|_| ErrorKind::InvalidData)?;
        self.u16_to_bytes(&mut arr[BYTES_IN_U16..BYTES_IN_U16*2], child_nodes_length);

        let mut right_pos = A - 1;

        //
        let mut starting_index = BYTES_IN_U16;
        for child_node in self.child_nodes.iter() {
            let ending_index :usize= starting_index + BYTES_IN_U64;
            self.u64_to_bytes(&mut arr[starting_index..ending_index], child_node.clone());
            starting_index = starting_index.saturating_add(8);
        }

        //

        let mut starting_index = BYTES_IN_U16*2;
        for (index, item) in self.items.iter().enumerate() {
            if is_leaf {
                let ending_index :usize= starting_index + BYTES_IN_U64;
                let child_node_page_number = self.child_nodes.get(index);

                if let Some(page_number) = child_node_page_number{
                    self.u64_to_bytes(&mut arr[starting_index..ending_index], page_number.clone());
                    starting_index = starting_index.saturating_add(BYTES_IN_U64);
                }
                
            }

            let offset = right_pos.saturating_sub(item.value.len()).saturating_sub(item.key.len());
            let offset:u16 = u16::try_from(offset).map_err(|_| ErrorKind::InvalidData)?;
            self.u16_to_bytes(&mut arr[..BYTES_IN_U16], offset);
            starting_index = starting_index.saturating_add(BYTES_IN_U16);
            right_pos = right_pos.saturating_sub(item.value.len());
            self.usize_to_bytes(&mut  arr[right_pos..], &item.value);
            right_pos = right_pos.saturating_sub(item.key.len());
            self.usize_to_bytes(&mut  arr[right_pos..], &item.key);

        }

        //if !self.is_leaf() {
        if !is_leaf && self.child_nodes.len() > self.items.len() {
            let ending_index :usize= starting_index + BYTES_IN_U64;
                let child_node_page_number = self.child_nodes.last();

                if let Some(page_number) = child_node_page_number {
                    self.u64_to_bytes(&mut arr[starting_index..ending_index], page_number.clone());
                }
        }

        Ok(())
    }

    pub fn deserialize<const A: usize>(&mut self, arr: &mut [u8; A]) -> Result<(), io::Error>  {  
        let mut is_leaf = [0; BYTES_IN_U16];
        is_leaf.copy_from_slice(&arr[..BYTES_IN_U16]) ;
        let is_leaf =  u16::from_le_bytes(is_leaf.clone());

        let mut items_length = [0; BYTES_IN_U16];
        items_length.copy_from_slice(&arr[BYTES_IN_U16..BYTES_IN_U16*2]) ;
        let items_length =  u16::from_le_bytes(items_length.clone());

        let mut i = 0;
        let mut starting_index = BYTES_IN_U16*2;
        while i < items_length {
            if is_leaf == 0 {
                let mut child_node_page_number = [0; BYTES_IN_U64];
                child_node_page_number.copy_from_slice(&arr[starting_index..starting_index+BYTES_IN_U64]) ;
                let child_node_page_number =  u64::from_le_bytes(child_node_page_number.clone());
                self.child_nodes.push(child_node_page_number);
                starting_index = starting_index.saturating_add(BYTES_IN_U64);
            }

            let mut offset = [0; BYTES_IN_U16];
            offset.copy_from_slice(&arr[starting_index..starting_index+2]) ;
            let offset =  u16::from_le_bytes(offset.clone());

            let mut key_length = [0; BYTES_IN_U16]; let mut offset:usize = offset.try_into().map_err(|_| ErrorKind::InvalidData)?;
            key_length.copy_from_slice(&arr[offset..offset+2]) ;
            let key_length =  u16::from_le_bytes(key_length.clone());let key_length:usize = key_length.try_into().map_err(|_| ErrorKind::InvalidData)?;

            let mut key: Box<[u8]> = Box::new([0u8]);
            key.copy_from_slice(&arr[offset..offset+key_length]);

            // should saturating_add be used or not?
            offset = offset.saturating_add(key_length);


            let mut value_length = [0; BYTES_IN_U16];
            value_length.copy_from_slice(&arr[offset..offset+2]) ;
            let value_length =  u16::from_le_bytes(value_length.clone());let value_length:usize = value_length.try_into().map_err(|_| ErrorKind::InvalidData)?;

            let mut value: Box<[u8]> = Box::new([0u8]);
            value.copy_from_slice(&arr[offset..offset+value_length]);

            let item = Item::new(key, value);
            self.items.push(item);
        }
        if is_leaf == 0 && self.child_nodes.len() > self.items.len() {
            let mut child_node_page_number = [0; BYTES_IN_U64];
            child_node_page_number.copy_from_slice(&arr[starting_index..starting_index+BYTES_IN_U64]) ;
            let child_node_page_number =  u64::from_le_bytes(child_node_page_number.clone());
            self.child_nodes.push(child_node_page_number);
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

    //rename num
    fn usize_to_bytes(&self, arr: &mut[u8], num:&Box<[u8]>) {
    //fn usize_to_bytes<const A: usize>(&self, arr: &mut[u8; A], num:&Box<[u8]>) {

       // let num = num.to_le_bytes();
        //check that A is greater or equal to num.len()
        arr[..num.len()].copy_from_slice(&num);
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
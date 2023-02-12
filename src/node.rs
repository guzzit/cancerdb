use std::{io::{self, ErrorKind, Error}, collections::HashMap, cmp::Ordering};

use crate::{dal::Dal, freelist::PageNumber, constants::{BYTES_IN_U16, BYTES_IN_U64, META_PAGE_NUM, BYTE_IN_U8}};


pub struct Node {
    //dal: &'a mut Dal,
    items: Vec<Item>,
    //items: HashMap<Box<[u8]>, ItemValue>,
    page_number: PageNumber,
    child_nodes: Vec<PageNumber>,
}

//type ItemValue = (Box<[u8]>, Option<PageNumber>);

impl Node {
    pub fn build (page_number: PageNumber) -> Result<Self, io::Error> {

        if page_number == META_PAGE_NUM {
            return Err(Error::new(ErrorKind::InvalidData, 
                "node page_number should not be equal to META_PAGE_NUM. /r/n 
                node page_number: {page_number:?} /r/n
                META_PAGE_NUM : {META_PAGE_NUM:?}"))
        }

        Ok(Node {
            //dal,
            items: Vec::new(),
            page_number,
            child_nodes: Vec::new(),
        })
    }

    fn is_leaf(&self) -> bool {
        self.child_nodes.is_empty()
    }

    //for testing purposes
    pub fn set_node_item(&mut self) {
        let key1 = Box::new(b"Key1".to_owned());
        let value1 = Box::new(b"value1".to_owned());
        let item = Item::new(key1, value1);
        self.items.push(item);        
    }

    pub fn get_page_number(&self) -> PageNumber {
        self.page_number
    }

    pub fn serialize<const A: usize>(&self, arr: &mut[u8; A]) -> Result<(), io::Error> {
        //enfore that A is the same size as const page_size

        let is_leaf = self.is_leaf(); 

        let is_leaf_to_num = match self.is_leaf() {
            true => 1,
            false => 0,
        };

        self.u8_to_bytes(&mut arr[..BYTE_IN_U8], is_leaf_to_num);

        //add checks to ensure A is greater than all these sizes

        let child_nodes_length = u16::try_from(self.items.len()).map_err(|_| ErrorKind::InvalidData)?;
        self.u16_to_bytes(&mut arr[BYTES_IN_U16..BYTES_IN_U16*2], child_nodes_length);

        let mut right_pos = A - 1;

        //
        //let mut starting_index = BYTES_IN_U16;
        // for child_node in self.child_nodes.iter() {
        //     let ending_index :usize= starting_index + BYTES_IN_U64;
        //     self.u64_to_bytes(&mut arr[starting_index..ending_index], child_node.clone());
        //     starting_index = starting_index.saturating_add(8);
        // }

        //if item.len() + 1 != child_nodes.len() and !is_leaf
        //return error

        let mut starting_index = BYTES_IN_U16 + BYTE_IN_U8;
        for (index, item) in self.items.iter().enumerate() {
            if !is_leaf {
                let child_node_page_number = self.child_nodes.get(index);

                if let Some(page_number) = child_node_page_number{
                let ending_index :usize= starting_index + BYTES_IN_U64;
                    self.u64_to_bytes(&mut arr[starting_index..ending_index], page_number.clone());
                    starting_index = starting_index.saturating_add(BYTES_IN_U64);
                }
                
            }

            //let offset = right_pos - item.value.len() - item.key.len();
            //let offset:u16 = u16::try_from(offset).map_err(|_| ErrorKind::InvalidData)?;
            //self.u16_to_bytes(&mut arr[..BYTES_IN_U16], offset);
            let key_length = u8::try_from(item.key.len()).map_err(|_| ErrorKind::InvalidData)?;
            self.u8_to_bytes(&mut arr[starting_index..starting_index+BYTE_IN_U8], key_length);
            starting_index = starting_index + BYTE_IN_U8;

            let value_length = u8::try_from(item.value.len()).map_err(|_| ErrorKind::InvalidData)?;
            self.u8_to_bytes(&mut arr[starting_index..starting_index+BYTE_IN_U8], value_length);
            starting_index = starting_index + BYTE_IN_U8;

            right_pos = right_pos - item.value.len();
            self.usize_to_bytes(&mut  arr[right_pos..], &item.value);
            right_pos = right_pos - item.key.len();
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

     // may need to cut all the steps into smaller functions
     pub fn deserialize<const A: usize>(&mut self, arr: &mut [u8; A]) -> Result<(), io::Error>  {    
        //enfore that A is the same size as const page_size
        
        let mut right_pos = A - 1;

        // read leaf      
        let mut is_leaf = [0u8];
        is_leaf.copy_from_slice(&arr[..BYTE_IN_U8]) ;
        let is_leaf =  u8::from_le_bytes(is_leaf);

        // read items_length
        let mut items_length = [0; BYTES_IN_U16];
        let items_length_end = BYTE_IN_U8 + BYTES_IN_U16;
        items_length.copy_from_slice(&arr[BYTE_IN_U8..items_length_end]) ;
        //why isn't little endian working?
        let items_length =  u16::from_be_bytes(items_length);

        let mut i = 0;
        let mut left_position = items_length_end;
        while i < items_length {
            if is_leaf == 0 {
                // read child node
                let mut child_node_page_number = [0; BYTES_IN_U64];
                let child_node_page_number_end = left_position + BYTES_IN_U64;
                child_node_page_number.copy_from_slice(&arr[left_position..child_node_page_number_end]);
                let child_node_page_number =  u64::from_le_bytes(child_node_page_number);
                self.child_nodes.push(child_node_page_number);
                left_position = left_position + BYTES_IN_U64;
            }

            // read offset
            // let mut offset = [0; BYTES_IN_U16];
            // offset.copy_from_slice(&arr[left_position..left_position+BYTES_IN_U16]) ;
            // let offset =  u16::from_le_bytes(offset);
            // left_position = left_position + BYTES_IN_U64;

            // read key length
            let mut key_length = [0u8];
            key_length.copy_from_slice(&arr[left_position..left_position+BYTE_IN_U8]);
            let key_length =  u8::from_le_bytes(key_length);
            left_position = left_position + BYTE_IN_U8;

            // read value length
            let mut value_length = [0u8];
            value_length.copy_from_slice(&arr[left_position..left_position+BYTE_IN_U8]);
            let value_length =  u8::from_le_bytes(value_length);
            left_position = left_position + BYTE_IN_U8;
            

            let mut offset = right_pos - usize::try_from(key_length + value_length).map_err(|_| ErrorKind::InvalidData)?;
            right_pos = offset;
            // read key
            let key_length:usize = key_length.try_into().map_err(|_| ErrorKind::InvalidData)?;
            let mut key: Box<[u8]> = vec![0; key_length].into_boxed_slice().to_owned();
            // the extra one byte you add to the offset is for the key length (explain better)
            //offset = offset + 1;
            key.copy_from_slice(&arr[offset..offset+key_length]);

            offset = offset + key_length;

            // read value length
            //let mut value_length =  [0u8];
            //value_length.copy_from_slice(&arr[offset..offset+1]) ;
            //let value_length =  u8::from_le_bytes(value_length.clone());

            //read value
            let value_length:usize = value_length.try_into().map_err(|_| ErrorKind::InvalidData)?;
            // the extra one byte you add to the offset is for the value length (explain better)
            let mut value: Box<[u8]> =  vec![0; value_length].into_boxed_slice().to_owned();
            //offset = offset + 1;
            value.copy_from_slice(&arr[offset..offset+value_length]);

            let item = Item::new(key, value);
            self.items.push(item);

            i = i + 1;
        }

        if is_leaf == 0 && self.child_nodes.len() > self.items.len() {
            let mut child_node_page_number = [0; BYTES_IN_U64];
            child_node_page_number.copy_from_slice(&arr[left_position..left_position+BYTES_IN_U64]);
            let child_node_page_number =  u64::from_le_bytes(child_node_page_number.clone());
            self.child_nodes.push(child_node_page_number);
        }

        Ok(())
    }

    // // may need to cut all the steps into smaller functions
    // pub fn deserialize<const A: usize>(&mut self, arr: &mut [u8; A]) -> Result<(), io::Error>  {    
    //     // read leaf      
    //     let mut is_leaf = [0u8];
    //     is_leaf.copy_from_slice(&arr[..BYTE_IN_U8]) ;
    //     let is_leaf =  u8::from_le_bytes(is_leaf);

    //     // read items_length
    //     let mut items_length = [0; BYTES_IN_U16];
    //     let items_length_end = BYTE_IN_U8 + BYTES_IN_U16;
    //     items_length.copy_from_slice(&arr[BYTE_IN_U8..items_length_end]) ;
    //     let items_length =  u16::from_le_bytes(items_length);

    //     let mut i = 0;
    //     let mut left_position = items_length_end;
    //     while i < items_length {
    //         if is_leaf == 0 {
    //             // read child node
    //             let mut child_node_page_number = [0; BYTES_IN_U64];
    //             let child_node_page_number_end = left_position + BYTES_IN_U64;
    //             child_node_page_number.copy_from_slice(&arr[left_position..child_node_page_number_end]);
    //             let child_node_page_number =  u64::from_le_bytes(child_node_page_number);
    //             self.child_nodes.push(child_node_page_number);
    //             left_position = left_position + BYTES_IN_U64;
    //         }

    //         // read offset
    //         let mut offset = [0; BYTES_IN_U16];
    //         offset.copy_from_slice(&arr[left_position..left_position+BYTES_IN_U16]) ;
    //         let offset =  u16::from_le_bytes(offset);
    //         left_position = left_position + BYTES_IN_U64;

    //         // read key length
    //         let mut key_length = [0u8];
    //         let mut offset:usize = offset.try_into().map_err(|_| ErrorKind::InvalidData)?;
    //         key_length.copy_from_slice(&arr[offset..offset+BYTE_IN_U8]);
    //         let key_length =  u8::from_le_bytes(key_length);

    //         // read key
    //         let key_length:usize = key_length.try_into().map_err(|_| ErrorKind::InvalidData)?;
    //         let mut key: Box<[u8]> = vec![0; key_length].into_boxed_slice().to_owned();
    //         // the extra one byte you add to the offset is for the key length (explain better)
    //         offset = offset + 1;
    //         key.copy_from_slice(&arr[offset..offset+key_length]);

    //         offset = offset + key_length;

    //         // read value length
    //         let mut value_length =  [0u8];;
    //         value_length.copy_from_slice(&arr[offset..offset+1]) ;
    //         let value_length =  u8::from_le_bytes(value_length.clone());

    //         //read value
    //         let value_length:usize = value_length.try_into().map_err(|_| ErrorKind::InvalidData)?;
    //         // the extra one byte you add to the offset is for the value length (explain better)
    //         let mut value: Box<[u8]> =  vec![0; value_length].into_boxed_slice().to_owned();
    //         offset = offset + 1;
    //         value.copy_from_slice(&arr[offset..offset+value_length]);

    //         let item = Item::new(key, value);
    //         self.items.push(item);

    //         i = i + 1;
    //     }

    //     if is_leaf == 0 && self.child_nodes.len() > self.items.len() {
    //         let mut child_node_page_number = [0; BYTES_IN_U64];
    //         child_node_page_number.copy_from_slice(&arr[left_position..left_position+BYTES_IN_U64]) ;
    //         let child_node_page_number =  u64::from_le_bytes(child_node_page_number.clone());
    //         self.child_nodes.push(child_node_page_number);
    //     }


    //     Ok(())
    // }

    fn u8_to_bytes(&self, arr: &mut[u8], num:u8) {
        let num :[u8; BYTE_IN_U8]= num.to_le_bytes();
        arr[..BYTE_IN_U8].copy_from_slice(&num);
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

    fn write_node(&mut self, node: &Node, dal: &mut Dal) -> Result<(), io::Error> {
        dal.write_node(node)?;
        Ok(())
    }

    fn write_nodes(&mut self, nodes: &Vec<Node>, dal: &mut Dal) -> Result<(), io::Error> {
        for node in nodes.iter() {
            self.write_node(node, dal)?
        }
        
        Ok(())
    }

    fn get_node(& mut self, page_number: PageNumber, dal: &mut Dal) -> Result<Node, io::Error> {
        let node = dal.get_node(page_number)?;
        Ok(node)
    }

    fn find_key_in_node(&mut self, key: &Box<[u8]>) -> Option<(bool, usize)> {

        if self.items.is_empty() {
            return None;
        }


        for (index, item) in self.items.iter().enumerate() {
             let k = std::str::from_utf8(item.key.as_ref()).unwrap();
             let v = std::str::from_utf8(item.value.as_ref()).unwrap();
            match  item.key.cmp(&key) {
                Ordering::Less => continue,
                Ordering::Equal => return Some((true, index)),
                Ordering::Greater => return Some((false, index)),
            }
        }


        Some((false, self.items.len().saturating_sub(1)))
    }

    pub fn find_key(&mut self, key: Box<[u8]>, dal: &mut Dal) -> Result<Option<Item>, io::Error> {
        if let Some((key_found, index)) = self.find_key_in_node(&key) {
            if key_found {
                let item = self.items.get(index).ok_or_else(|| ErrorKind::InvalidData)?;
                return Ok(Some(item.clone()));
                //return Ok(Some((index, node)));
            }

            if self.is_leaf() {
                return Ok(None);//or not found enum
            }

            if let Some(child_node_page_num) = self.child_nodes.get(index) {
                let mut child_node = self.get_node(child_node_page_num.clone(), dal)?;
                
                return Node::find_key(&mut child_node, key, dal);
            }

            return Ok(None);
        }
        //let Some(key_found, index) = self.find_key_in_node(key);
        Ok(None)
    }

    // pub fn find_key(&'a mut self, key: Box<[u8]>) -> Result<Option<(usize, &'a mut Node<'a>)>, io::Error> {
    //     if let Some((key_found, index)) = self.find_key_in_node(&key) {
    //         if key_found {
    //             return Ok(Some((index, self)));
    //         }

    //         if self.is_leaf() {
    //             return Ok(None);//or not found enum
    //         }

    //         if let Some(child_node_page_num) = self.child_nodes.get(index) {
    //             self = & mut self.get_node(child_node_page_num.clone())?;
    //             return self.find_key(key);
    //         }

    //         return Ok(None);
    //     }
    //     //let Some(key_found, index) = self.find_key_in_node(key);
    //     Ok(None)
    // }
    
}

#[derive(Clone, Debug)]
pub struct Item {
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
use std::{io::{self, ErrorKind, Error}, cmp::Ordering};

use crate::{dal::Dal, freelist::PageNumber, constants::{BYTES_IN_U16, BYTES_IN_U64, META_PAGE_NUM, BYTE_IN_U8, NODE_HEADER_SIZE}};

#[derive(Clone, Debug)]
pub struct Node {
    //dal: &'a mut Dal,
    pub items: Vec<Item>,
    //items: HashMap<Box<[u8]>, ItemValue>,
    page_number: PageNumber,
    pub child_nodes: Vec<PageNumber>,
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

    pub fn is_leaf(&self) -> bool {
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

    pub fn set_page_number(&mut self, dal: &mut Dal) {
        self.page_number = dal.freelist.get_next_page();
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

        if !self.is_leaf() {
        //if !is_leaf && self.child_nodes.len() > self.items.len() {
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

        if is_leaf == 0 {
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

    // fn write_node(&mut self, node: &Node, dal: &mut Dal) -> Result<(), io::Error> {
    //     dal.write_node(node)?;
    //     Ok(())
    // }

    fn write(&mut self, dal: &mut Dal) -> Result<(), io::Error> {
        dal.write_node(self)?;
        Ok(())
    }

    // fn write_nodes(&mut self, nodes: &Vec<Node>, dal: &mut Dal) -> Result<(), io::Error> {
    //     for node in nodes.iter() {
    //         self.write_node(node, dal)?
    //     }
        
    //     Ok(())
    // }

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

    pub fn find_key(&mut self, key: &Box<[u8]>, dal: &mut Dal) -> Result<Option<Item>, io::Error> {
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

            // it should never get to this point. It is either a leaf or has that child node
            return Ok(None);
        }
        //let Some(key_found, index) = self.find_key_in_node(key);
        Ok(None)
    }

    // pub fn find_node(&mut self, key: &Box<[u8]>, dal: &mut Dal) -> Result<Option<Node>, io::Error> {
    //     if let Some((key_found, index)) = self.find_key_in_node(&key) {
    //         if key_found {
    //             let item = self.items.get(index).ok_or_else(|| ErrorKind::InvalidData)?;
    //             return Ok(Some(self.clone()));
    //             //return Ok(Some((index, node)));
    //         }

    //         if self.is_leaf() {
    //             return Ok(None);//or not found enum
    //         }

    //         if let Some(child_node_page_num) = self.child_nodes.get(index) {
    //             let mut child_node = self.get_node(child_node_page_num.clone(), dal)?;
                
    //             return Node::find_node(&mut child_node, key, dal);
    //         }

    //         return Ok(None);
    //     }
    //     Ok(None)
    // }

//rename usize type
    pub fn find_node(&mut self, key: &Box<[u8]>, dal: &mut Dal, ancestor_page_numbers: &mut Vec<PageNumber>) -> Result<Option<(Node, usize)>, io::Error> {
        if let Some((key_found, index)) = self.find_key_in_node(&key) {
            ancestor_page_numbers.push(self.get_page_number());
            if key_found {
                //let item = self.items.get(index).ok_or_else(|| ErrorKind::InvalidData)?;
                return Ok(Some((self.clone(), index)));
                //return Ok(Some((index, node)));
            }

            // return the same with above, size the above should lead to an update
            // and this should lead to an insert
            if self.is_leaf() {
                return Ok(Some((self.clone(), 0)));//or not found enum
            }

            if let Some(child_node_page_num) = self.child_nodes.get(index) {
                let mut child_node = self.get_node(child_node_page_num.clone(), dal)?;
                
                return Node::find_node(&mut child_node, key, dal, ancestor_page_numbers);
            }

            // it should never get to this point. It is either a leaf or has that child node
            return Ok(None);
        }
        Ok(None)
    }

    pub fn calculate_element_size(&self, item: &Item) -> usize {
        let mut size = 0;
        size = size + item.key.len() + item.value.len() + BYTE_IN_U8 + BYTE_IN_U8; //2 BYTE_IN_U8 values represent the lengths of 
        //the key_length value and the value_length value
        size = size + BYTES_IN_U64; //BYTES_IN_U64 represent the page num size
        size
    }

    pub fn calculate_node_size(&self) -> usize {
        let mut size = 0;
        size = size + NODE_HEADER_SIZE; 

        for item in self.items.iter() {
            size = size + self.calculate_element_size(item);
        }

        // Add last page
        size = size + BYTES_IN_U64;
        size
    }

    pub fn add_item(&mut self, insertion_index: usize, item: Item) -> Result<(), io::Error>{
        if insertion_index > self.items.len() {
            // it should be an array out ot bounds error, not invalid data
            return Err(Error::new(ErrorKind::InvalidData, 
                "Array out of bouunds"));
        }

        self.items.insert(insertion_index, item);
        Ok(())
    }

    pub fn is_overpopulated(&self, dal:&mut Dal) -> Result<bool, io::Error> {
        dal.node_is_overpopulated(self)
    }

    fn is_underpopulated(&mut self, dal:&mut Dal) -> Result<bool, io::Error> {
        dal.node_is_underpopulated(self)
    }

    pub fn split_child(&mut self, child_node:&mut Node, child_node_index:usize, dal:&mut Dal) -> Result<(), io::Error> {
        let (middle_item, mut new_node) = child_node.split(dal)?;

        self.add_item(child_node_index, middle_item)?;

        //if self.child_nodes.len() == node_index + 1 {
        //    self.child_nodes.push(newNode.page_number)
        //}
        //else {
            self.child_nodes.insert(child_node_index, new_node.page_number);
        //}

        new_node.write(dal)?;
        child_node.write(dal)?;
        self.write(dal)?;

        Ok(())
    }

    fn split(&mut self, dal:&mut Dal) -> Result<(Item, Node), io::Error> {
        let split_index = dal.get_split_index(self)?.ok_or_else(|| ErrorKind::InvalidData)?.clone();
        //check that this remove call can't panic
        let middle_item = self.items.remove(split_index);
        let new_node = if self.is_leaf() {
            //let pg_num = dal.freelist.get_next_page();
            //let mut new_node = Node::build( pg_num)?;
            let mut new_node = dal.new_node()?;
            //let (a, b) = node.items.split_at(split_index+1);
            //new_node.items.append(&mut b.to_vec());

            //check that drain call can't panic
            let mut split_items: Vec<Item> = self.items.drain(split_index..).collect();
            new_node.items.append(&mut split_items);
            //self.write_node(&new_node, dal)?;
            //new_node.write(dal)?;
            new_node
        }
        else {
            //let pg_num = dal.freelist.get_next_page();
            //let mut new_node = Node::build( pg_num)?;
            let mut new_node = dal.new_node()?;
            // let (a, b) = node.items.split_at(split_index+1);
            // let (c, d) = node.child_nodes.split_at(split_index+1);
            // new_node.items.append(&mut b.to_vec());
            // new_node.child_nodes.append(&mut d.to_vec());
            //check that drain call can't panic
            let mut split_items: Vec<Item> = self.items.drain(split_index..).collect();
            let mut split_child_nodes: Vec<u64> = self.child_nodes.drain(split_index..).collect();
            new_node.items.append(&mut split_items);
            new_node.child_nodes.append(&mut split_child_nodes);
            //self.write_node(&new_node, dal)?;
            //new_node.write(dal)?;
            new_node
        };

        Ok((middle_item, new_node))
        
    }

    // fn split(&mut self, node:&mut Node, node_index: usize, split_index:usize, dal:&mut Dal) -> Result<(), io::Error> {
    //     //check the two indices are less than item length - 1
    //     let middle_item = node.items.get(split_index).ok_or_else(|| ErrorKind::InvalidData)?.clone();
    //     let newNode = if node.is_leaf() {
    //         //let pg_num = dal.freelist.get_next_page();
    //         //let mut new_node = Node::build( pg_num)?;
    //         let mut new_node = dal.new_node()?;
    //         //let (a, b) = node.items.split_at(split_index+1);
    //         //new_node.items.append(&mut b.to_vec());
            
    //         let mut split_items: Vec<Item> = node.items.drain(split_index..).collect();
    //         new_node.items.append(&mut split_items);
    //         //self.write_node(&new_node, dal)?;
    //         new_node.write(dal)?;
    //         new_node
    //     }
    //     else {
    //         //let pg_num = dal.freelist.get_next_page();
    //         //let mut new_node = Node::build( pg_num)?;
    //         let mut new_node = dal.new_node()?;
    //         // let (a, b) = node.items.split_at(split_index+1);
    //         // let (c, d) = node.child_nodes.split_at(split_index+1);
    //         // new_node.items.append(&mut b.to_vec());
    //         // new_node.child_nodes.append(&mut d.to_vec());
    //         let mut split_items: Vec<Item> = node.items.drain(split_index..).collect();
    //         let mut split_child_nodes: Vec<u64> = node.child_nodes.drain(split_index..).collect();
    //         new_node.items.append(&mut split_items);
    //         new_node.child_nodes.append(&mut split_child_nodes);
    //         //self.write_node(&new_node, dal)?;
    //         new_node.write(dal)?;
    //         new_node
    //     };

    //     self.add_item(node_index, middle_item)?;

    //     //if self.child_nodes.len() == node_index + 1 {
    //     //    self.child_nodes.push(newNode.page_number)
    //     //}
    //     //else {
    //         self.child_nodes.insert(node_index, newNode.page_number);
    //     //}

    //     node.write(dal)?;
    //     self.write(dal)?;

    //     Ok(())
    // }


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
    pub fn new(key: Box<[u8]>, value: Box<[u8]>) -> Self {
        Item{
            key,
            value,
        }
    }

    pub fn get_key(&self) -> &Box<[u8]> {
        &self.key
    }

    pub fn get_value(&self) -> &Box<[u8]> {
        &self.value
    }
}

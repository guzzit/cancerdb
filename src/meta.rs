use crate::{freelist::PageNum, constants::BYTES_IN_U64};


pub struct Meta {
    pub freelist_page: Option<PageNum>
}

impl Meta {
    pub fn new() -> Self {
        Meta { freelist_page: None }
    }

    //need to find a way to make arr type &mut[u8;BYTES_IN_U64]
    pub fn serialize(&self, arr: &mut[u8]) {
        
        let page_num = self.freelist_page.unwrap();

        // let odd_or_even = page_num & 1;
        // if page_num & 1 == 1 {
        //     print!("odd");
        // }
        // else {
        //     print!("even");
        // }

        let page_num :[u8; BYTES_IN_U64]= page_num.to_le_bytes();

        arr[..BYTES_IN_U64].copy_from_slice(&page_num);
    }

    pub fn deserialize(&mut self, array: &[u8; BYTES_IN_U64]) {
        //self.freelist_page = Some(self.byte_to_u64(array));
        self.freelist_page = Some(u64::from_le_bytes(array.clone()));
    }
    
    // indicate whether little or big endian
    // also might have to put this method in a utility struct
    fn byte_to_u64 (&mut self, array: &[u8; BYTES_IN_U64]) -> u64 {
        ((array[0] as u64) <<  0) +
        ((array[1] as u64) <<  8) +
        ((array[2] as u64) << 16) +
        ((array[3] as u64) << 24) +
        ((array[4] as u64) << 32) +
        ((array[5] as u64) << 40) +
        ((array[6] as u64) << 48) +
        ((array[7] as u64) << 56) 
    }

}

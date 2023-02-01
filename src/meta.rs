use std::io::{self, ErrorKind};

use crate::{freelist::PageNumber, constants::BYTES_IN_U64};


pub struct Meta {
    pub root_page: Option<PageNumber>,
    pub freelist_page: Option<PageNumber>,
}

impl Meta {
    pub fn new() -> Self {
        Meta { root_page: None, freelist_page: None }
    }

    //need to find a way to make arr type &mut[u8;BYTES_IN_U64]
    pub fn serialize(&self, arr: &mut[u8; BYTES_IN_U64]) -> Result<(), io::Error>{
        let root_page = self.root_page.ok_or_else(|| ErrorKind::InvalidData)?;
        let freelist_page = self.freelist_page.ok_or_else(|| ErrorKind::InvalidData)?;

        // let odd_or_even = page_num & 1;
        // if page_num & 1 == 1 {
        //     print!("odd");
        // }
        // else {
        //     print!("even");
        // }
        let root_page :[u8; BYTES_IN_U64]= root_page.to_le_bytes();
        let freelist_page :[u8; BYTES_IN_U64]= freelist_page.to_le_bytes();

        arr[..BYTES_IN_U64].copy_from_slice(&root_page);
        arr[BYTES_IN_U64..BYTES_IN_U64*2].copy_from_slice(&freelist_page);

        Ok(())
    }

    pub fn deserialize(&mut self, array: &[u8; BYTES_IN_U64*2]) {
        //self.freelist_page = Some(self.byte_to_u64(array));
        let mut root_page = [0; BYTES_IN_U64];
        root_page.copy_from_slice(&array[..BYTES_IN_U64]) ;
        self.root_page =  Some(u64::from_le_bytes(root_page.clone()));

        let mut freelist_page = [0; BYTES_IN_U64];
        freelist_page.copy_from_slice(&array[BYTES_IN_U64..BYTES_IN_U64*2]);
        self.freelist_page =  Some(u64::from_le_bytes(freelist_page.clone()));

        //self.freelist_page = Some(u64::from_le_bytes(array.clone()));
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

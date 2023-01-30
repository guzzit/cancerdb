use std::io::{self, ErrorKind, Error};

use crate::constants::BYTES_IN_U64;


pub type PageNumber = u64;

pub struct Freelist {
   max_page: PageNumber,
   released_pages: Vec<u64>,
}

const META_PAGE:u64 = 0;

impl Freelist {
    pub fn new() -> Self {
        Freelist {
            max_page: META_PAGE,
            released_pages: Vec::new(),
        }
    }
     
    // if page taken isn't used, add back?
    pub fn get_next_page(&mut self) -> PageNumber {
        let next_page = self.released_pages.iter().take(1).next();
    
        if let Some(page) = next_page {
            page.to_owned()
        }
        else {
            self.max_page = self.max_page.saturating_add(1);
            self.max_page
        }
            
    }

    // should there be a check to ensure that the page is empty? or at least to 
    // clear the page to be extra sure it's empty
    fn release_page(&mut self, page_num: PageNumber) {
        self.released_pages.push(page_num);
        
    }

    pub fn serialize<const A: usize>(&self, arr: &mut[u8; A]) -> Result<(), io::Error> {

        // the 2 is for page_count and max_space
        let total_space_needed = BYTES_IN_U64 * (self.released_pages.len() + 2);

        if total_space_needed > A {
            return Err(Error::new(ErrorKind::InvalidData, 
                "Array too small. /r/n 
                Total space needed: {total_space_needed:?} /r/n
                Total space in array : {arr:?}"))
        }

        let page_count:u64 = u64::try_from(self.released_pages.len()).map_err(|_| ErrorKind::InvalidData)?;
        
        self.u64_to_bytes(&mut arr[..BYTES_IN_U64], self.max_page);
        self.u64_to_bytes(&mut arr[BYTES_IN_U64..BYTES_IN_U64*2], page_count);


        let mut starting_index = BYTES_IN_U64*2;
        for released_page in self.released_pages.iter() {
            let ending_index :usize= starting_index + BYTES_IN_U64;
            self.u64_to_bytes(&mut arr[starting_index..ending_index], released_page.clone());
            starting_index = starting_index.saturating_add(BYTES_IN_U64);
        }

        Ok(())

    }

    pub fn deserialize<const A: usize>(&mut self, arr: &[u8; A]) -> Result<(), io::Error>  {        
        let mut chunks = arr.chunks_exact(BYTES_IN_U64); 
        //
        self.max_page = Freelist::bytes_to_u64(chunks.nth(0))?;
        let page_count = Freelist::bytes_to_u64(chunks.nth(0))?;
        //
        //self.max_page = self.bytes_to_u64( chunks.nth(0).unwra());
        //let page_count = self.bytes_to_u64( chunks.nth(0).unwra());

        let mut i = 0;
        while i < page_count {
            let val = Freelist::bytes_to_u64(chunks.nth(0))?;//self.bytes_to_u64(chunks.nth(0).unwra());
            self.released_pages.push(val);   
            i = i + 1;        
        }

        Ok(())
    }

    fn bytes_to_u64(bytes: Option<&[u8]>) -> Result<u64, io::Error> {
        let array: &[u8;BYTES_IN_U64] = bytes.ok_or_else(|| ErrorKind::InvalidData)?
        .try_into().map_err( |_| ErrorKind::InvalidData)?;
        let number = u64::from_le_bytes(array.clone());
        Ok(number)
    }

    // indicate whether little or big endian
    // also might have to put this method in a utility struct
    // fn bytes_to_u64 (&mut self, array: &[u8]) -> u64 {
    //     ((array[0] as u64) <<  0) +
    //     ((array[1] as u64) <<  8) +
    //     ((array[2] as u64) << 16) +
    //     ((array[3] as u64) << 24) +
    //     ((array[4] as u64) << 32) +
    //     ((array[5] as u64) << 40) +
    //     ((array[6] as u64) << 48) +
    //     ((array[7] as u64) << 56) 
    // }

    fn u64_to_bytes(&self, arr: &mut[u8], num:u64) {
        let num :[u8; BYTES_IN_U64]= num.to_le_bytes();
        arr[..BYTES_IN_U64].copy_from_slice(&num);
    }
}

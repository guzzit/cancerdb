use crate::freelist::PageNum;
use serde::{Serialize, Deserialize};

pub const META_PAGE_NUM:u64 = 0;

#[derive(Serialize, Deserialize, Debug)]
pub struct Meta {
    pub freelist_page: Option<PageNum>
}

impl Meta {
    pub fn new() -> Self {
        Meta { freelist_page: None }
    }

    pub fn serialize<const A: usize>(&self, arr: &mut[u8; A]) {
        //
        let page_num = self.freelist_page.unwrap();

        let odd_or_even = page_num & 1;
        if page_num & 1 == 1 {
            print!("odd");
        }
        else {
            print!("even");
        }
        let page_num :[u8; 8]= page_num.to_le_bytes();
        assert!(A >= page_num.len()); //just for a nicer error message, adding #[track_caller] to the function may also be desirable

        arr[..A].copy_from_slice(&page_num);

       
        
        //
        // let serialized = serde_json::to_string(&self).unwrap();
        // let meta = serialized.as_bytes();

        // let meta = bincode::serialize(&self).unwrap();
        // let meta = bincode::serialize(&self).unwrap();
        
        // let meta = meta.into_boxed_slice();

        // *meta
    }

    pub fn deserialize(&mut self, array: &[u8; 8]) {
        self.freelist_page = Some(self.byte_to_u64(array));
    }

    
    fn byte_to_u64 (&mut self, array: &[u8; 8]) -> u64 {
        ((array[0] as u64) <<  0) +
        ((array[1] as u64) <<  8) +
        ((array[2] as u64) << 16) +
        ((array[3] as u64) << 24) +
        ((array[4] as u64) << 32) +
        ((array[5] as u64) << 40) +
        ((array[6] as u64) << 48) +
        ((array[7] as u64) << 56) 
    }
    
fn as_u32_le(array: &[u8; 4]) -> u32 {
    ((array[0] as u32) <<  0) +
    ((array[1] as u32) <<  8) +
    ((array[2] as u32) << 16) +
    ((array[3] as u32) << 24)
}



    // fn deserialize<const A: usize>(&self, arr: &mut[u8; A]) {
    //     let free_list_page:u64 = arr.try_into().unwrap();
    // }
}


use std::error::Error;
use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::os::unix::prelude::FileExt;

use crate::freelist::{self, Freelist};
use crate::meta::{self, Meta, META_PAGE_NUM};


pub const PAGE_SIZE:usize = 1024 * 4;
/// Dal stands for Data Access Layer
pub struct Dal {
   file: File,
   page_size: u64,
   pub freelist: Freelist,
}

impl Dal {
    pub fn build(path: &str) -> Result<Self, io::Error> {
        let file = fs::File::create(path)?;
        Ok(Dal {
            file,
            page_size: u64::try_from(PAGE_SIZE).unwrap(),
            freelist: Freelist::new()
        })
    }

    pub fn allocate_empty_page(&self, page_num: u64) -> Page {
        Page::new(page_num)
    }

    fn read_page(&mut self, page_num: u64) -> Result<Page, io::Error> {
        let mut page = self.allocate_empty_page(page_num); 

        let offset = page_num * self.page_size;
        //let mut buf = [0u8; PAGE_SIZE];

        //dal.file.seek(SeekFrom::Start(offset))?;
        //dal.file.read_exact(&mut page.data)?;
        self.file.read_exact_at(&mut page.data, offset)?;
        //page.data = buf;
        Ok(page)
    }

    pub fn write_page(&mut self, page: &Page) -> Result<(), io::Error> {    
        let offset = page.num * self.page_size;
       // dal.file.seek(SeekFrom::Start(offset))?;
        self.file.write_all_at(&page.data, offset)?;
        Ok(())
        //dal.file.write(&page.data)
    }    
    
    fn write_meta(&self, meta: Meta) -> Result<Page, io::Error> {
        let mut page = self.allocate_empty_page(META_PAGE_NUM);
        meta.serialize(&mut page.data);
        Ok(page)
    }

    fn read_meta(&mut self) -> Result<Meta, io::Error> {
        let page = self.read_page(META_PAGE_NUM).unwrap();
        let mut page_data = [0u8; 8];
        page_data.copy_from_slice(&page.data[..7]);
        let mut meta = Meta::new();
        meta.deserialize(&page_data);
        Ok(meta)
    }
    // function for close?
    
}

pub struct Page {
    pub num: u64,
    pub data: [u8; PAGE_SIZE],
}

impl Page {
    /// allocate an empty page
    fn new(num: u64) -> Self { 
        Page { num, data: [0u8; PAGE_SIZE] }
    }

    // fn read_page(dal: &mut Dal, page_num: u64) -> Result<Self, io::Error> {
    //     let mut page = Page::new(&dal);

    //     let offset = page_num * dal.page_size;
    //     //let mut buf = [0u8; PAGE_SIZE];

    //     //dal.file.seek(SeekFrom::Start(offset))?;
    //     //dal.file.read_exact(&mut page.data)?;
    //     dal.file.read_exact_at(&mut page.data, offset)?;
    //     //page.data = buf;
    //     Ok(page)
    // }

    // fn write_page(dal: &mut Dal, page: &Page) -> Result<(), io::Error> {    
    //     let offset = page.num.unwrap() * dal.page_size;
    //    // dal.file.seek(SeekFrom::Start(offset))?;
    //     dal.file.write_all_at(&page.data, offset)?;
    //     Ok(())
    //     //dal.file.write(&page.data)
    // }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}



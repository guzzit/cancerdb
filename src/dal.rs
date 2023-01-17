

use std::error::Error;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write, ErrorKind};
use std::os::unix::prelude::FileExt;

use crate::freelist::{self, Freelist};
use crate::meta::{self, Meta, META_PAGE_NUM};


pub const PAGE_SIZE:usize = 1024 * 4;
/// Dal stands for Data Access Layer
pub struct Dal {
   file: File,
   page_size: u64,
   pub freelist: Freelist,
   pub meta: Meta,
}

impl Dal {
    pub fn build(path: &str) -> Result<Self, io::Error> {
        /// OpenOptions::new().create(true).write(true).read(true).open(path)?; //fs::File::create(path)?;

        
        let file = OpenOptions::new().write(true).read(true).open(path);// fs::File::open(path);
        
        let dal = match file {
            Ok(file) => {
                let mut a = Dal {
                    file,
                    page_size: u64::try_from(PAGE_SIZE).unwrap(),
                    freelist: Freelist::new(),
                    meta: Meta::new(),
                };

                a.read_meta()?;
                a.read_freelist()?;
                a
            },
            Err(error) => match error.kind() {
                ErrorKind::NotFound => match File::create(path) {
                    Ok(file) => { let mut a = Dal {
                        file,
                        page_size: u64::try_from(PAGE_SIZE).unwrap(),
                        freelist: Freelist::new(),
                        meta: Meta::new(),
                    };
    
                    a.meta.freelist_page = Some(a.freelist.get_next_page()); 
                    a.write_meta()?;
                    a.write_freelist()?;
                    a
                },
                    Err(e) => panic!("Problem creating the file: {:?}", e),
                },
                other_error => {
                    panic!("Problem opening the file: {:?}", other_error);
                }
            },
        };
        Ok(dal)
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

    fn write_to_disk(&mut self, page: &Page) -> Result<(), io::Error> {    
        let offset = page.num * self.page_size;
       // dal.file.seek(SeekFrom::Start(offset))?;
        self.file.write_all_at(&page.data, offset)?;
        Ok(())
        //dal.file.write(&page.data)
    }  

    pub fn write_page(&mut self, page: &Page) -> Result<(), io::Error> {    
        self.write_to_disk(page)?;
        //let offset = page.num * self.page_size;
       // dal.file.seek(SeekFrom::Start(offset))?;
        //self.file.write_all_at(&page.data, offset)?;

        //self.write_meta()?;
        self.write_freelist()?;
        Ok(())
        //dal.file.write(&page.data)
    }  
    
    fn read_meta(&mut self) -> Result<(), io::Error> {
        let page = self.read_page(META_PAGE_NUM).unwrap();
        let mut page_data = [0u8; 8];
        page_data.copy_from_slice(&page.data[..8]);
        self.meta.deserialize(&page_data);
        //let mut meta = Meta::new();
        //meta.deserialize(&page_data);
        Ok(())
    }
    
    fn write_meta(&mut self) -> Result<Page, io::Error> {
        let mut page = self.allocate_empty_page(META_PAGE_NUM);
        self.meta.serialize(&mut page.data);
        //
        //page.data = pad_zeroes(b"Sherlock Bones".to_owned());  
        self.write_to_disk(&page).unwrap();
        //
        Ok(page)
    }

    fn read_freelist(&mut self) -> Result<(), io::Error> {
        let page = self.read_page(self.meta.freelist_page.unwrap()).unwrap();
        //let mut freelist = Freelist::new();
        self.freelist.deserialize(&page.data);
        Ok(())
    }

    fn write_freelist(&mut self) -> Result<Page, io::Error> {
        let mut page = self.allocate_empty_page(self.meta.freelist_page.unwrap());
        self.freelist.serialize(&mut page.data);
        //
        //page.data = pad_zeroes(b"Sherlock Bones".to_owned());  
        self.write_to_disk(&page).unwrap();
        //
        Ok(page)
    }

    fn pad_zeroes<const A: usize>(arr: [u8; A]) -> [u8; PAGE_SIZE] {
        assert!(PAGE_SIZE >= A); //just for a nicer error message, adding #[track_caller] to the function may also be desirable
        let mut b = [0; PAGE_SIZE];
        //let a = vec![0,2];
        //b[..A].copy_from_slice(&a);
        b[..A].copy_from_slice(&arr);
        b
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



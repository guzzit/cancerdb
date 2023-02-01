use std::fs::{File, OpenOptions};
use std::io::{self, ErrorKind};
use std::os::unix::prelude::FileExt;

use crate::freelist::{Freelist, PageNumber};
use crate::constants::{META_PAGE_NUM, BYTES_IN_U64};
use crate::meta::Meta;
use crate::node::Node;


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
        // OpenOptions::new().create(true).write(true).read(true).open(path)?; //fs::File::create(path)?;

        let file = OpenOptions::new().write(true).read(true).open(path);// fs::File::open(path);
        
        let dal = match file {
            Ok(file) => {
                let mut a = Dal {
                    file,
                    page_size: u64::try_from(PAGE_SIZE).unwrap_or_else(|_| 1024 * 4),
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
                        page_size: u64::try_from(PAGE_SIZE).unwrap_or_else(|_| 1024 * 4),
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
        self.file.read_exact_at(&mut page.data, offset)?;

        Ok(page)
    }

    fn write_to_disk(&mut self, page: &Page) -> Result<(), io::Error> {    
        let offset = page.num * self.page_size;
        self.file.write_all_at(&page.data, offset)?;
        Ok(())
    }  

    pub fn write_page(&mut self, page: &Page) -> Result<(), io::Error> {    
        self.write_to_disk(page)?;
        self.write_freelist()?;

        Ok(())
    }  
    
    fn read_meta(&mut self) -> Result<(), io::Error> {
        let page = self.read_page(META_PAGE_NUM)?;
        let mut page_data = [0u8; BYTES_IN_U64];
        page_data.copy_from_slice(&page.data[..BYTES_IN_U64]);
        self.meta.deserialize(&page_data);
        
        Ok(())
    }
    
    fn write_meta(&mut self) -> Result<Page, io::Error> {
        let page = self.allocate_empty_page(META_PAGE_NUM); 
        let page_data_slice:&mut [u8;BYTES_IN_U64] = &mut page.data[..BYTES_IN_U64].try_into()
        //.map_err( |e| io::Error::new(ErrorKind::InvalidData, e))?;
        .map_err( |_| ErrorKind::InvalidData)?;
        self.meta.serialize(page_data_slice)?;
        self.write_to_disk(&page)?;
        
        Ok(page)
    }

    fn read_freelist(&mut self) -> Result<(), io::Error> {
        let freelist_page = self.meta.freelist_page.ok_or_else(|| ErrorKind::InvalidData)?;
        let page = self.read_page(freelist_page)?;
        self.freelist.deserialize(&page.data)?;

        Ok(())
    }

    fn write_freelist(&mut self) -> Result<Page, io::Error> {
        let freelist_page = self.meta.freelist_page.ok_or_else(|| ErrorKind::InvalidData)?;
        let mut page = self.allocate_empty_page(freelist_page);
        self.freelist.serialize(&mut page.data)?; 
        self.write_to_disk(&page)?;
        
        Ok(page)
    }

    pub fn get_node(&mut self, page_number: PageNumber) -> Result<Node, io::Error> {
        let mut page = self.read_page(page_number)?;
        let mut node = Node::build(self, page_number)?;
        node.deserialize(&mut page.data)?;
        Ok(node)
    }
    
    pub fn write_node(&mut self, node: &Node) -> Result<(), io::Error> {
        //let page_number = self.freelist.get_next_page();
        let mut page = self.allocate_empty_page(node.get_page_number());
        //let mut node = Node::new(self, page_number);
        node.serialize(&mut page.data)?;
        self.write_page(&page)?;

        Ok(())
    }

    fn delete_node(&mut self, page_number: PageNumber) {
        self.freelist.release_page(page_number);
    }
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

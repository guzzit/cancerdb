use std::fs::{File, OpenOptions};
use std::io::{self, ErrorKind};
use std::os::unix::prelude::FileExt;

use crate::freelist::{Freelist, PageNumber};
use crate::constants::{META_PAGE_NUM, BYTES_IN_U64, PAGE_SIZE, NODE_HEADER_SIZE};
use crate::meta::Meta;
use crate::node::Node;



/// Dal stands for Data Access Layer
pub struct Dal {
   file: File,
   page_size: u64,
   pub freelist: Freelist,
   pub meta: Meta,
   options: Options,
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
                    options: Options::default(),
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
                        options: Options::default(),
                    };
    
                    a.meta.root_page = Some(META_PAGE_NUM);
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
        let mut page_data = [0u8; BYTES_IN_U64*2];
        page_data.copy_from_slice(&page.data[..BYTES_IN_U64*2]);
        self.meta.deserialize(&page_data);
        
        Ok(())
    }
    
    fn write_meta(&mut self) -> Result<Page, io::Error> {
        let page = self.allocate_empty_page(META_PAGE_NUM); 
        let page_data_slice:&mut [u8;BYTES_IN_U64*2] = &mut page.data[..BYTES_IN_U64*2].try_into()
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

    pub fn new_node(&mut self) -> Result<Node, io::Error> {
        let pg_num = self.freelist.get_next_page();
        let new_node = Node::build( pg_num)?;
        Ok(new_node)
    }

    pub fn get_node(&mut self, page_number: PageNumber) -> Result<Node, io::Error> {
        let mut page = self.read_page(page_number)?;
        let mut node = Node::build(page_number)?;
        node.deserialize(&mut page.data)?;
        Ok(node)
    }

    pub fn get_nodes(&mut self, page_numbers: Vec<PageNumber>) -> Result<Vec<Node>, io::Error> {
        let mut nodes = Vec::new();
        for page_number in page_numbers.iter() {
            nodes.push(self.get_node(page_number.clone())?);
        }

        Ok(nodes)
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

    fn calculate_maximum_threshold(&self) -> Result<f32, io::Error> {
        let page_size:u16 = u16::try_from(self.options.page_size).map_err(|_| ErrorKind::InvalidData)?;
        let maximum_threshold = self.options.maximum_fill_percent * f32::try_from(page_size).map_err(|_| ErrorKind::InvalidData)?;
        Ok(maximum_threshold)
    }

    pub fn node_is_overpopulated(&self, node:&Node) -> Result<bool, io::Error> {
        let maximum_threshold = self.calculate_maximum_threshold()?;
        let node_size = node.calculate_node_size();
        //will u16 be enough to contain node_size?
        let node_size = u16::try_from(node_size).map_err(|_| ErrorKind::InvalidData)?;
        let node_size = f32::try_from(node_size).map_err(|_| ErrorKind::InvalidData)?;
        
        if node_size > maximum_threshold {
            return Ok(true);
        }
        else {
            return Ok(false);
        }
    }

    fn calculate_minimum_threshold(&mut self) -> Result<f32, io::Error> {
        let page_size:u16 = u16::try_from(self.options.page_size).map_err(|_| ErrorKind::InvalidData)?;
        let minimum_threshold = self.options.minimum_fill_percent * f32::try_from(page_size).map_err(|_| ErrorKind::InvalidData)?;
        Ok(minimum_threshold)
    }

    pub fn node_is_underpopulated(&mut self, node:&Node) -> Result<bool, io::Error> {
        let minimum_threshold = self.calculate_minimum_threshold()?;
        let node_size = node.calculate_node_size();
        //will u16 be enough to contain node_size?
        let node_size = u16::try_from(node_size).map_err(|_| ErrorKind::InvalidData)?;
        let node_size = f32::try_from(node_size).map_err(|_| ErrorKind::InvalidData)?;
        
        if node_size < minimum_threshold {
            return Ok(true);
        }
        else {
            return Ok(false);
        }
    }

    pub fn get_split_index(&mut self, node: &Node) -> Result<Option<usize>, io::Error> {
        let mut node_size = 0;
        node_size = node_size + NODE_HEADER_SIZE; 

        for (index, item) in node.items.iter().enumerate() {
            node_size = node_size + node.calculate_element_size(item);
            let node_size = u16::try_from(node_size).map_err(|_| ErrorKind::InvalidData)?;
            let node_size = f32::try_from(node_size).map_err(|_| ErrorKind::InvalidData)?;

            //if we have enough space in the page and the item is not the last in the vector
            if node_size > self.calculate_minimum_threshold()? && (index < node.items.len() - 1)  {
                return Ok(Some(index + 1));
            }
        }

        Ok(None)            
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

struct Options {
    page_size: usize,
    minimum_fill_percent: f32,
    maximum_fill_percent: f32,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            page_size: PAGE_SIZE,
            minimum_fill_percent: 0.5,
            maximum_fill_percent: 0.95,
        }
        
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

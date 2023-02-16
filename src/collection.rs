use crate::freelist::PageNumber;


struct Collection {
    name: Box<[u8]>,
    root: PageNumber,
}

impl Collection {
    fn new(name: Box<[u8]>, root: PageNumber) -> Self {
        Collection { 
            name, 
            root, 
        }
    }
}
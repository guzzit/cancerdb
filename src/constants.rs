pub const BYTE_IN_U8:usize = 1;
pub const BYTES_IN_U16:usize = 2;
pub const BYTES_IN_U64:usize = 8;
pub const META_PAGE_NUM:u64 = 0;
pub const PAGE_SIZE:usize = 1024 * 4;
pub const NODE_HEADER_SIZE:usize = 3;// header represents the total size of the is_leaf and item_length values. 
//is_leaf's size is 1 byte, while item_length's size is 2 bytes
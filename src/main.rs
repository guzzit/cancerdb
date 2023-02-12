use cancerdb::{dal::{Dal, PAGE_SIZE}, node::Node};

fn main() {
    //dbg!(std::mem::size_of::<Meta>());

    let db_path = "db.db";
    //let db_path = "mainTest";
    let dal = Dal::build(db_path);

    //use a match expression instead of an if statement
    if let Ok(mut dal) = dal {
        let mut node = dal.get_node(2).unwrap();
        let key = b"Key1".to_owned();
        let key: Box<[u8; 4]> = Box::new(key);
        let item = node.find_key(key, &mut dal).unwrap().unwrap();//add getter function for items

        //
        // let pg_num = dal.freelist.get_next_page();
        // let mut node = Node::build( pg_num).unwrap();
        // node.set_node_item();
        // dal.write_node(&node).unwrap();
        //

        // let page_num = dal.freelist.get_next_page();
        // let mut page = dal.allocate_empty_page(page_num);
        // page.data = pad_zeroes(b"Be yourself no matter what they say".to_owned());  
        // dal.write_page(&page).unwrap();
        
    }
    else{
        panic!("error!!!!!!!!!!!!!!!")
    }

}

fn pad_zeroes<const A: usize>(arr: [u8; A]) -> [u8; PAGE_SIZE] {
    //instead of an assert here and panicking, return an error
    assert!(PAGE_SIZE >= A); //just for a nicer error message, adding #[track_caller] to the function may also be desirable
    let mut b = [0; PAGE_SIZE];
    b[..A].copy_from_slice(&arr);
    b
}

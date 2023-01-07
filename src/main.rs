use libradb::{dal::{Dal, Page}, freelist::{self, Freelist}};


fn main() {
    let db_path = "db.db";
    let dal = Dal::build(db_path);

    if let Ok(mut dal) = dal {
        let mut page = Page::new(&dal);
        let mut freelist = Freelist::new();
        let page_num = freelist.get_next_page();
        page.num = Some(page_num);
        //let a = b"jrk".to_owned();

        page.data = pad_zeroes(b"jrk".to_owned());  
        dal.write_page(&page);

    }
    else{
        panic!("error!!!!!!!!!!!!!!!")
    }

}

fn pad_zeroes<const A: usize>(arr: [u8; A]) -> [u8; 4096] {
    const PAGE_SIZE:usize = 1024 * 4;
    assert!(PAGE_SIZE >= A); //just for a nicer error message, adding #[track_caller] to the function may also be desirable
    let mut b = [0; PAGE_SIZE];
    b[..A].copy_from_slice(&arr);
    //b[A..].copy_from_slice(&arr);
    b
}

// fn pad_zeroes<const A: usize, const B: usize>(arr: [u8; A]) -> [u8; B] {
//     assert!(B >= A); //just for a nicer error message, adding #[track_caller] to the function may also be desirable
//     let mut b = [0; B];
//     //b[..A].copy_from_slice(&arr);
//     b[A..].copy_from_slice(&arr);
//     b
// }
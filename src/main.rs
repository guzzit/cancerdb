use libradb::{dal::{Dal, PAGE_SIZE}};
use libradb::meta::Meta;


fn main() {
    //dbg!(std::mem::size_of::<Meta>());

    let db_path = "db.db";
    let dal = Dal::build(db_path);

    if let Ok(mut dal) = dal {
        //let mut freelist = Freelist::new();
        let page_num = dal.freelist.get_next_page();
        let mut page = dal.allocate_empty_page(page_num);

        //page.num = Some(page_num);
        //let a = b"jrk".to_owned();

        page.data = pad_zeroes(b"No shit Sherlock".to_owned());  
        dal.write_page(&page).unwrap();
        

    }
    else{
        panic!("error!!!!!!!!!!!!!!!")
    }

}

//fn pad_zeroes<const A: usize>(arr: [u8; A]) -> [u8; A] {

fn pad_zeroes<const A: usize>(arr: [u8; A]) -> [u8; PAGE_SIZE] {
    assert!(PAGE_SIZE >= A); //just for a nicer error message, adding #[track_caller] to the function may also be desirable
    let mut b = [0; PAGE_SIZE];
    //let a = vec![0,2];
    //b[..A].copy_from_slice(&a);
    b[..A].copy_from_slice(&arr);
    b
}

// fn pad_zeroes<const A: usize, const B: usize>(arr: [u8; A]) -> [u8; B] {
//     assert!(B >= A); //just for a nicer error message, adding #[track_caller] to the function may also be desirable
//     let mut b = [0; B];
//     //b[..A].copy_from_slice(&arr);
//     b[A..].copy_from_slice(&arr);
//     b
// }
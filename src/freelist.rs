
type Page_Num = u64;

pub struct Freelist {
   max_page: Page_Num,
   released_pages: Vec<u64>,
}

impl Freelist {
    pub fn new() -> Self {
        Freelist {
            max_page:0,
            released_pages: Vec::new(),
        }
    }
     
    pub fn get_next_page(&mut self) -> Page_Num {
        let next_page = self.released_pages.iter().take(1).next();
    
        if let Some(page) = next_page {
            page.to_owned()
        }
        else {
            self.max_page = self.max_page.saturating_add(1);
            self.max_page
        }
            
    }

    fn release_page(&mut self, page_num: Page_Num) {
        self.released_pages.push(page_num);
        
    }
}
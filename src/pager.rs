use std::{cmp::max, collections::VecDeque};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PageInfo {
    pub cursor: Option<u64>,
    pub page_size: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Pager {
    pub prev: Option<u64>,
    pub next: Option<u64>,
    pub total: Option<u64>,
}
pub trait Paginator: Sized {
    fn get_pager<T: Container>(&self, data: &mut T) -> Pager;
    fn next_page(&self, pager: &Pager) -> Option<Self>;
    fn prev_page(&self, pager: &Pager) -> Option<Self>;
}

pub trait Container {
    fn pop(&mut self);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> Container for VecDeque<T> {
    fn pop(&mut self) {
        self.pop_back();
    }

    fn len(&self) -> usize {
        self.len()
    }
}

impl<T> Container for Vec<T> {
    fn pop(&mut self) {
        self.pop();
    }

    fn len(&self) -> usize {
        self.len()
    }
}

impl Paginator for PageInfo {
    fn get_pager<T: Container>(&self, data: &mut T) -> Pager {
        let prev = match self.cursor {
            Some(v) if v > 0 => Some(max(0, v - self.page_size)),
            _ => None,
        };

        let has_next = data.len() as u64 > self.page_size;
        let next = if has_next {
            data.pop();
            Some(self.cursor.unwrap_or(0) + self.page_size)
        } else {
            None
        };

        Pager {
            prev,
            next,
            total: None,
        }
    }

    fn next_page(&self, pager: &Pager) -> Option<Self> {
        if pager.next.is_some() {
            Some(PageInfo {
                cursor: pager.next,
                page_size: self.page_size,
            })
        } else {
            None
        }
    }

    fn prev_page(&self, pager: &Pager) -> Option<Self> {
        if pager.prev.is_some() {
            Some(PageInfo {
                cursor: pager.prev,
                page_size: self.page_size,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
pub mod pager_test_utils {
    use std::collections::VecDeque;
    pub struct TestId(u64);

    pub fn generate_test_ids(start: u64, end: u64) -> VecDeque<TestId> {
        (start..=end).map(TestId).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paginator_should_work() {
        // first page
        let page = PageInfo {
            cursor: None,
            page_size: 10,
        };

        // assume we got 11 items from db
        let mut items = pager_test_utils::generate_test_ids(1, 11);
        let pager = page.get_pager(&mut items);
        assert!(pager.prev.is_none());
        assert_eq!(pager.next, Some(10));

        {
            let prev_page = page.prev_page(&pager);
            assert!(prev_page.is_none());
        }

        // second page
        let page = page.next_page(&pager).unwrap();
        let mut items = pager_test_utils::generate_test_ids(11, 21);
        let pager = page.get_pager(&mut items);
        assert_eq!(pager.prev, Some(0));
        assert_eq!(pager.next, Some(20));

        {
            let prev_page = page.prev_page(&pager);
            assert_eq!(prev_page.unwrap().cursor, Some(0));
        }

        // third page
        let page = page.next_page(&pager).unwrap();
        let mut items = pager_test_utils::generate_test_ids(21, 25);
        let pager = page.get_pager(&mut items);
        assert_eq!(pager.prev, Some(10));
        assert!(pager.next.is_none());

        {
            let prev_page = page.prev_page(&pager);
            assert_eq!(prev_page.unwrap().cursor, Some(10));
        }
    }
}

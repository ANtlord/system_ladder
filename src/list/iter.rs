use super::item::Link;


pub struct Iter<'a, T: 'a> {
    pub current: &'a Link<T>,
    pub is_next_called: bool,
}

impl<'a, T: 'a> Iterator for Iter<'a, T> {
    type Item = &'a T;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            return None;
        }
        if self.is_next_called {
            self.current = self.current.next()?;
        }
        self.is_next_called = true;
        Some(self.current.get()?.get())
    }
}

use super::item::Link;

pub struct Iter<'a, T: 'a> {
    current: &'a Link<T>,
    is_next_called: bool,
}

impl<'a, T: 'a> Iter<'a, T> {
    pub fn new(link: &'a Link<T>) -> Self {
        Iter {
            current: link,
            is_next_called: false,
        }
    }
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

pub struct Cursor<'a, T: 'a> {
    current_link: &'a Link<T>,
}

impl<'a, T: 'a> Cursor<'a, T> {
    pub fn new(link: &'a Link<T>) -> Self {
        Cursor { current_link: link }
    }

    pub fn move_next(&mut self) -> Option<&mut T> {
        self.current_link = self.current_link.next()?;
        self.current()
    }

    pub fn current(&mut self) -> Option<&mut T> {
        let ptr = self.current_link.get_ptr();
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { (*ptr).get_mut() })
    }
}

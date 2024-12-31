pub struct Cursor {
    pub offset: usize,
}

impl Cursor {
    pub fn new() -> Self {
        Cursor { offset: 0 }
    }

    pub fn insert_at_cursor(&self, str: &mut String, c: char) {
        str.insert(str.len() - self.offset, c);
    }

    pub fn remove_at_cursor(&self, str: &mut String) -> bool {
        let i: i8 = str.len() as i8 - self.offset as i8 - 1;
        if i >= 0 {
            str.remove(i as usize);
            return true;
        }
        return false;
    }

    pub fn c_left(&mut self, max: usize) -> bool {
        if self.offset < max {
            self.offset += 1;
            return true;
        }
        return false;
    }

    pub fn c_right(&mut self) -> bool {
        if self.offset > 0 {
            self.offset -= 1;
            return true;
        }
        return false;
    }

    pub fn reset(&mut self) {
        self.offset = 0;
    }
}

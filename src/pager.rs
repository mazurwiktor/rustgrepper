#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum Key {
    Backspace,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    Tab,
    Enter,
    PageUp,
    PageDown,
    Delete,
    Insert,
    F(u8),
    Char(char),
    Alt(char),
    Ctrl(char),
    Null,
    Esc,
}

pub trait TermOperations {
    fn term_size(&mut self) -> (usize, usize);
    fn print(&mut self, &str);
    fn clear(&mut self);
    fn mv_cursor(&mut self, (usize, usize));
    fn cursor_pos(&mut self) -> (usize, usize);
    fn clear_line(&mut self);
    fn clear_line_from(&mut self, (usize, usize));
    fn input_key(&mut self) -> Key;

    fn print_buffer(&mut self, buffer: &str, offset: usize) {
        let end_height = self.term_size().1 - offset;
        let buffer_len = buffer.len();
        let lines_to_end = end_height - self.cursor_pos().1;
        let limit = lines_to_end * self.term_size().0 - self.cursor_pos().0;
        if buffer_len > limit {
            self.print(&buffer[..(limit) - 1]);
        } else {
            self.print(buffer);
        }
    }
}
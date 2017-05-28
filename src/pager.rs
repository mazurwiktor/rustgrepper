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

    fn top_leftofer<'a>(&mut self, line: &'a str) -> Option<&'a str> {
        let (max_x, _) = self.term_size();
        if line.len() <= max_x {
            None
        } else {
            Some(&line[max_x..])
        }
    }
}

#[allow(dead_code)]
struct PagerMock {
    size: (usize, usize),
    cursor_pos: (usize, usize),
    input_key: Key,
}

#[allow(dead_code)]
impl PagerMock {
    fn default() -> Self {
        PagerMock {
            size: (10, 10),
            cursor_pos: (0, 0),
            input_key: Key::Enter,
        }
    }
    fn with_size(mut self, size: (usize, usize)) -> Self {
        self.size = size;
        self
    }
    fn with_cursor_pos(mut self, pos: (usize, usize)) -> Self {
        self.cursor_pos = pos;
        self
    }
}

#[allow(unused)]
impl TermOperations for PagerMock {
    fn term_size(&mut self) -> (usize, usize) {
        self.size
    }
    fn print(&mut self, text: &str) {}
    fn clear(&mut self) {}
    fn mv_cursor(&mut self, pos: (usize, usize)) {}
    fn cursor_pos(&mut self) -> (usize, usize) {
        self.cursor_pos
    }
    fn clear_line(&mut self) {}
    fn clear_line_from(&mut self, pos: (usize, usize)) {}
    fn input_key(&mut self) -> Key {
        self.input_key
    }
}

#[test]
fn top_leftofer_test_none() {
    let mut pager = PagerMock::default().with_size((10, 10)).with_cursor_pos((0, 0));
    let test_line = String::from("a")
        .chars()
        .cycle()
        .take(10)
        .collect::<String>();
    assert_eq!(test_line.len(), 10);
    assert_eq!(&pager.top_leftofer(&test_line), &None);
}

#[test]
fn top_leftofer_test_overflow() {
    let mut pager = PagerMock::default().with_size((10, 10)).with_cursor_pos((0, 0));
    let test_line = String::from("a")
        .chars()
        .cycle()
        .take(11)
        .collect::<String>();
    assert_eq!(test_line.len(), 11);
    assert_eq!(&pager.top_leftofer(&test_line), &Some("a"));
}
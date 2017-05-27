use utils;
use pager::*;
use greps::*;
use ncurses::*;

pub struct CursesPager {
    userbar_height: i32,
}

impl CursesPager {
    pub fn new() -> CursesPager {
        CursesPager { userbar_height: 2 }
    }

    pub fn initialize(&mut self) {
        initscr();
        keypad(stdscr(), true);
        noecho();
        //start_color();
    }

    pub fn print_logs(&mut self,
                      lines: &[utils::Line],
                      greps: Vec<utils::DecorationPattern>)
                      -> usize {
        let mut printed_lines = 0;
        let end_height = (self.term_size().1 as i32) - self.userbar_height;

        for line in lines {
            if self.cursor_pos().1 as i32 >= end_height {
                return printed_lines;
            } else {
                for word in &line.decorate(greps.clone()) {
                    self.print_decoration(word);
                }
            }
            let current_pos = self.cursor_pos();
            self.clear_line_from(current_pos);
            if self.cursor_pos().1 as i32 <= end_height - 1 {
                // let (_, curr_y) = self.cursor_pos();
                // self.mv_cursor((0, curr_y + 1));
                printed_lines += 1;
            }
        }

        while self.cursor_pos().1 as i32 != (self.term_size().1 as i32 - self.userbar_height) {
            let (_, curr_y) = self.cursor_pos();
            self.print("~");
            let current_pos = self.cursor_pos();
            self.clear_line_from(current_pos);
            self.mv_cursor((0, curr_y + 1));
        }

        return printed_lines;
    }

    fn print_decoration(&mut self, decoration: &utils::Decorations) {
        match decoration {
            &utils::Decorations::None(ref buffer) => {
                let offset = self.userbar_height as usize;
                self.print_buffer(buffer, offset);
            }
            &utils::Decorations::Some(ref attrs, ref buffer) => {
                for attr in attrs {
                    match attr {
                        &utils::Attribute::Inverse => {
                            attron(A_REVERSE());
                        }
                        &utils::Attribute::Red => {}
                        _ => {}    
                    }
                }
                let offset = self.userbar_height as usize;
                self.print_buffer(buffer, offset);
                for attr in attrs {
                    match attr {
                        &utils::Attribute::Inverse => {
                            attroff(A_REVERSE());
                        } 
                        _ => {}    
                    }
                }
            }
        }
    }

    pub fn status(&mut self, greps: &Greps) {
        self.clear_line();
        let mut idx = 0;
        let selected = greps.selected;
        for grep in &greps.greps {
            if idx == selected {
                attron(A_REVERSE());
                self.print(&grep.patern);
                attroff(A_REVERSE());
            } else {
                self.print(&grep.patern);
            }
            self.print(" ");
            idx += 1;
        }
        let pos = self.cursor_pos();
        self.clear_line_from(pos);
    }
}

impl Drop for CursesPager {
    fn drop(&mut self) {
        let last_line = (self.term_size().1 - 1) as usize;
        self.mv_cursor((last_line, 0));
        endwin();
    }
}

impl TermOperations for CursesPager {
    fn term_size(&mut self) -> (usize, usize) {
        let mut x = 0;
        let mut y = 0;
        getmaxyx(stdscr(), &mut y, &mut x);
        (x as usize, y as usize)
    }

    fn print(&mut self, slice: &str) {
        printw(slice);
    }

    fn clear(&mut self) {
        clear();
    }

    fn mv_cursor(&mut self, pos: (usize, usize)) {
        mv(pos.1 as i32, pos.0 as i32);
    }

    fn cursor_pos(&mut self) -> (usize, usize) {
        let mut x = 0;
        let mut y = 0;
        getyx(stdscr(), &mut y, &mut x);
        (x as usize, y as usize)
    }

    fn clear_line(&mut self) {
        let (max_x, _) = self.term_size();
        let (_, y) = self.cursor_pos();

        self.mv_cursor((0, y as usize));
        let mut clear_line = String::new();
        for _ in 0..max_x {
            clear_line.push(' ');
        }
        self.print(&clear_line);
        self.mv_cursor((0, y as usize));
    }

    fn clear_line_from(&mut self, pos: (usize, usize)) {
        let (max_x, _) = self.term_size();
        let (x, _) = pos;
        let mut clear_line = String::new();
        for _ in (x - 1)..max_x - 1 {
            clear_line.push(' ');
        }
        self.print(&clear_line);
    }

    fn input_key(&mut self) -> Key {
        match getch() as u8 {
            27 => Key::Esc,
            2 => Key::Down,
            3 => Key::Up,
            4 => Key::Left,
            5 => Key::Right,
            9 => Key::Tab,
            10 => Key::Enter,
            32 => Key::Ctrl('w'),
            ch => Key::Char(ch as char),
        }
    }
}
use utils;

use ncurses::*;

pub struct Pager {
    screen_width: i32,
    screen_height: i32,
    curr_x: i32,
    curr_y: i32,
}

impl Pager {
    pub fn new() -> Pager {
        Pager {
            screen_width: 0,
            screen_height: 0,
            curr_x: 0,
            curr_y: 0,
        }
    }

    pub fn initialize(&mut self) {
        initscr();
        keypad(stdscr(), true);
        noecho();
        //start_color();
        getmaxyx(stdscr(), &mut self.screen_height, &mut self.screen_width);
    }
    pub fn execute_logs(&mut self,
                        lines: &[utils::Line],
                        greps: Vec<utils::DecorationPattern>)
                        -> usize {
        let mut scrolled = 0;

        for line in lines {
            getyx(stdscr(), &mut self.curr_y, &mut self.curr_x);
            let end_height = self.screen_height - 2;
            if self.curr_y >= end_height {
                return scrolled;
            } else {
                for word in &line.decorate(greps.clone()) {
                    self.print_decoration(word);
                }
            }
            if self.curr_y <= end_height - 1 {
                printw("\n");
                scrolled += 1;
            }
        }

        while self.curr_y != (self.screen_height - 3) {
            getyx(stdscr(), &mut self.curr_y, &mut self.curr_x);
            printw("~\n");
        }

        return scrolled;
    }

    fn print_decoration(&self, decoration: &utils::Decorations) {
        match decoration {
            &utils::Decorations::None(ref buffer) => {
                self.print_buffer(buffer);
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
                self.print_buffer(buffer);
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

    fn print_buffer(&self, buffer: &str) {
        let end_height = self.screen_height - 2;
        let buffer_len = buffer.len();
        let lines_to_end = end_height - self.curr_y;
        let limit = lines_to_end * self.screen_width;
        if buffer_len > limit as usize {
            printw(&buffer[..(limit as usize) - 1]);
        } else {
            printw(buffer);
        }
    }
}

impl Drop for Pager {
    fn drop(&mut self) {
        mv(self.screen_height - 1, 0);
        endwin();
    }
}
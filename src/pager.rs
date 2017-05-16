use utils;

use ncurses::*;
use greps::*;

pub struct Pager {
    screen_width: i32,
    screen_height: i32,
    curr_x: i32,
    curr_y: i32,
    userbar_height: i32,
}

impl Pager {
    pub fn new() -> Pager {
        Pager {
            screen_width: 0,
            screen_height: 0,
            curr_x: 0,
            curr_y: 0,
            userbar_height: 2
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
        let mut printed_lines = 0;

        for line in lines {
            self.update_cursor_position();
            let end_height = self.screen_height - self.userbar_height;
            if self.curr_y >= end_height {
                return printed_lines;
            } else {
                for word in &line.decorate(greps.clone()) {
                    self.update_cursor_position();
                    self.print_decoration(word);
                }
            }
            if self.curr_y <= end_height - 1 {
                printw("\n");
                printed_lines += 1;
            }
        }

        while self.curr_y != (self.screen_height - self.userbar_height + 1) {
            self.update_cursor_position();
            printw("~\n");
        }

        return printed_lines;
    }

    fn update_cursor_position(&mut self) {
        getyx(stdscr(), &mut self.curr_y, &mut self.curr_x);
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
        let end_height = self.screen_height - self.userbar_height;
        let buffer_len = buffer.len();
        let lines_to_end = end_height - self.curr_y;
        let limit = lines_to_end * self.screen_width - self.curr_x;
        if buffer_len > limit as usize {
            printw(&buffer[..(limit as usize) - 1]);
        } else {
            printw(buffer);
        }
    }

    pub fn status(&self, greps: &Greps) {
        clear_current_line();
        let mut idx = 0;
        let selected = greps.selected;
        for grep in &greps.greps {
            if idx == selected {
                attron(A_REVERSE());
                printw(&grep.patern);
                attroff(A_REVERSE());
            } else {
                printw(&grep.patern);
            }
            printw(" ");
            idx += 1;
        }
        fill_current_line();
    }
}

impl Drop for Pager {
    fn drop(&mut self) {
        mv(self.screen_height - 1, 0);
        endwin();
    }
}

pub fn fill_current_line() {
    let mut x = 0;
    let mut y = 0;
    let mut max_y = 0;
    let mut max_x = 0;
    getyx(stdscr(), &mut y, &mut x);
    getmaxyx(stdscr(), &mut max_y, &mut max_x);
    let mut clear_line = String::new();
    for _ in (x - 1)..max_x {
        clear_line.push(' ');
    }
    printw(&clear_line);
}

pub fn clear_current_line() {
    let mut x = 0;
    let mut y = 0;
    let mut max_y = 0;
    let mut max_x = 0;
    getyx(stdscr(), &mut y, &mut x);
    getmaxyx(stdscr(), &mut max_y, &mut max_x);
    mv(y, 0);
    let mut clear_line = String::new();
    for _ in 0..max_x {
        clear_line.push(' ');
    }
    printw(&clear_line);
    mv(y, 0);
}
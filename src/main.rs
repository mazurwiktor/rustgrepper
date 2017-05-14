extern crate ncurses;
extern crate regex;

use ncurses::*;
use regex::Regex;

use std::collections::HashMap;

mod utils;

struct Pager {
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
            if self.curr_y < end_height - 2 {
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
                        },
                        &utils::Attribute::Red => {
                            
                        }
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

struct Grep<'a> {
    patern: String,
    line_index: usize,
    lines: Vec<utils::Line<'a>>,
}

struct Greps<'a> {
    greps: Vec<Grep<'a>>,
    current_search_pattern: String,
    decorations: HashMap<String, utils::DecorationPattern>,
    selected: usize,
}

impl<'a> Greps<'a> {
    fn new(lines: Vec<utils::Line<'a>>) -> Self {
        let greps = vec![Grep {
                             patern: "ROOT".to_string(),
                             line_index: 0,
                             lines: lines,
                         }];
        Greps {
            greps: greps,
            current_search_pattern: "".to_string(),
            decorations: HashMap::new(),
            selected: 0,
        }
    }

    fn current_grep(&self) -> &Grep {
        &self.greps[self.selected]
    }

    fn change_current_line_index(&mut self, index: usize) {
        self.greps[self.selected].line_index = index;
    }

    fn apply_search_patern(&mut self, pattern: &str) {
        self.decorations.remove(&self.current_search_pattern);
        self.current_search_pattern = pattern.to_string();
        self.decorations
            .insert(self.current_search_pattern.clone(),
                    utils::DecorationPattern::from_single_attr(utils::Attribute::Inverse,
                                                               &self.current_search_pattern));
    }

    fn decorations(&self) -> Vec<utils::DecorationPattern> {
        self.decorations
            .clone()
            .into_iter()
            .map(|(_, v)| v)
            .collect()
    }

    fn status(&self) {
        clear_current_line();
        let mut idx = 0;
        for grep in &self.greps {
            if idx == self.selected {
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

    fn new_grep(&mut self, patern: &str) {
        let cur_patern = self.current_grep().patern.clone();
        let re = Regex::new(&patern).unwrap();
        let new_lines = self.greps[self.selected]
            .lines
            .clone()
            .into_iter()
            .filter(|l| re.is_match(&l.buffer))
            .collect();

        self.greps.push(Grep {
                            patern: cur_patern + " > " + patern,
                            line_index: 0,
                            lines: new_lines,
                        });
        self.selected = self.greps.len() - 1;
    }

    fn select_one_to_left(&mut self) {
        if self.selected >= 1 {
            self.selected = self.selected - 1;
        }
    }
    fn select_one_to_right(&mut self) {
        if self.selected < self.greps.len() - 1 {
            self.selected = self.selected + 1;
        }
    }

    fn close_grep(&mut self) {
        if self.selected != 0 {
            let curent = self.selected;
            self.selected = self.selected - 1;
            self.greps.remove(curent);
        }
    }
}

fn main() {
    let buffer = utils::test_buffer_from_file();

    let mut pager = Pager::new();
    pager.initialize();
    let mut greps = Greps::new(utils::Text::from(&buffer).lines);

    let capacity = pager.execute_logs(&greps.current_grep().lines, greps.decorations());

    loop {
        let index = greps.current_grep().line_index;
        pager.execute_logs(&greps.current_grep().lines[index..], greps.decorations());

        greps.status();
        match prompt(PromptMode::Visual) {
            Prompt::Exit => break,
            Prompt::SearchPattern(pat) => {
                greps.apply_search_patern(&pat);
            }
            Prompt::GrepPattern(pat) => {
                 greps.new_grep(&pat);
                clear();
            }
            Prompt::GrepLeft => greps.select_one_to_left(),
            Prompt::GrepRight => greps.select_one_to_right(),
            Prompt::CloseGrep => greps.close_grep(),
            Prompt::SingleLineDown => {
                let last_index = greps.current_grep().lines.len() - 1;
                if index < last_index - capacity + 2 {
                    greps.change_current_line_index(index + 1)
                }
            }
            Prompt::SingleLineUp => {
                if index > 0 {
                    greps.change_current_line_index(index - 1);
                }
            }
            Prompt::ScrollTop => greps.change_current_line_index(0),
            Prompt::NextPage => greps.change_current_line_index(index + capacity),
            Prompt::ScrollBottom => {
                let last_index = greps.current_grep().lines.len() - capacity + 1;
                greps.change_current_line_index(last_index);
            }
            //_ => {}
        }

        mv(0, 0);
    }
}

#[allow(unused)]
enum Prompt {
    Exit,
    NextPage,
    ScrollTop,
    ScrollBottom,
    SingleLineDown,
    SingleLineUp,
    SearchPattern(String),
    GrepPattern(String),
    GrepLeft,
    GrepRight,
    CloseGrep,
}

enum PromptMode {
    Visual,
    Search,
    Grep,
    Command,
}

#[allow(unused)]
enum KeyCodes {
    ESC = 27,
    Tab = 9,
    Enter = 10,
    Down = 2,
    Up = 3,
    Left = 4,
    Right = 5,
    CtrlPlusW = 23,
}

fn prompt(mode: PromptMode) -> Prompt {
    let mut typed = String::from("");
    let mut tabbed = String::from("");

    //let commands = vec!["close"];
    clear_current_line();
    match mode {
        PromptMode::Visual => {
            printw(":");
        }
        PromptMode::Search => {
            printw("/");
        }
        PromptMode::Grep => {
            printw("&/");
        }
        PromptMode::Command => {
            printw("#");
        }
    }
    loop {
        match mode {
            PromptMode::Visual => {
                match getch() as u8 as char {
                    ' ' => {
                        return Prompt::NextPage;
                    }
                    'q' => return Prompt::Exit,
                    '/' => {
                        return prompt(PromptMode::Search);
                    }
                    '&' => {
                        return prompt(PromptMode::Grep);
                    }
                    '#' => {
                        return prompt(PromptMode::Command);
                    }
                    x if x == KeyCodes::Down as u8 as char => {
                        return Prompt::SingleLineDown;
                    }
                    x if x == KeyCodes::Up as u8 as char => {
                        return Prompt::SingleLineUp;
                    }
                    x if x == KeyCodes::Left as u8 as char => {
                        return Prompt::GrepLeft;
                    }
                    x if x == KeyCodes::Right as u8 as char => {
                        return Prompt::GrepRight;
                    }
                    'g' => {
                        return Prompt::ScrollTop;
                    }
                    'G' => {
                        return Prompt::ScrollBottom;
                    }
                    x if x == KeyCodes::CtrlPlusW as u8 as char => {
                        return Prompt::CloseGrep;
                    }
                    code => {
                        printw(&format!("{}", code as u8));
                    }
                }
            }
            PromptMode::Search => {
                match getch() as u8 {
                    x if x == KeyCodes::ESC as u8 => {
                        return prompt(PromptMode::Visual);
                    }
                    x if x == KeyCodes::Enter as u8 => {
                        return Prompt::SearchPattern(typed);
                    }
                    code => {
                        typed.push(code as char);
                        addch(code as chtype);
                    }
                }
            }
            PromptMode::Grep => {
                match getch() as u8 {
                    x if x == KeyCodes::ESC as u8 => {
                        return prompt(PromptMode::Visual);
                    }
                    x if x == KeyCodes::Enter as u8 => {
                        attroff(A_BOLD());
                        return Prompt::GrepPattern(typed);
                    }
                    code => {
                        typed.push(code as char);
                        addch(code as chtype);
                    }
                }
            }
            PromptMode::Command => {
                match getch() as u8 {
                    x if x == KeyCodes::ESC as u8 => {
                        return prompt(PromptMode::Visual);
                    }
                    x if x == KeyCodes::Tab as u8 => {
                        // find in commands, since is only one return close ^^
                        tabbed = "close".to_string();
                        clear_current_line();
                        printw(&format!("#{}", tabbed));
                    }
                    x if x == KeyCodes::Enter as u8 => {
                        let result = if tabbed == "".to_string() {
                            typed.clone()
                        } else {
                            tabbed.clone()
                        };
                        match result.as_ref() {
                            "close" => return Prompt::CloseGrep,
                            _ => {}
                        }
                        //return Prompt::GrepPattern(typed);
                    }
                    code => {
                        typed.push(code as char);
                        addch(code as chtype);
                    }
                }
            }
        }
    }
}

fn fill_current_line() {
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

fn clear_current_line() {
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
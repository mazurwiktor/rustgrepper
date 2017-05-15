use ncurses::*;
use pager::clear_current_line;

#[allow(unused)]
pub enum Prompt {
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
    NextSearch,
    PrevSearch,
}

pub enum PromptMode {
    Visual,
    Search,
    Grep,
    Command,
}

#[allow(unused)]
pub enum KeyCodes {
    ESC = 27,
    Tab = 9,
    Enter = 10,
    Down = 2,
    Up = 3,
    Left = 4,
    Right = 5,
    CtrlPlusW = 23,
}

pub fn prompt(mode: PromptMode) -> Prompt {
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
                    'n' => {
                        return Prompt::NextSearch;
                    }
                    'N' => {
                        return Prompt::PrevSearch;
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
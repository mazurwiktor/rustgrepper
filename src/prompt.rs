use pager::*;

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

pub fn prompt<P>(pager: &mut P, mode: PromptMode) -> Prompt
    where P: TermOperations
{
    let mut typed = String::from("");
    let mut tabbed = String::from("");

    pager.clear_line();
    match mode {
        PromptMode::Visual => {
            pager.print(":");
        }
        PromptMode::Search => {
            pager.print("/");
        }
        PromptMode::Grep => {
            pager.print("&/");
        }
        PromptMode::Command => {
            pager.print("#");
        }
    }
    loop {
        match mode {
            PromptMode::Visual => {
                match pager.input_key() {
                    Key::Char(ch) => {
                        match ch {
                            ' ' => {
                                return Prompt::NextPage;
                            }
                            'q' => return Prompt::Exit,
                            '/' => {
                                return prompt(pager, PromptMode::Search);
                            }
                            '&' => {
                                return prompt(pager, PromptMode::Grep);
                            }
                            '#' => {
                                return prompt(pager, PromptMode::Command);
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
                            },
                            c => {
                                pager.print(&format!("CH: {} code: {} ", c, c as u8));
                            },
                        }
                    },
                    Key::Down => return Prompt::SingleLineDown,
                    Key::Up => return Prompt::SingleLineUp,
                    Key::Left => return Prompt::GrepLeft, 
                    Key::Right => return Prompt::GrepRight,
                    Key::Ctrl(ch) => {
                        match ch {
                            'w' => return Prompt::CloseGrep,
                            _ => {}
                        }
                    },
                    _ => {}
                }
            }
            PromptMode::Search | PromptMode::Grep => {
                match pager.input_key() {
                    Key::Esc => {
                        return prompt(pager, PromptMode::Visual);
                    }
                    Key::Enter => {
                        match mode {
                            PromptMode::Search => return Prompt::SearchPattern(typed),
                            PromptMode::Grep => return Prompt::GrepPattern(typed),
                            _ => {}
                        }
                    }
                    Key::Char(ch) => {
                        typed.push(ch);
                        pager.print(&format!("{}", ch));
                    }
                    _ => {}
                }
            }
            PromptMode::Command => {
                match pager.input_key(){
                    Key::Esc => {
                        return prompt(pager, PromptMode::Visual);
                    }
                    Key::Tab => {
                        // find in commands, since is only one return close ^^
                        tabbed = "close".to_string();
                        pager.clear_line();
                        pager.print(&format!("#{}", tabbed));
                    }
                    Key::Enter => {
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
                    Key::Char(ch) => {
                        typed.push(ch);
                        pager.print(&format!("{}", ch));
                    }
                    _ => {}
                }
            }
        }
    }
}
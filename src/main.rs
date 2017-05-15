extern crate ncurses;
extern crate regex;

use ncurses::*;
use regex::Regex;

use std::collections::HashMap;

mod utils;
mod prompt;
mod pager;
use prompt::*;
use pager::*;

struct Grep<'a> {
    patern: String,
    line_index: usize,
    search_lines_idxs: Vec<usize>,
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
                             search_lines_idxs: Vec::new(),
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
        let re = Regex::new(&pattern).unwrap();

        let search_lines_idxs = self.greps[self.selected]
            .lines
            .iter()
            .enumerate()
            .filter(|&(_, ref l)| re.is_match(l.buffer))
            .map(|(idx, _)| idx)
            .collect();
        self.greps[self.selected].search_lines_idxs = search_lines_idxs;
        self.greps[self.selected].line_index = self.greps[self.selected].search_lines_idxs[0];
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
        // printw(&format!("DEBUG: [greps] cur line: {} indexes {:?}",
        //                 self.current_grep().line_index,
        //                 self.current_grep().search_lines_idxs));
        fill_current_line();
    }

    fn new_grep(&mut self, patern: &str) {
        let cur_patern = self.current_grep().patern.clone();
        let re = Regex::new(&patern).unwrap();
        let mut search_lines_idxs = Vec::new();
        let new_lines = self.greps[self.selected]
            .lines
            .clone()
            .into_iter()
            .enumerate()
            .filter(|&(_, ref l)| re.is_match(l.buffer))
            .map(|(idx, l)| {
                     search_lines_idxs.push(idx);
                     l
                 })
            .collect();

        self.greps.push(Grep {
                            patern: cur_patern + " > " + patern,
                            line_index: 0,
                            search_lines_idxs: search_lines_idxs,
                            lines: new_lines,
                        });
        self.selected = self.greps.len() - 1;
    }

    fn modify_search<F>(&mut self, modifier: F)
        where F: Fn(usize) -> usize
    {
        let search_lines_idxs = self.greps[self.selected].search_lines_idxs.clone();
        let current_line_idx = self.greps[self.selected].line_index;

        match utils::find_closest_index(&search_lines_idxs, current_line_idx) {
            Some(found_idx) => {
                if found_idx == current_line_idx {
                    match search_lines_idxs.binary_search(&found_idx) {
                        Ok(idx) => {
                            match search_lines_idxs.get(modifier(idx)) {
                                Some(_) => {
                                    self.greps[self.selected].line_index = search_lines_idxs
                                        [modifier(idx)]
                                }
                                None => {}
                            }
                        }
                        Err(_) => {}
                    }
                } else {
                    self.greps[self.selected].line_index = found_idx;
                }
            }
            None => {}
        }
    }

    pub fn next_search(&mut self) {
        self.modify_search(|searches_idx| searches_idx + 1);
    }

    pub fn prev_search(&mut self) {
        self.modify_search(|searches_idx| {
            if searches_idx != 0 {
                searches_idx - 1
            } else {
                searches_idx
            }
        });
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
            Prompt::NextSearch => greps.next_search(),
            Prompt::PrevSearch => greps.prev_search(),
            //_ => {}
        }

        mv(0, 0);
    }
}
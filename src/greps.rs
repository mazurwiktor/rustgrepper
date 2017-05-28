use regex::Regex;
use utils;

use std::collections::HashMap;

pub struct Grep<'a> {
    pub patern: String,
    pub line_index: usize,
    search_lines_idxs: Vec<usize>,
    pub lines: Vec<utils::Line<'a>>,
}

pub struct Greps<'a> {
    pub greps: Vec<Grep<'a>>,
    current_search_pattern: String,
    decorations: HashMap<String, utils::DecorationPattern>,
    pub selected: usize,
}

impl<'a> Greps<'a> {
    pub fn new(lines: Vec<utils::Line<'a>>) -> Self {
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

    pub fn current_grep(&self) -> &Grep {
        &self.greps[self.selected]
    }

    pub fn change_current_line_index(&mut self, index: usize) {
        self.greps[self.selected].line_index = index;
    }

    pub fn apply_search_patern(&mut self, pattern: &str) {
        if let Ok(re) = Regex::new(&pattern) {
            let search_lines_idxs = self.greps[self.selected]
                .lines
                .iter()
                .enumerate()
                .filter(|&(_, ref l)| re.is_match(l.buffer))
                .map(|(idx, _)| idx)
                .collect::<Vec<usize>>();

            if search_lines_idxs.len() > 0 {
                self.greps[self.selected].search_lines_idxs = search_lines_idxs;
                self.greps[self.selected].line_index = self.greps[self.selected].search_lines_idxs
                    [0];
                self.decorations.remove(&self.current_search_pattern);
                self.current_search_pattern = pattern.to_string();
                self.decorations
                .insert(self.current_search_pattern.clone(),
                        utils::DecorationPattern::from_single_attr(utils::Attribute::Inverse,
                                                                   &self.current_search_pattern));
            }
        }
    }

    pub fn decorations(&self) -> Vec<utils::DecorationPattern> {
        self.decorations
            .clone()
            .into_iter()
            .map(|(_, v)| v)
            .collect()
    }

    pub fn new_grep(&mut self, patern: &str) {
        if let Ok(re) = Regex::new(&patern) {
            let cur_patern = self.current_grep().patern.clone();
            let new_lines = self.greps[self.selected]
                .lines
                .clone()
                .into_iter()
                .filter(|l| re.is_match(l.buffer))
                .map(|l| l)
                .collect();

            self.greps.push(Grep {
                                patern: cur_patern + " > " + patern,
                                line_index: 0,
                                search_lines_idxs: Vec::new(),
                                lines: new_lines,
                            });
            self.selected = self.greps.len() - 1;
        }
    }

    pub fn modify_search<F>(&mut self, modifier: F)
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
        self.modify_search(|searches_idx| if searches_idx != 0 {
                               searches_idx - 1
                           } else {
                               searches_idx
                           });
    }

    pub fn select_one_to_left(&mut self) {
        if self.selected >= 1 {
            self.selected = self.selected - 1;
        }
    }
    pub fn select_one_to_right(&mut self) {
        if self.selected < self.greps.len() - 1 {
            self.selected = self.selected + 1;
        }
    }

    pub fn close_grep(&mut self) {
        if self.selected != 0 {
            let curent = self.selected;
            self.selected = self.selected - 1;
            self.greps.remove(curent);
        }
    }
}

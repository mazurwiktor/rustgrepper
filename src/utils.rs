extern crate regex;

use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::env;
use std::path::Path;

use std::cmp::Ordering;

#[allow(unused)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Attribute {
    None,
    Inverse,
    Red,
}

#[derive(Clone, PartialEq, Debug)]
pub struct DecorationPattern {
    pattern: String,
    attributes: Vec<Attribute>,
}

impl DecorationPattern {
    fn into_decoration<'a>(&self, decoration_str: &'a str) -> Decorations<'a> {
        Decorations::Some(self.attributes.clone(), decoration_str)
    }

    pub fn from_single_attr(attr: Attribute, pattern: &str) -> DecorationPattern {
        DecorationPattern {
            pattern: pattern.to_string(),
            attributes: vec![attr],
        }
    }

    fn cmp(&self, b: &DecorationPattern, buffer: &str) -> Ordering {
        if let Some(first_string) = match_against_pattern(buffer, &self.pattern) {
            if let Some(second_string) = match_against_pattern(buffer, &b.pattern) {
                let first_idx = buffer.find(&first_string);
                let second_idx = buffer.find(&second_string);
                if first_idx > second_idx {
                    Ordering::Greater
                } else if second_idx > first_idx {
                    Ordering::Less
                } else {
                    Ordering::Equal
                }
            } else {
                return Ordering::Less;
            }
        } else {
            return Ordering::Greater;
        }


    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Decorations<'a> {
    None(&'a str),
    Some(Vec<Attribute>, &'a str),
}

#[derive(Clone)]
pub struct Line<'a> {
    pub buffer: &'a str,
}

impl<'a> Line<'a> {
    fn from(buffer: &'a str) -> Line<'a> {
        Line { buffer: buffer }
    }
    /// [Decorations::None(&"some"), Decorations::Inverse(&"test b"), Decorations::Inverse(&"cd n"), Decorations::Inverse(&"etc")]
    /// buffer : some test buffer cd nothing etc empty
    /// output : [Decorations::None(&"some "), Decorations::Inverse(&"test b"), Decorations::None(&"uffer")]
    ///
    /// example
    /// some test b |match against all patterns| -> first matched -> None(some) && Imp(test b)
    /// uffer cd n |match against all patterns| -> first matched -> None(uffer) && Imp(cd n)
    /// othing etc |match against all pattern| -> first matched -> None(othing ) && Imp(test etc)
    /// empty |match against all pattern| -> none matched -> None(empty)

    pub fn decorate(&self,
                    mut decorations_patterns: Vec<DecorationPattern>)
                    -> Vec<Decorations<'a>> {
        let line_buffer = &self.buffer;
        let mut words: Vec<Decorations<'a>> = Vec::new();
        let mut current_idx = 0;
        let mut should_try_matching = true;

        while should_try_matching {
            should_try_matching = false;
            decorations_patterns.sort_by(|a, b| a.cmp(b, &line_buffer[current_idx..]));
            for decoration in &decorations_patterns {
                let pattern = &decoration.pattern;
                if let Some(matched_str) =
                    match_against_pattern(&line_buffer[current_idx..], pattern) {
                    if let Some(matched_idx) = line_buffer[current_idx..].find(&matched_str) {
                        let match_begin = current_idx + matched_idx;
                        let matched = &line_buffer[match_begin..(match_begin + matched_str.len())];
                        if match_begin != current_idx {
                            words.push(Decorations::None(&line_buffer[current_idx..match_begin]));
                            current_idx += match_begin - current_idx;
                        }
                        words.push(decoration.into_decoration(matched));
                        current_idx += matched_str.len();

                        should_try_matching = true;
                        break;
                    }
                }


            }
        }
        if current_idx < line_buffer.len() {
            words.push(Decorations::None(&line_buffer[current_idx..]));
        }

        words
    }
}

pub fn match_against_pattern(line_buffer: &str, pattern: &str) -> Option<String> {
    let re = regex::Regex::new(pattern).unwrap();
    let mut capture = "".to_string();
    for cap in re.captures_iter(line_buffer) {
        capture = format!("{}", &cap[0]);
    }
    match capture.as_ref() {
        "" => None,
        _ => Some(capture),
    }
}
#[test]
fn test_match_against_pattern() {
    assert_eq!(match_against_pattern(&"test", &"test"),
               Some("test".to_string()));
    assert_eq!(match_against_pattern(&"test", &"t"), Some("t".to_string()));
    assert_eq!(match_against_pattern(&"test test", &"test"),
               Some("test".to_string()));
    assert_eq!(match_against_pattern(&"string test", &"test"),
               Some("test".to_string()));
    assert_eq!(match_against_pattern(&"SOME TEST BUFFER", &"SOME"),
               Some("SOME".to_string()));
    assert_eq!(match_against_pattern(&"SOME", &"NONE"), None);
}

#[derive(Clone)]
pub struct Text<'a> {
    pub lines: Vec<Line<'a>>,
}

impl<'a> Text<'a> {
    #[allow(unused)]
    pub fn new() -> Self {
        Text { lines: Vec::new() }
    }

    pub fn from(buffer: &'a str) -> Self {
        Text { lines: buffer.split('\n').map(Line::from).collect() }
    }

    #[allow(unused)]
    pub fn add_line(&mut self, line: &'a str) {
        self.lines.push(Line::from(line));
    }

    #[allow(unused)]
    pub fn fill_from_buffer(&mut self, buffer: &'a str) {
        let lines = buffer.split('\n').collect::<Vec<&str>>();
        for line in &lines {
            self.add_line(&line);
        }
    }
}

#[test]
fn test_new_tex_from_buffer() {
    let mut text = Text::new();
    let buffer = &"SOME TEST BUFFER";
    text.fill_from_buffer(buffer);

    assert_eq!(&text.lines[0].buffer, buffer);
}

#[test]
fn test_pattern_decorations() {
    let mut text = Text::new();
    let buffer = &"SOME TEST BUFFER";
    text.fill_from_buffer(buffer);

    assert_eq!(text.lines[0].decorate(vec![DecorationPattern::from_single_attr(Attribute::Inverse,
                                                                      &"SOME")])
                   [0],
               Decorations::Some(vec![Attribute::Inverse], &"SOME"));
    assert_eq!(text.lines[0].decorate(vec![DecorationPattern::from_single_attr(Attribute::Inverse,
                                                                      &"TEST")])
                   [0],
               Decorations::None(&"SOME "));
    assert_eq!(text.lines[0].decorate(vec![DecorationPattern::from_single_attr(Attribute::Inverse,
                                                                      &"TEST")])
                   [1],
               Decorations::Some(vec![Attribute::Inverse], &"TEST"));
    assert_eq!(text.lines[0].decorate(vec![DecorationPattern::from_single_attr(Attribute::Inverse,
                                                                      &"TEST")])
                   [2],
               Decorations::None(&" BUFFER"));
}

#[test]
fn more_complicated_pattern_decorations() {
    let mut text = Text::new();
    let buffer = &"a b c d e f g h i j k l m n o p r s t";
    /////////////////1///2///////3////////4/////////5////
    text.fill_from_buffer(buffer);
    let decorations = vec![DecorationPattern::from_single_attr(Attribute::Red, &"a b c"),
                           DecorationPattern::from_single_attr(Attribute::Red, &"g h i"),
                           DecorationPattern::from_single_attr(Attribute::Red, &"r s t")];


    let decorated_line = text.lines[0].decorate(decorations);
    //assert!(false);
    assert_eq!(decorated_line.len(), 5);
}

#[test]
fn more_complicated_pattern_decorations_last_is_first() {
    let mut text = Text::new();
    let buffer = &"a b c d e f g h i j k l m n o p r s t";
    /////////////////1///2///////3////////4/////////5////
    text.fill_from_buffer(buffer);
    let decorations = vec![DecorationPattern::from_single_attr(Attribute::Red, &"r s t"),
                           DecorationPattern::from_single_attr(Attribute::Red, &"g h i"),
                           DecorationPattern::from_single_attr(Attribute::Red, &"a b c")];


    let decorated_line = text.lines[0].decorate(decorations);
    //assert!(false);
    assert_eq!(decorated_line.len(), 5);
}

#[test]
fn match_same_pattern() {
    let mut text = Text::new();
    let buffer = &"a a a d e f g h i j k l m n o p a a a";
    /////////////////1///2///////3////////4/////////5////
    text.fill_from_buffer(buffer);
    let decorations = vec![DecorationPattern::from_single_attr(Attribute::Red, &"a a a"),
                           DecorationPattern::from_single_attr(Attribute::Red, &"g h i"),
                           DecorationPattern::from_single_attr(Attribute::Red, &"unmatched")];


    let decorated_line = text.lines[0].decorate(decorations);
    //assert!(false);
    assert_eq!(decorated_line.len(), 5);
}

pub fn find_closest_index(indexes: &[usize], search: usize) -> Option<usize> {
    let idx = indexes.binary_search(&search);
    if indexes.is_empty() {
        return None;
    }
    match idx {
        Ok(found) => return Some(indexes[found]),
        Err(mut closest) => {
            if closest == indexes.len() {
                closest = indexes.len() - 1;
            }
            if closest < indexes.len() / 2 {
                for (i, number) in indexes.iter().enumerate() {
                    if *number > search {
                        if i != 0 {
                            return Some(indexes[i - 1]);
                        } else {
                            return Some(*number);
                        }

                    }
                }
            } else {
                for (i, number) in indexes.iter().rev().enumerate() {
                    if *number < search {
                        if i != 0 {
                            return Some(indexes[i - 1]);
                        } else {
                            return Some(*number);
                        }

                    }
                }
            }
        }
    }
    None
}

#[test]
fn find_closest_index_test() {
    let test_data = (10..20 + 1).collect::<Vec<usize>>();

    assert_eq!(Some(10), find_closest_index(&test_data, 9));
    assert_eq!(Some(10), find_closest_index(&test_data, 1));
    assert_eq!(Some(20), find_closest_index(&test_data, 21));
    assert_eq!(Some(10), find_closest_index(&test_data, 10));
    assert_eq!(Some(20), find_closest_index(&test_data, 100));
}

#[test]
fn find_closest_index_more_data_test() {
    let mut first = (10..20 + 1).collect::<Vec<usize>>();
    let second = (40..60 + 1).collect::<Vec<usize>>();
    first.extend(second);
    let test_data = first;

    assert_eq!(Some(10), find_closest_index(&test_data, 9));
    assert_eq!(Some(10), find_closest_index(&test_data, 1));
    assert_eq!(Some(20), find_closest_index(&test_data, 21));
    assert_eq!(Some(10), find_closest_index(&test_data, 10));
    assert_eq!(Some(15), find_closest_index(&test_data, 15));
    assert_eq!(Some(60), find_closest_index(&test_data, 100));
}


#[test]
fn find_closest_index_empty_test() {
    let test_data = Vec::new();

    assert_eq!(None, find_closest_index(&test_data, 10));
}

pub fn test_buffer_from_file() -> String {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage:\n\t{} <rust file>", args[0]);
        println!("Example:\n\t{} examples/ex_5.rs", args[0]);
        panic!("Unable to open file");
    }
    let mut buffer = String::new();

    let f = File::open(Path::new(&args[1])).unwrap();
    let reader = BufReader::new(f);

    for line in reader.lines() {
        buffer.push_str(&line.unwrap());
        buffer.push_str(&"\n");
    }
    buffer
}

extern crate ncurses;
extern crate regex;

mod utils;
mod prompt;
mod pager;
mod curses_pager;
mod greps;

use prompt::*;
use curses_pager::*;
use pager::*;
use greps::*;

fn main() {
    let buffer = utils::test_buffer_from_file();

    let mut pager = CursesPager::new();
    pager.initialize();
    let mut greps = Greps::new(utils::Text::from(&buffer).lines);

    loop {
        let index = greps.current_grep().line_index;
        let printed_lines = pager.execute_logs(&greps.current_grep().lines[index..], greps.decorations());

        pager.status(&greps);
        match prompt(&mut pager, PromptMode::Visual) {
            Prompt::Exit => break,
            Prompt::SearchPattern(pat) => {
                greps.apply_search_patern(&pat);
            }
            Prompt::GrepPattern(pat) => {
                greps.new_grep(&pat);
                pager.clear();
            }
            Prompt::GrepLeft => greps.select_one_to_left(),
            Prompt::GrepRight => greps.select_one_to_right(),
            Prompt::CloseGrep => greps.close_grep(),
            Prompt::SingleLineDown => {
                let last_index = greps.current_grep().lines.len() - 1;
                if index < last_index - printed_lines + 2 {
                    greps.change_current_line_index(index + 1)
                }
            }
            Prompt::SingleLineUp => {
                if index > 0 {
                    greps.change_current_line_index(index - 1);
                }
            }
            Prompt::ScrollTop => greps.change_current_line_index(0),
            Prompt::NextPage => greps.change_current_line_index(index + printed_lines),
            Prompt::ScrollBottom => {
                let last_index = greps.current_grep().lines.len() - printed_lines + 1;
                greps.change_current_line_index(last_index);
            }
            Prompt::NextSearch => greps.next_search(),
            Prompt::PrevSearch => greps.prev_search(),
            //_ => {}
        }

        pager.mv_cursor((0, 0));
    }
}
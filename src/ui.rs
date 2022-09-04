use crate::state::{SearchMode, State};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use ncurses as nc;
use pp::*;
use regex::Regex;
use unicode_width::UnicodeWidthChar;

const LABEL: &str =
    "Type to filter, UP/DOWN move, LEFT/RIGHT move cursor, ENTER/TAB select, ESC quit";

pub struct UserInterface {
    pub page: Page,
    pub cursor: Cursor,
}

impl UserInterface {
    pub fn new() -> Self {
        Self {
            page: Page::new(1),
            cursor: Cursor::new(),
        }
    }

    pub fn populate_screen(&self, state: &State) {
        for (row_idx, cmd) in self.page.contents(state).iter().enumerate() {
            // Make command fit the screen and print everything normally first
            let cmd = &cmd
                .chars()
                .take(nc::COLS() as usize - 2)
                .collect::<String>();
            nc::mvaddstr(row_idx as i32 + 3, 1, &ljust(cmd));

            // Paint matched chars, if any;
            match state.search_mode {
                SearchMode::Exact | SearchMode::Regex => {
                    let matches = self.substring_indices(cmd, &state.query.text);

                    if !matches.is_empty() {
                        self.paint_matched_chars(cmd, matches, row_idx);
                    }
                }
                SearchMode::Fuzzy => {
                    let matcher = SkimMatcherV2::default();

                    if let Some(matches) = matcher.fuzzy_indices(cmd, &state.query.text) {
                        self.paint_matched_chars(cmd, matches.1, row_idx);
                    }
                }
            }
            // Finally, paint selection
            self.paint_selected(cmd, row_idx);
        }
        self.paint_bars(state);
    }

    fn substring_indices<'a>(&self, string: &'a str, substring: &'a str) -> Vec<usize> {
        // Returns the indices of a substring within a string
        match Regex::new(substring) {
            Ok(r) => r.find_iter(string).flat_map(|m| m.range()).collect(),
            Err(_) => vec![],
        }
    }

    fn paint_matched_chars(&self, command: &str, indices: Vec<usize>, row_idx: usize) {
        for (col_idx, byte_idx, ch) in column_indices(command) {
            if indices.contains(&byte_idx) {
                nc::attron(nc::COLOR_PAIR(5) | nc::A_BOLD());
                nc::mvaddstr(row_idx as i32 + 3, col_idx as i32 + 1, &ch.to_string());
                nc::attroff(nc::COLOR_PAIR(5) | nc::A_BOLD());
            }
        }
    }

    fn paint_selected(&self, entry: &str, index: usize) {
        if index == self.page.selected as usize {
            nc::attron(nc::COLOR_PAIR(2));
            nc::mvaddstr(index as i32 + 3, 1, &ljust(entry));
            nc::attroff(nc::COLOR_PAIR(2));
        }
    }

    fn paint_bars(&self, state: &State) {
        nc::mvaddstr(1, 1, LABEL);
        nc::attron(nc::COLOR_PAIR(3));
        nc::mvaddstr(2, 1, &ljust(&status_bar(state, self)));
        nc::attroff(nc::COLOR_PAIR(3));
        nc::mvaddstr(0, 1, &top_bar(&state.query.text));
    }
}

pub struct Page {
    pub value: i32,
    pub selected: i32,
}

impl Page {
    pub fn new(value: i32) -> Self {
        Self { value, selected: 0 }
    }

    fn size(&self, state: &State) -> i32 {
        self.contents(state).len() as i32
    }

    fn contents(&self, state: &State) -> Vec<String> {
        match state
            .search_results
            .chunks(nc::LINES() as usize - 3)
            .nth(self.value as usize - 1)
        {
            Some(cmds) => cmds.to_vec(),
            None => Vec::new(),
        }
    }

    pub fn selected(&self, state: &State) -> Option<String> {
        self.contents(state).get(self.selected as usize).cloned()
    }

    pub fn turn(&mut self, state: &State, direction: Direction) {
        /* Turning the page essentially works as follows:
         *
         * We are getting the potential page by subtracting 1
         * from the page number, because pages are 1-based, and
         * we need them to be 0-based for the calculation to work.
         * Then we stately the direction which is always +1 or -1.
         *
         * We then use the remainder part of Euclidean division of
         * potential page over total number of pages, in order to
         * wrap the page number around the total number of pages.
         *
         * This means that if we are on page 4, and there are 4 pages in total,
         * the command to go to the next page would result in rem(4, 4),
         * which is 0, and by adjusting the page number to be 1-based,
         * we get back to page 1, as desired.
         *
         * This also works in the opposite direction:
         *
         * If there are 4 total pages, and we are on page 1, and we issue
         * the command to go to the previous page, we are doing: rem(-1, 4),
         * which is 3. By adjusting the page number to be 1-based,
         * we get to the 4th page.
         *
         * The total number of pages being 0, which is the case when there
         * are no history in the history, means that we are dividing by 0,
         * which is undefined, and rem() returns None, which means that we are
         * on page 1.
         */
        nc::clear();
        let next_page = self.value - 1 + direction as i32;
        let pages = self.total_pages(state);
        self.value = match i32::checked_rem_euclid(next_page, pages) {
            Some(x) => x + 1,
            None => 1,
        }
    }

    pub fn total_pages(&self, state: &State) -> i32 {
        state.history.chunks(nc::LINES() as usize - 3).len() as i32
    }

    pub fn move_selected(&mut self, state: &State, direction: Direction) {
        self.selected += direction as i32;
        if let Some(wraparound) = i32::checked_rem_euclid(self.selected, self.size(state)) {
            self.selected = wraparound;
            match direction {
                Direction::Forward => {
                    if self.selected == 0 {
                        self.turn(state, Direction::Forward);
                    }
                }
                Direction::Backward => {
                    if self.selected == (self.size(state) - 1) {
                        self.turn(state, Direction::Backward);
                        self.selected = self.size(state) - 1;
                    }
                }
            }
        }
    }
}

pub struct Cursor {
    pub position: usize,
}

impl Cursor {
    pub fn new() -> Self {
        Self { position: 0 }
    }

    pub fn step(&mut self, state: &State, direction: Direction) {
        match direction {
            Direction::Forward => {
                if self.position < state.query.text.chars().count() {
                    self.position += 1;
                }
            }
            Direction::Backward => {
                if self.position > 0 {
                    self.position -= 1;
                }
            }
        }

        let prompt_length = pp::get_shell_prompt().chars().count();
        let query_width: usize = state
            .query
            .text
            .chars()
            .take(self.position)
            .map(|ch| ch.width().unwrap_or(0))
            .sum();

        nc::wmove(
            nc::stdscr(),
            0,
            (prompt_length + 1 + 1 + query_width) as i32,
        );
    }
}

pub mod curses {
    use ncurses as nc;

    pub fn init() {
        nc::setlocale(nc::LcCategory::all, "");
        nc::initscr();
        nc::noecho();
        nc::keypad(nc::stdscr(), true);
        init_color_pairs();
    }

    pub fn init_color_pairs() {
        nc::start_color();
        nc::init_pair(1, nc::COLOR_WHITE, nc::COLOR_BLACK); // normal
        nc::init_pair(2, nc::COLOR_WHITE, nc::COLOR_GREEN); // highlighted-green (selected item)
        nc::init_pair(3, nc::COLOR_BLACK, nc::COLOR_WHITE); // highlighted-white (status)
        nc::init_pair(5, nc::COLOR_RED, nc::COLOR_BLACK); // red (searched items)
        nc::init_pair(6, nc::COLOR_WHITE, nc::COLOR_RED); // higlighted-red
    }

    pub fn teardown() {
        nc::clear();
        nc::refresh();
        nc::doupdate();
        nc::endwin();
    }
}

mod pp {
    /* Pretty printer */
    use crate::state::{SearchMode, State};
    use crate::ui::UserInterface;
    use ncurses as nc;
    use std::env;
    use unicode_width::UnicodeWidthStr;

    pub fn status_bar(state: &State, user_interface: &UserInterface) -> String {
        let total_pages = user_interface.page.total_pages(state);
        format!(
            "- search:{} (C-e) - case:{} (C-t) - page {}/{} -",
            search_mode(state.search_mode),
            case(state.case_sensitivity),
            current_page(user_interface.page.value, total_pages),
            total_pages,
        )
    }

    pub fn top_bar(query: &str) -> String {
        format!("{} {}", get_shell_prompt(), query)
    }

    pub fn get_shell_prompt() -> String {
        format!(
            "{}@{}$",
            env::var("USER").unwrap(),
            gethostname::gethostname().into_string().unwrap()
        )
    }

    pub fn search_mode(value: SearchMode) -> &'static str {
        match value {
            SearchMode::Exact => "exact",
            SearchMode::Regex => "regex",
            SearchMode::Fuzzy => "fuzzy",
        }
    }

    pub fn case(value: bool) -> &'static str {
        if value {
            "sensitive"
        } else {
            "insensitive"
        }
    }

    fn current_page(current_page: i32, total_pages: i32) -> i32 {
        if total_pages > 0 {
            current_page
        } else {
            0
        }
    }

    pub fn ljust(string: &str) -> String {
        let overhead = string.width() - string.chars().count();
        format!("{0:1$}", string, nc::COLS() as usize - 2 - overhead)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Forward = 1,
    Backward = -1,
}

pub struct ColumnIndices<'a> {
    inner: std::str::CharIndices<'a>,
    next_col: usize,
}

impl Iterator for ColumnIndices<'_> {
    type Item = (usize, usize, char);

    fn next(&mut self) -> Option<Self::Item> {
        let col_idx = self.next_col;
        let (byte_idx, ch) = self.inner.next()?;

        self.next_col += ch.width().unwrap_or(0);

        Some((col_idx, byte_idx, ch))
    }
}

pub fn column_indices(s: &str) -> ColumnIndices {
    ColumnIndices {
        inner: s.char_indices(),
        next_col: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest(
        string,
        substring,
        expected,
        case("cat spam", "cat", vec![0, 1, 2]),
        case("make -j4", "[0-9]+", vec![7]),
        case("ping -c 10 www.google.com", "[0-9]+", vec![8, 9])
    )]
    fn matched_chars_indices(string: &str, substring: &str, expected: Vec<usize>) {
        let user_interface = UserInterface::new();
        assert_eq!(
            user_interface.substring_indices(string, substring),
            expected
        );
    }
}

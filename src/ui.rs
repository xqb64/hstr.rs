use crate::state::{SearchMode, State};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use ncurses as nc;
use pp::*;
use regex::Regex;
use unicode_width::UnicodeWidthChar;

const LABEL: &str =
    "Type to filter, UP/DOWN move, LEFT/RIGHT move cursor, ENTER/TAB select, ESC quit";

pub struct UserInterface {
    cursor_position: usize,
    page_count: usize,
    page: usize,
    highlighted: usize,
    pub state: State,
}

impl UserInterface {
    pub fn new(query: &str) -> anyhow::Result<Self> {
        Ok(Self {
            cursor_position: 0,
            page_count: 0,
            page: 1,
            highlighted: 0,
            state: State::new(query)?,
        })
    }

    pub fn populate_screen(&self) {
        for (row_idx, cmd) in self.get_page_contents().iter().enumerate() {
            // Make command fit the screen and print everything normally first
            let cmd = &cmd
                .chars()
                .take(nc::COLS() as usize - 2)
                .collect::<String>();
            nc::mvaddstr(row_idx as i32 + 3, 1, &ljust(cmd));

            // Paint matched chars, if any;
            match self.state.search_mode {
                SearchMode::Exact | SearchMode::Regex => {
                    let matches = self.substring_indices(cmd, &self.state.query.text);

                    if !matches.is_empty() {
                        self.paint_matched_chars(cmd, matches, row_idx);
                    }
                }
                SearchMode::Fuzzy => {
                    let matcher = SkimMatcherV2::default();

                    if let Some(matches) = matcher.fuzzy_indices(cmd, &self.state.query.text) {
                        self.paint_matched_chars(cmd, matches.1, row_idx);
                    }
                }
            }
            // Finally, paint selection
            self.paint_highlighted(cmd, row_idx);
        }
        self.paint_bars();
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

    fn paint_highlighted(&self, entry: &str, index: usize) {
        if index == self.highlighted {
            nc::attron(nc::COLOR_PAIR(2));
            nc::mvaddstr(index as i32 + 3, 1, &ljust(entry));
            nc::attroff(nc::COLOR_PAIR(2));
        }
    }

    fn paint_bars(&self) {
        nc::mvaddstr(1, 1, LABEL);
        nc::attron(nc::COLOR_PAIR(3));
        nc::mvaddstr(2, 1, &ljust(&self.status_bar()));
        nc::attroff(nc::COLOR_PAIR(3));
        nc::mvaddstr(0, 1, &top_bar(&self.state.query.text));
    }

    pub fn status_bar(&self) -> String {
        format!(
            "- search:{} (C-e) - case:{} (C-t) - page {}/{} -",
            search_mode(self.state.search_mode),
            case(self.state.case_sensitivity),
            self.current_page(),
            self.compute_page_count(),
        )
    }

    pub fn compute_page_count(&self) -> usize {
        self.state
            .search_results
            .chunks(nc::LINES() as usize - 3)
            .len()
    }

    fn current_page(&self) -> usize {
        if self.page_count > 0 {
            self.page
        } else {
            0
        }
    }

    fn compute_page_size(&self) -> usize {
        self.get_page_contents().len()
    }

    fn get_page_contents(&self) -> Vec<String> {
        match self
            .state
            .search_results
            .chunks(nc::LINES() as usize - 3)
            .nth(self.page as usize - 1)
        {
            Some(cmds) => cmds.to_vec(),
            None => Vec::new(),
        }
    }

    pub fn compute_highlighted(&self) -> Option<String> {
        self.get_page_contents()
            .get(self.highlighted as usize)
            .cloned()
    }

    pub fn turn_page(&mut self, direction: Direction) {
        /* Turning the page essentially works as follows:
         *
         * We are getting the potential page by subtracting 1
         * from the page number, because pages are 1-based, and
         * we need them to be 0-based for the calculation to work.
         * Then we add the direction which is always +1 or -1.
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
         * are no entries in the history, means that we are dividing by 0,
         * which is undefined, and rem() returns None, which means that we are
         * on page 1.
         */
        nc::clear();
        let potential_page = (self.page - 1) as isize + (direction as isize);
        self.page = match potential_page.checked_rem_euclid(self.compute_page_count() as isize) {
            Some(x) => (x + 1) as usize,
            None => 1,
        }
    }

    pub fn move_highlighted(&mut self, direction: Direction) {
        /* Moving the highlighted entry works as follows:
         *
         * We are getting the potential highlighted entry
         * index by adding the direction to the current
         * highlighted entry index. Then, we do a checked
         * Euclidian division of potential highlighted entry
         * index over total number of entries on the current
         * page. Specifically, we are interested in the
         * remainder part:
         *
         * If the remainder is zero, and the direction is
         * Direction::Forward, this means that the potential
         * highlighted entry is on the next page and we need to
         * turn the page forward.
         *
         * If the remainder is equal to the number of entries
         * on a page minus one (adjusting for `self.highlighted`
         * being 0-based), and the direction is Direction::Backward,
         * this means the potential highlighted entry is on the
         * previous page, and we need to turn the page backwards.
         */
        let potential_highlighted = self.highlighted as isize + direction as isize;
        if let Some(rem) =
            potential_highlighted.checked_rem_euclid(self.compute_page_size() as isize)
        {
            self.highlighted = rem as usize;
            match direction {
                Direction::Forward => {
                    if self.highlighted == 0 {
                        self.turn_page(Direction::Forward);
                    }
                }
                Direction::Backward => {
                    /* -1 because `self.highlighted` is 0-based. */
                    if self.highlighted == self.compute_page_size() - 1 {
                        self.turn_page(Direction::Backward);

                        /* Because we might end up on a page that
                         * has fewer entries than the one used in
                         * the calculation, we need to select the
                         * last entry again - now based on the count
                         * of entries of the newly selected page. */
                        self.highlighted = self.compute_page_size() - 1;
                    }
                }
            }
        }
    }

    pub fn set_highlighted(&mut self, i: usize) {
        self.highlighted = i;
    }

    pub fn set_page(&mut self, i: usize) {
        self.page = i;
    }

    pub fn move_cursor(&mut self, direction: Direction) {
        match direction {
            Direction::Forward => {
                if self.cursor_position < self.state.query.text.chars().count() {
                    self.cursor_position += 1;
                }
            }
            Direction::Backward => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
            }
        }

        let prompt_length = pp::get_shell_prompt().chars().count();
        let query_width: usize = self
            .state
            .query
            .text
            .chars()
            .take(self.cursor_position)
            .map(|ch| ch.width().unwrap_or(0))
            .sum();

        nc::wmove(
            nc::stdscr(),
            0,
            (prompt_length + 1 + 1 + query_width) as i32,
        );
    }

    pub fn get_cursor_position(&self) -> usize {
        self.cursor_position
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
        nc::init_pair(2, nc::COLOR_WHITE, nc::COLOR_GREEN); // highlighted-green (highlighted item)
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
    use crate::state::SearchMode;
    use ncurses as nc;
    use std::env;
    use unicode_width::UnicodeWidthStr;

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

    pub fn ljust(string: &str) -> String {
        let overhead = string.width() - string.chars().count();
        format!("{0:1$}", string, nc::COLS() as usize - 2 - overhead)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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
        let user_interface = UserInterface::new("").unwrap();
        assert_eq!(
            user_interface.substring_indices(string, substring),
            expected
        );
    }
}

use crate::app::{Application, View};
use crate::util::get_shell_prompt;
use ncurses::*;
use regex::Regex;

const LABEL: &str =
    "Type to filter, UP/DOWN move, RET/TAB select, DEL remove, ESC quit, C-f add/rm fav";

pub struct UserInterface {
    page: i32,
    selected: i32,
}

impl UserInterface {
    pub fn new() -> Self {
        Self {
            page: 1,
            selected: 0,
        }
    }

    pub fn set_page(&mut self, v: i32) {
        self.page = v;
    }

    pub fn set_selected(&mut self, v: i32) {
        self.selected = v;
    }

    pub fn init_color_pairs(&self) {
        start_color();
        init_pair(1, COLOR_WHITE, COLOR_BLACK); // normal
        init_pair(2, 15, COLOR_GREEN); // highlighted-green (selected item)
        init_pair(3, 0, 15); // highlighted-white (status)
        init_pair(4, 15, 0); // white (favorites)
        init_pair(5, COLOR_RED, 0); // red (searched items)
        init_pair(6, 15, COLOR_RED); // higlighted-red
    }

    pub fn populate_screen(&self, app: &Application) {
        clear();
        let commands = self.get_page(app.get_commands());
        for (index, entry) in commands.iter().enumerate() {
            mvaddstr(
                index as i32 + 3,
                1,
                &format!("{1:0$}", COLS() as usize - 1, entry),
            );
            let substring_indexes = self.get_substring_indexes(&entry, &app.search_string);
            if !substring_indexes.is_empty() {
                for (idx, letter) in entry.chars().enumerate() {
                    if substring_indexes.contains(&idx) {
                        attron(COLOR_PAIR(5) | A_BOLD());
                        mvaddch(index as i32 + 3, idx as i32 + 1, letter as chtype);
                        attroff(COLOR_PAIR(5) | A_BOLD());
                    } else {
                        mvaddch(index as i32 + 3, idx as i32 + 1, letter as chtype);
                    }
                }
            }
            if app.get_commands().contains(&entry) {
                attron(COLOR_PAIR(4));
                mvaddstr(
                    index as i32 + 3,
                    1,
                    &format!("{1:0$}", COLS() as usize - 1, entry),
                );
                attroff(COLOR_PAIR(4));
            }
            if index == self.selected as usize {
                attron(COLOR_PAIR(2));
                mvaddstr(
                    index as i32 + 3,
                    1,
                    &format!("{1:0$}", COLS() as usize - 1, entry),
                );
                attroff(COLOR_PAIR(2));
            }
        }
        mvaddstr(1, 1, LABEL);
        attron(COLOR_PAIR(3));
        mvaddstr(
            2,
            1,
            &format!(
                "{1:0$}",
                COLS() as usize - 1,
                format!(
                    "- view:{} (C-/) - regex:{} (C-e) - case:{} (C-t) - page {}/{} -",
                    self.display_view(app.view),
                    self.display_regex_mode(app.regex_mode),
                    self.display_case(app.case_sensitivity),
                    self.page,
                    self.total_pages(app.get_commands())
                )
            ),
        );
        attroff(COLOR_PAIR(3));
        mvaddstr(
            0,
            1,
            &format!("{} {}", get_shell_prompt(), app.search_string),
        );
    }

    pub fn turn_page(&mut self, commands: &[String], direction: i32) {
        // Turning the page essentially works as follows:
        //
        // We are getting the potential page by subtracting 1
        // from the page number, because pages are 1-based, and
        // we need them to be 0-based for the calculation to work.
        // Then we apply the direction which is always +1 or -1.
        //
        // We then use the remainder part of Euclidean division of
        // potential page over total number of pages, in order to
        // wrap the page number around the total number of pages.
        //
        // This means that if we are on page 4, and there are 4 pages in total,
        // the command to go to the next page would result in rem(4, 4),
        // which is 0, and by adjusting the page number to be 1-based,
        // we get back to page 1, as desired.
        //
        // This also works in the opposite direction:
        //
        // If there are 4 total pages, and we are on page 1, and we issue
        // the command to go to the previous page, we are doing: rem(-1, 4),
        // which is 3. By adjusting the page number to be 1-based,
        // we get to the 4th page.
        //
        // The total number of pages being 0, which is the case when there
        // are no commands in the history, means that we are dividing by 0,
        // which is undefined, and rem() returns None, which means that we are
        // on page 1.

        let potential_page = self.page - 1 + direction;
        self.page = match i32::checked_rem_euclid(potential_page, self.total_pages(commands)) {
            Some(x) => x + 1,
            None => 1,
        }
    }

    pub fn move_selected(&mut self, commands: &[String], direction: i32) {
        let page_size = self.get_page_size(commands);
        self.selected += direction;
        if let Some(x) = i32::checked_rem_euclid(self.selected, page_size) {
            self.selected = x;
            if direction == 1 && self.selected == 0 {
                self.turn_page(commands, 1);
            } else if direction == -1 && self.selected == (page_size - 1) {
                self.turn_page(commands, -1);
                self.selected = self.get_page_size(commands) - 1;
            }
        }
    }

    pub fn get_selected(&self, commands: &[String]) -> String {
        String::from(
            self.get_page(&commands)
                .get(self.selected as usize)
                .unwrap(),
        )
    }

    pub fn prompt_for_deletion(&self, command: &str) {
        mvaddstr(1, 0, &format!("{1:0$}", COLS() as usize, ""));
        attron(COLOR_PAIR(6));
        mvaddstr(
            1,
            1,
            &format!("Do you want to delete all occurences of {}? y/n", command),
        );
        attroff(COLOR_PAIR(6));
    }

    fn total_pages(&self, commands: &[String]) -> i32 {
        commands.chunks(LINES() as usize - 3).len() as i32
    }

    fn get_page(&self, commands: &[String]) -> Vec<String> {
        match commands
            .chunks(LINES() as usize - 3)
            .nth(self.page as usize - 1)
        {
            Some(cmds) => cmds.to_vec(),
            None => Vec::new(),
        }
    }

    fn get_page_size(&self, commands: &[String]) -> i32 {
        self.get_page(commands).len() as i32
    }

    fn get_substring_indexes<'a>(&self, string: &'a str, substring: &'a str) -> Vec<usize> {
        match Regex::new(substring) {
            Ok(r) => r.find_iter(string).flat_map(|m| m.range()).collect(),
            Err(_) => vec![],
        }
    }

    fn display_view(&self, value: View) -> String {
        match value {
            View::Sorted => String::from("sorted"),
            View::Favorites => String::from("favorites"),
            View::All => String::from("all"),
        }
    }

    fn display_case(&self, value: bool) -> String {
        match value {
            true => String::from("sensitive"),
            false => String::from("insensitive"),
        }
    }

    fn display_regex_mode(&self, value: bool) -> String {
        match value {
            true => String::from("on"),
            false => String::from("off"),
        }
    }
}

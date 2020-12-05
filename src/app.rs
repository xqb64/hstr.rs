use crate::sort::sort;
use crate::util::{read_file, write_file};
use itertools::Itertools;
use maplit::hashmap;
use ncurses::*;
use regex::{escape, Regex, RegexBuilder};
use std::collections::HashMap;

const Y: i32 = 121;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum View {
    Sorted = 0,
    Favorites = 1,
    All = 2,
}

pub struct Application {
    pub to_restore: Option<HashMap<View, Vec<String>>>,
    pub commands: Option<HashMap<View, Vec<String>>>,
    pub view: View,
    pub regex_mode: bool,
    pub case_sensitivity: bool,
    pub search_string: String,
    pub shell: String,
}

impl Application {
    pub fn new(
        view: View,
        regex_mode: bool,
        case_sensitivity: bool,
        search_string: String,
        shell: String,
    ) -> Self {
        Self {
            to_restore: None,
            commands: None,
            view,
            regex_mode,
            case_sensitivity,
            search_string,
            shell,
        }
    }

    pub fn load_history(&mut self) {
        let history = read_file(format!(".{}_history", self.shell)).unwrap();
        let commands = hashmap! {
            View::All => history.clone().into_iter().unique().collect(),
            View::Sorted => sort(history),
            View::Favorites => read_file(
                format!(
                    ".config/hstr-rs/.{}_favorites",
                    self.shell
                )
            ).unwrap()
        };
        self.to_restore = Some(commands.clone());
        self.commands = Some(commands);
    }

    pub fn restore(&mut self) {
        self.commands = self.to_restore.clone();
    }

    pub fn get_commands(&self) -> &[String] {
        self.commands.as_ref().unwrap().get(&self.view).unwrap()
    }

    fn create_search_regex(&self) -> Option<Regex> {
        let search_string = if self.regex_mode {
            self.search_string.clone()
        } else {
            escape(&self.search_string)
        };
        RegexBuilder::new(&search_string)
            .case_insensitive(!self.case_sensitivity)
            .build()
            .ok()
    }

    pub fn search(&mut self) {
        let search_regex = match self.create_search_regex() {
            Some(r) => r,
            None => {
                return;
            }
        };
        self.commands
            .as_mut()
            .unwrap()
            .get_mut(&self.view)
            .unwrap()
            .retain(|x| search_regex.is_match(x));
    }

    pub fn add_or_rm_fav(&mut self, command: String) -> Result<(), std::io::Error> {
        let favorites = self
            .commands
            .as_mut()
            .unwrap()
            .get_mut(&View::Favorites)
            .unwrap();
        if !favorites.contains(&command) {
            favorites.push(command);
        } else {
            favorites.retain(|x| *x != command);
        }
        write_file(
            format!(".config/hstr-rs/.{}_favorites", self.shell),
            favorites,
        )?;
        Ok(())
    }

    pub fn delete_from_history(&mut self, command: String) -> Result<(), std::io::Error> {
        if getch() == Y {
            if let Some(cmds) = self.commands.as_mut().unwrap().get_mut(&View::All) {
                cmds.retain(|x| *x != command);
                write_file(format!(".{}_history", self.shell), cmds)?;
            }
        }
        Ok(())
    }

    pub fn toggle_case(&mut self) {
        self.case_sensitivity = !self.case_sensitivity;
    }

    pub fn toggle_regex_mode(&mut self) {
        self.regex_mode = !self.regex_mode;
    }

    pub fn toggle_view(&mut self) {
        self.view = match (self.view as u8 + 1) % 3 {
            0 => View::Sorted,
            1 => View::Favorites,
            2 => View::All,
            _ => View::Sorted,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(View::Sorted, View::Favorites; "Sorted -> Favorites")]
    #[test_case(View::Favorites, View::All; "Favorites -> All")]
    #[test_case(View::All, View::Sorted; "All -> Sorted")]
    fn toggle_view(before: View, after: View) {
        let mut app = Application::new(
            before,
            false,
            false,
            String::new(),
            String::from("bash"),
        );
        app.toggle_view();
        assert_eq!(app.view, after);
    }

    #[test_case(true; "true -> false")]
    #[test_case(false; "false -> true")]
    fn toggle_regex_mode(regex_mode: bool) {
        let mut app = Application::new(
            View::Sorted,
            regex_mode,
            false,
            String::new(),
            String::from("bash"),
        );
        app.toggle_regex_mode();
        assert_eq!(app.regex_mode, !regex_mode);
    }

    #[test_case(true; "true -> false")]
    #[test_case(false; "false -> true")]
    fn toggle_case(case_sensitivity: bool) {
        let mut app = Application::new(
            View::Sorted,
            false,
            case_sensitivity,
            String::new(),
            String::from("bash"),
        );
        app.toggle_case();
        assert_eq!(app.case_sensitivity, !case_sensitivity);
    }

}

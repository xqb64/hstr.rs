use crate::hstr::{self, History, Shell};
use crate::ui::UserInterface;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use regex::{escape, Regex, RegexBuilder};

#[derive(Clone)]
pub struct State {
    pub case_sensitivity: bool,
    pub search_mode: SearchMode,
    pub shell: Shell,
    pub query: Query,
    pub history: History,
    pub search_results: History,
}

impl State {
    pub fn new(query: &str) -> Self {
        let shell = setenv::get_shell().get_name();
        let history = match Shell::from_str(shell) {
            Some(sh) => match sh {
                Shell::Bash => hstr::get_bash_history(),
                Shell::Zsh => hstr::get_zsh_history(),
            },
            None => panic!("{} is not supported yet.", shell),
        };

        Self {
            case_sensitivity: false,
            search_mode: SearchMode::Exact,
            shell: Shell::from_str(shell).unwrap(),
            query: Query::new(query),
            search_results: history.clone(),
            history,
        }
    }

    pub fn search(&mut self) {
        match self.search_mode {
            SearchMode::Exact | SearchMode::Regex => {
                let search_regex = match self.create_search_regex() {
                    Some(r) => r,
                    None => {
                        return;
                    }
                };
                self.search_results = self.history.clone();
                self.search_results.retain(|cmd| search_regex.is_match(cmd));
            }
            SearchMode::Fuzzy => {
                let query = self.query.text.clone();
                if self.case_sensitivity {
                    let matcher = SkimMatcherV2::default().respect_case();
                    self.history
                        .retain(|cmd| matcher.fuzzy_match(cmd, &query).is_some());
                } else {
                    let matcher = SkimMatcherV2::default();
                    self.search_results = self.history.clone();
                    self.search_results
                        .retain(|cmd| matcher.fuzzy_match(cmd, &query).is_some());
                }
            }
        }
    }

    fn create_search_regex(&self) -> Option<Regex> {
        let query = match self.search_mode {
            SearchMode::Regex => self.query.text.clone(),
            SearchMode::Exact => escape(&self.query.text),
            _ => unreachable!(),
        };
        RegexBuilder::new(&query)
            .case_insensitive(!self.case_sensitivity)
            .build()
            .ok()
    }

    pub fn toggle_case(&mut self) {
        self.case_sensitivity = !self.case_sensitivity;
    }

    pub fn toggle_search_mode(&mut self) {
        self.search_mode = match (self.search_mode as u8 + 1) % 3 {
            0 => SearchMode::Exact,
            1 => SearchMode::Regex,
            2 => SearchMode::Fuzzy,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone)]
pub struct Query {
    pub text: String,
}

impl Query {
    pub fn new(text: &str) -> Self {
        Self {
            text: String::from(text),
        }
    }

    pub fn insert_char(&mut self, user_interface: &UserInterface, ch: char) {
        let position = self.bytelength(user_interface.cursor.position);
        self.text.insert(position, ch);
    }

    pub fn remove_char(&mut self, user_interface: &UserInterface) {
        let position = self.bytelength(user_interface.cursor.position - 1);
        self.text.remove(position);
    }

    fn bytelength(&self, index: usize) -> usize {
        self.text.chars().take(index).map(|ch| ch.len_utf8()).sum()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SearchMode {
    Exact = 0,
    Regex = 1,
    Fuzzy = 2,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::fixture;
    use rstest::rstest;

    #[fixture]
    pub fn fake_history() -> History {
        vec![
            "cat spam",
            "cat SPAM",
            "git add .",
            "git add . --dry-run",
            "git push origin master",
            "git rebase -i HEAD~2",
            "git checkout -b tests",
            "grep -r spam .",
            "ping -c 10 www.google.com",
            "ls -la",
            "lsusb",
            "lspci",
            "sudo reboot",
            "source .venv/bin/activate",
            "deactivate",
            "pytest",
            "cargo test",
            "xfce4-panel -r",
            "nano .gitignore",
            "sudo dkms add .",
            "cd ~/Downloads",
            "make -j4",
            "gpg --card-status",
            "echo šampion",
            "nano .github/workflows/build.yml",
            "cd /home/bwk/",
        ]
        .iter()
        .map(|&x| x.into())
        .collect()
    }

    #[fixture]
    pub fn fake_state(fake_history: History) -> State {
        let mut state = State::new("");
        state.history = fake_history;
        state
    }

    #[rstest(
        query,
        expected,
        search_mode,
        case_sensitivity,
        case("cat", vec!["cat spam", "cat SPAM"], SearchMode::Exact, false),
        case("spam", vec!["cat spam", "cat SPAM", "grep -r spam ."], SearchMode::Exact, false),
        case("SPAM", vec!["cat SPAM"], SearchMode::Exact, true),
        case("[0-9]+", vec!["git rebase -i HEAD~2", "ping -c 10 www.google.com", "xfce4-panel -r", "make -j4"], SearchMode::Regex, false),
        case("šp", vec!["echo šampion"], SearchMode::Fuzzy, false),
        case("hwk", vec!["nano .github/workflows/build.yml", "cd /home/bwk/"], SearchMode::Fuzzy, false)
    )]
    fn search(
        query: &str,
        expected: Vec<&str>,
        search_mode: SearchMode,
        case_sensitivity: bool,
        mut fake_state: State,
    ) {
        fake_state.search_mode = search_mode;
        fake_state.case_sensitivity = case_sensitivity;
        fake_state.query = Query::new(query);
        fake_state.search();
        assert_eq!(fake_state.search_results, expected);
    }
}

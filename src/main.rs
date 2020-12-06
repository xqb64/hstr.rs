use crate::app::Application;
use crate::ui::UserInterface;
use crate::util::write_file;
use ncurses::*;
use setenv::get_shell;

mod app;
mod sort;
mod ui;
mod util;

const CTRL_E: u32 = 5;
const CTRL_F: u32 = 6;
const TAB: u32 = 9;
const ENTER: u32 = 10;
const CTRL_T: u32 = 20;
const ESC: u32 = 27;
const CTRL_SLASH: u32 = 31;
const Y: i32 = 121;

fn main() -> Result<(), std::io::Error> {
    initscr();
    noecho();
    keypad(stdscr(), true);
    let shell = get_shell().get_name();
    let mut app = Application::new(shell);
    app.load_commands();
    let mut user_interface = UserInterface::new();
    user_interface.init_color_pairs();
    user_interface.populate_screen(&app);
    loop {
        let user_input = get_wch();
        match user_input.unwrap() {
            WchResult::Char(ch) => match ch {
                CTRL_E => {
                    app.toggle_regex_mode();
                    user_interface.set_selected(0);
                    user_interface.populate_screen(&app);
                }
                CTRL_F => {
                    let commands = app.get_commands();
                    let command = user_interface.get_selected(&commands);
                    app.add_or_rm_fav(command);
                    write_file(
                        format!(".config/hstr-rs/.{}_favorites", shell),
                        app.commands
                            .as_ref()
                            .unwrap()
                            .get(&app::View::Favorites)
                            .unwrap(),
                    )?;
                }
                TAB => {
                    let commands = app.get_commands();
                    let command = user_interface.get_selected(&commands);
                    util::echo(command);
                    break;
                }
                ENTER => {
                    let commands = app.get_commands();
                    let command = user_interface.get_selected(&commands);
                    util::echo(command);
                    util::echo("\n".to_string());
                    break;
                }
                CTRL_T => {
                    app.toggle_case();
                    user_interface.populate_screen(&app);
                }
                ESC => break,
                CTRL_SLASH => {
                    app.toggle_view();
                    user_interface.set_selected(0);
                    user_interface.populate_screen(&app);
                }
                _ => {
                    app.search_string
                        .push(std::char::from_u32(ch as u32).unwrap());
                    user_interface.set_selected(0);
                    user_interface.set_page(1);
                    app.search();
                    user_interface.populate_screen(&app);
                }
            },
            WchResult::KeyCode(code) => match code {
                KEY_UP => {
                    let commands = app.get_commands();
                    user_interface.move_selected(commands, -1);
                    user_interface.populate_screen(&app);
                }
                KEY_DOWN => {
                    let commands = app.get_commands();
                    user_interface.move_selected(commands, 1);
                    user_interface.populate_screen(&app);
                }
                KEY_BACKSPACE => {
                    app.search_string.pop();
                    app.restore();
                    app.search();
                    user_interface.populate_screen(&app);
                }
                KEY_DC => {
                    let commands = app.get_commands();
                    let command = user_interface.get_selected(&commands);
                    user_interface.prompt_for_deletion(&command);
                    if getch() == Y {
                        app.delete_from_history(command);
                        write_file(format!(".{}_history", shell), app.get_commands())?;
                    }
                    app.load_commands();
                    user_interface.populate_screen(&app);
                }
                KEY_NPAGE => {
                    let commands = app.get_commands();
                    user_interface.turn_page(commands, 1);
                    user_interface.populate_screen(&app);
                }
                KEY_PPAGE => {
                    let commands = app.get_commands();
                    user_interface.turn_page(commands, -1);
                    user_interface.populate_screen(&app);
                }
                _ => {}
            },
        }
    }
    endwin();
    Ok(())
}

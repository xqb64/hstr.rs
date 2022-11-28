use crate::ui::Direction;
use hstr::Shell;
use ncurses as nc;
use structopt::StructOpt;

mod hstr;
mod io;
mod sort;
mod state;
mod ui;

const CTRL_E: u32 = 5;
const TAB: u32 = 9;
const ENTER: u32 = 10;
const CTRL_T: u32 = 20;
const ESC: u32 = 27;

fn main() {
    let args = Opt::from_args();
    let result = run(args);

    if let Err(e) = result {
        eprintln!("hstr-rs error: {:?}", e);
    }
}

fn run(args: Opt) -> anyhow::Result<()> {
    /* If the --show-config option was passed, print config and exit. */
    if let Some(config_option) = args.show_config {
        let shell = Shell::from_str(&config_option)?;
        io::print_config(shell);
        return Ok(());
    }

    let query = args.query.unwrap_or_default();
    let mut user_interface = ui::UserInterface::new(&query).unwrap();

    ui::curses::init();

    /* If a search query was passed when hstr was started, search
     * and move the cursor to the end of the query. */
    if !query.is_empty() {
        user_interface.state.search();

        for _ in 0..query.chars().count() {
            user_interface.move_cursor(Direction::Forward);
        }
    }

    user_interface.populate_screen();

    loop {
        let user_input = nc::get_wch();

        match user_input.unwrap() {
            nc::WchResult::Char(ch) => match ch {
                CTRL_E => {
                    user_interface.state.toggle_search_mode();
                    user_interface.set_highlighted(0);
                    user_interface.populate_screen();
                }
                TAB => match user_interface.compute_highlighted() {
                    Some(command) => {
                        io::echo(command);
                        break;
                    }
                    None => continue,
                },
                ENTER => match user_interface.compute_highlighted() {
                    Some(command) => {
                        io::echo(command + "\n");
                        break;
                    }
                    None => continue,
                },
                CTRL_T => {
                    user_interface.state.toggle_case();
                    user_interface.populate_screen();
                }
                ESC => break,
                _ => {
                    let ch = std::char::from_u32(ch).unwrap();
                    user_interface
                        .state
                        .query
                        .insert_char(user_interface.get_cursor_position(), ch);
                    user_interface.set_highlighted(0);
                    user_interface.set_page(1);
                    nc::clear();
                    user_interface.state.search();
                    user_interface.populate_screen();
                    user_interface.move_cursor(Direction::Forward);
                }
            },
            nc::WchResult::KeyCode(code) => match code {
                nc::KEY_LEFT => user_interface.move_cursor(Direction::Backward),
                nc::KEY_RIGHT => user_interface.move_cursor(Direction::Forward),
                nc::KEY_UP => {
                    user_interface.move_highlighted(Direction::Backward);
                    user_interface.populate_screen();
                }
                nc::KEY_DOWN => {
                    user_interface.move_highlighted(Direction::Forward);
                    user_interface.populate_screen();
                }
                nc::KEY_BACKSPACE => {
                    if !user_interface.state.query.text.is_empty() {
                        user_interface
                            .state
                            .query
                            .remove_char(user_interface.get_cursor_position());
                    }
                    nc::clear();
                    user_interface.state.search();
                    user_interface.populate_screen();
                    user_interface.move_cursor(Direction::Backward);
                }
                nc::KEY_NPAGE => {
                    user_interface.turn_page(Direction::Forward);
                    user_interface.populate_screen();
                }
                nc::KEY_PPAGE => {
                    user_interface.turn_page(Direction::Backward);
                    user_interface.populate_screen();
                }
                nc::KEY_RESIZE => {
                    nc::clear();
                    user_interface.populate_screen();
                }
                _ => {}
            },
        }
    }

    ui::curses::teardown();

    Ok(())
}

#[derive(Debug, StructOpt)]
struct Opt {
    query: Option<String>,
    #[structopt(name = "show-config", long)]
    show_config: Option<String>,
}

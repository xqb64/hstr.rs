use crate::hstr::Shell;
use libc::{ioctl, TIOCSTI};
use std::fs::File;
use std::io::{Error, Read};
use std::path::Path;

pub fn read_as_bytes(path: impl AsRef<Path>) -> Result<Vec<u8>, Error> {
    /* `path` is relative to the home directory */
    let home = dirs::home_dir().unwrap();
    let target = home.join(path);
    let mut buffer = Vec::new();

    if target.exists() {
        let mut file = File::open(target)?;
        file.read_to_end(&mut buffer)?;
    }

    Ok(buffer)
}

pub fn echo(command: String) {
    for byte in command.as_bytes() {
        unsafe {
            ioctl(0, TIOCSTI, byte);
        }
    }
}

pub fn print_config(shell: Shell) {
    match shell {
        Shell::Bash => println!("{}", include_str!("config/bash")),
        Shell::Zsh => println!("{}", include_str!("config/zsh")),
    }
}

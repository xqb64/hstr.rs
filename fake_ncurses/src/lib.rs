use ncurses::{attr_t, NCURSES_ATTR_T};

#[allow(non_snake_case)]
pub const fn A_BOLD() -> attr_t {
    0
}

#[allow(non_snake_case)]
pub fn LINES() -> i32 {
    24
}

#[allow(non_snake_case)]
pub fn COLS() -> i32 {
    80
}

#[allow(non_snake_case)]
pub fn COLOR_PAIR(_n: i16) -> attr_t {
    0
}

pub const COLOR_BLACK: i16 = 0;
pub const COLOR_RED: i16 = 1;
pub const COLOR_GREEN: i16 = 2;
pub const COLOR_CYAN: i16 = 6;
pub const COLOR_WHITE: i16 = 7;

pub fn attron(_a: NCURSES_ATTR_T) -> i32 {
    0
}

pub fn attroff(_a: NCURSES_ATTR_T) -> i32 {
    0
}

pub fn clear() -> i32 {
    0
}

pub fn mvaddstr(_y: i32, _x: i32, _s: &str) -> i32 {
    0
}

pub fn mvaddch(_y: i32, _x: i32, _c: chtype) -> i32 {
    0
}

pub fn start_color() -> i32 {
    0
}

pub fn init_pair(_pair: i16, _f: i16, _b: i16) -> i32 {
    0
}

#[allow(non_camel_case_types)]
pub type chtype = u64;

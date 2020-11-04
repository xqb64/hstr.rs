# hstr-rs

**hstr** is shell history suggest box. Like hstr, but with pages.

It was initially made for bash, but it supports zsh and ksh, too. If you want to use it with tcsh, make sure tcsh saves its history to `~/.tcsh_history`.
​
## Installation
​
Make sure you have ncurses packages installed.
​
```
sudo apt install libncurses5 libncurses5-dev libncursesw5 libncursesw5-dev
```
​
Then run:
​
```
cargo install --git https://github.com/xvm32/hstr-rs.git
```
​
## Usage
​
Making an alias should be the most convenient option:

```sh
alias hh=hstr-rs
```
​
## Screencast

![screenshot](hstr-rs.gif)


# hstr-rs

![build status](https://github.com/xvm32/hstr-rs/workflows/CI/badge.svg) [![codecov](https://codecov.io/gh/xvm32/hstr-rs/branch/master/graph/badge.svg?token=0BZM100XU5)](https://codecov.io/gh/xvm32/hstr-rs)

**hstr-rs** is a shell history suggest box. Like hstr, but with pages. As opposed to original hstr which was the inspiration for this project, hstr-rs has pages and provides Unicode support out of the box on both bash and zsh.

hstr-rs was initially designed to be used with bash, but it also works on zsh, too. There is an ongoing effort to support other shells too. Contributors are very welcome.
​
## Installation
​
Make sure you have ncurses and readline packages installed.

If on Ubuntu:
​
```
sudo apt install libncurses5 libncurses5-dev libncursesw5 libncursesw5-dev libreadline5 libreadline-dev
```
​
Then run:
​
```
cargo install --git https://github.com/xvm32/hstr-rs.git
```
​
If on bash, add this to .bashrc:

```bash
# append new history items to .bash_history
shopt -s histappend 
# don't put duplicate lines or lines starting with space in the history
HISTCONTROL=ignoreboth
# increase history file size
HISTFILESIZE=1000000
# increase history size
HISTSIZE=${HISTFILESIZE}
# append new entries from memory to .bash_history
export PROMPT_COMMAND="history -a; ${PROMPT_COMMAND}"
```

## Usage
​
The most convenient option if you're using bash is to make the alias below:

```sh
alias hh=hstr-rs
```

Then invoke the program with `hh`.

## Screencast

![screenshot](hstr-rs.gif)

## Licensing

Licensed under the [MIT License](https://opensource.org/licenses/MIT). For details, see [LICENSE](https://github.com/xvm32/hstr-rs/blob/master/LICENSE).

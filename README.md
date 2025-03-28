# hstr-rs

![build status](https://github.com/xqb64/hstr-rs/workflows/build/badge.svg)

![screenshot](screencast.gif)

**hstr-rs** is a shell history suggest box. Like hstr, but with pages. As opposed to original hstr which was the inspiration for this project, hstr-rs has pages and provides Unicode support out of the box on both bash and zsh.

There is an ongoing effort to support other shells too. Contributors are very welcome.
​
## Installation
​
Make sure you have a Rust compiler and ncurses packages.

If on Ubuntu, run:
​
```
sudo apt install libncurses5 libncurses5-dev libncursesw5 libncursesw5-dev
```
​
Then run:
​
```
cargo install --git https://github.com/xqb64/hstr-rs.git
```
​
If on bash, run:

```
hstr-rs --show-config bash >> ~/.bashrc
```

...or manually add [these lines](src/config/bash) to your `.bashrc`.

For zsh, run:

```
hstr-rs --show-config zsh >> ~/.zshrc
```
...or manually add [these lines](src/config/zsh) to your `.zshrc`.

## Usage
​
The most convenient is to make the alias:

```sh
alias hh='hstr-rs'
```

Then invoke the program with `hh`.

## Licensing

Licensed under the [MIT License](https://opensource.org/licenses/MIT). For details, see [LICENSE](https://github.com/xqb64/hstr-rs/blob/master/LICENSE).

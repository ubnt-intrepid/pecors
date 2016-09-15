extern crate pecors;
extern crate regex;

use std::io::BufRead;
use regex::Regex;

fn read_lines() -> Vec<String> {
  let ansi = Regex::new(r"\x1B\[([0-9]{1,2}(;[0-9]{1,2})?)?[m|K]").unwrap();
  let stdin = std::io::stdin();
  let lines = stdin.lock()
    .lines()
    .map(|line| ansi.replace_all(&line.unwrap(), ""))
    .collect();
  lines
}

fn main() {
  let lines = read_lines();
  let mut cli = pecors::Client::new(lines);

  let selected = cli.select_line().unwrap();
  match selected {
    Some(line) => println!("{}", line),
    None => (),
  }
}

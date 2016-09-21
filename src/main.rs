extern crate pecors;
extern crate regex;

use std::io::BufRead;
use regex::Regex;

pub fn read_lines<R: BufRead>(reader: R) -> Vec<String> {
  let ansi = Regex::new(r"\x1B\[([0-9]{1,2}(;[0-9]{1,2})?)?[m|K]").unwrap();
  let lines = reader.lines()
    .map(|line| ansi.replace_all(&line.unwrap(), ""))
    .filter_map(|line| {
      let trimmed = line.trim();
      if trimmed.len() != 0 {
        Some(trimmed.to_owned())
      } else {
        None
      }
    })
    .collect();
  lines
}

fn main() {
  let stdin = std::io::stdin();
  let lines = read_lines(stdin.lock());
  let mut cli = pecors::Client::new(lines);

  let selected = cli.select_line().unwrap();
  match selected {
    Some(line) => println!("{}", line),
    None => (),
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use std::io::{self, Cursor};

  #[test]
  fn read_lines_simple_case() {
    let input = r#"
hogehoge
征夷大将軍
"#;
    let lines = read_lines(io::BufReader::new(Cursor::new(input.as_bytes())));

    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "hogehoge");
    assert_eq!(lines[1], "征夷大将軍");
  }

  #[test]
  fn read_lines_includes_ansi_code() {
    let input = "
\x1b[0m\x1b[01;34msrc\x1b[0m/
\x1b[01;34mtarget\x1b[0m/
\x1b[0m[01;34m💰\x1b[0m
";
    let lines = read_lines(io::BufReader::new(Cursor::new(input.as_bytes())));

    assert_eq!(lines[0], "src/");
    assert_eq!(lines[1], "target/");
    assert_eq!(lines[2], "💰");
  }
}

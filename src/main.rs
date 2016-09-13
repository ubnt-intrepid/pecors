extern crate rustbox;
extern crate regex;

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::cmp::{min, max};
use std::io::BufRead;
use rustbox::{Color, RustBox, Key};
use rustbox::Event::KeyEvent;
use regex::Regex;

enum Status {
  Selected(String),
  Escaped,
  Continue,
}
use Status::*;

trait PrintLine {
  fn print_line(&self, y: usize, item: &str, fg: Color, bg: Color);
}

impl PrintLine for RustBox {
  fn print_line(&self, y: usize, item: &str, fg: Color, bg: Color) {
    for x in 0..(self.width()) {
      let ch = item.chars().nth(x).unwrap_or(' ');
      self.print_char(x, y, rustbox::RB_NORMAL, fg, bg, ch);
    }
  }
}

struct PecorsClient {
  prompt: String,
  y_offset: usize,
  stdin: Vec<String>,

  filtered: Vec<String>,
  query: String,
  cursor: usize,
  offset: usize,
}

impl PecorsClient {
  fn new(stdin: Vec<String>) -> PecorsClient {
    let mut cli = PecorsClient {
      stdin: stdin,
      filtered: Vec::new(),
      query: String::new(),
      prompt: "QUERY> ".to_owned(),
      y_offset: 1,
      cursor: 0,
      offset: 0,
    };
    cli.filtered = cli.stdin.clone();
    cli
  }

  fn run(&mut self) -> Option<String> {
    let term = RustBox::init(Default::default()).unwrap();

    self.render_items(&term);
    loop {
      match term.poll_event(false) {
        Err(err) => panic!("Error during handle event: {:?}", err),
        Ok(event) => {
          match self.handle_event(&term, event) {
            Selected(s) => return Some(s),
            Escaped => break,
            _ => (),
          }
        }
      }
      self.render_items(&term);
    }
    None
  }

  fn handle_event(&mut self, term: &RustBox, event: rustbox::Event) -> Status {
    match event {
      KeyEvent(Key::Enter) => {
        return if self.filtered.len() > 0 {
          Selected(self.selected_text())
        } else {
          Escaped
        }
      }
      KeyEvent(Key::Esc) => return Escaped,
      KeyEvent(Key::Up) => self.cursor_up(),
      KeyEvent(Key::Down) => self.cursor_down(term.height()),
      KeyEvent(Key::Backspace) => self.remove_query(),
      KeyEvent(Key::Char(c)) => self.append_query(c),
      _ => (),
    }
    Continue
  }

  fn query_str(&self) -> String {
    format!("{}{}", self.prompt, self.query)
  }

  fn selected_text(&self) -> String {
    self.filtered[self.cursor + self.offset].clone()
  }

  fn append_query(&mut self, c: char) {
    self.query.push(c);
    self.apply_filter();
  }

  fn remove_query(&mut self) {
    if !self.query.is_empty() {
      let idx = self.query.len() - 1;
      self.query.remove(idx);
      self.apply_filter();
    }
  }

  fn cursor_up(&mut self) {
    if self.cursor == 0 {
      if self.offset > 0 {
        self.offset -= 1
      }
    } else {
      self.cursor -= 1;
    }
  }

  fn cursor_down(&mut self, height: usize) {
    if self.cursor == height - self.y_offset - 1 {
      self.offset = min(self.offset + 1,
                        (max(0,
                             (self.filtered.len() as isize) - (height as isize) +
                             (self.y_offset as isize)) as usize));
    } else {
      self.cursor = min(self.cursor + 1,
                        min(self.filtered.len() - self.offset - 1,
                            height - self.y_offset - 1));
    }
  }

  fn apply_filter(&mut self) {
    self.filtered = if self.query.len() == 0 {
      self.stdin.clone()
    } else {
      let re = Regex::new(self.query.as_str()).unwrap();
      self.stdin.iter().filter(|&input| re.is_match(input)).cloned().collect()
    };

    self.cursor = 0;
    self.offset = 0;
  }

  fn render_items(&self, term: &RustBox) {
    term.clear();

    let query_str = self.query_str();
    term.print_line(0, &query_str, Color::White, Color::Black);

    for (y, item) in self.filtered.iter().skip(self.offset).enumerate() {
      if y == self.cursor {
        term.print_line(y + self.y_offset, item, Color::Red, Color::White);
      } else {
        term.print_line(y + self.y_offset, item, Color::White, Color::Black);
      }
    }

    term.set_cursor(query_str.len() as isize, 0);

    term.present();
  }
}


fn main() {
  // make filterd list from stdin.
  let ansi = Regex::new(r"\x1B\[([0-9]{1,2}(;[0-9]{1,2})?)?[m|K]").unwrap();
  let stdin = std::io::stdin();
  let inputs = stdin.lock()
    .lines()
    .map(|line| ansi.replace_all(&line.unwrap(), ""))
    .collect();

  let mut cli = PecorsClient::new(inputs);
  let selected = catch_unwind(AssertUnwindSafe(move || cli.run()));

  match selected {
    Ok(Some(selected)) => println!("{}", selected),
    Ok(None) => (),
    Err(err) => panic!("Error: {:?}", err),
  }
}

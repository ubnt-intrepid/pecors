extern crate rustbox;
extern crate regex;

use std::default::Default;
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


struct PecorsClient {
  term: RustBox,
  query_header: String,

  query: String,

  stdin: Vec<String>,
  rendered: Vec<String>,
  filtered: Vec<String>,
  selected: isize,
  cursor: isize,
  offset: usize,
}

impl PecorsClient {
  fn new(stdin: Vec<String>, term: RustBox) -> PecorsClient {
    PecorsClient {
      term: term,
      stdin: stdin,
      rendered: Vec::new(),
      filtered: Vec::new(),
      query: String::new(),
      query_header: "QUERY> ".to_owned(),
      selected: 0,
      cursor: 0,
      offset: 0,
    }
  }

  fn select_item(&mut self) -> Option<String> {
    self.apply_filter();
    loop {
      self.render_items();
      match self.term.poll_event(false) {
        Err(err) => panic!("Error during handle event: {:?}", err),
        Ok(event) => {
          match self.handle_event(event) {
            Selected(s) => return Some(s),
            Escaped => break,
            Continue => continue,
          }
        }
      }
    }
    None
  }

  fn handle_event(&mut self, event: rustbox::Event) -> Status {
    match event {
      KeyEvent(Key::Enter) => return Selected(self.selected_text()),
      KeyEvent(Key::Esc) => return Escaped,
      KeyEvent(Key::Up) => self.cursor_up(),
      KeyEvent(Key::Down) => self.cursor_down(),
      KeyEvent(Key::Backspace) => self.remove_query(),
      KeyEvent(Key::Char(c)) => self.append_query(c),
      _ => (),
    }
    Continue
  }

  fn coord_offset(&self) -> usize {
    1
  }

  fn query_str(&self) -> String {
    format!("{}{}", self.query_header, self.query)
  }

  fn selected_text(&self) -> String {
    self.rendered[self.offset + self.selected as usize].clone()
  }

  fn selected_coord(&self) -> usize {
    self.selected as usize
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
    if self.selected > -1 {
      self.selected -= 1;
    }
    if self.cursor > 0 {
      self.cursor -= 1;
    }

    if self.selected == -1 {
      self.selected += 1;
      if self.offset > 0 {
        self.offset -= 1;
        self.rendered = Vec::from(&self.filtered[(self.offset)..]);
      }
    }
  }

  fn cursor_down(&mut self) {
    let height = self.term.height();

    if (self.cursor as usize) < self.rendered.len() - 1 {
      self.cursor += 1;
    }

    if ((self.rendered.len() < height - 1) && (self.selected_coord() < self.rendered.len())) ||
       ((self.rendered.len() > height - 1) && (self.selected_coord() < height - 1)) {
      self.selected += 1;
    }

    if self.selected_coord() == height - 1 {
      self.selected -= 1;
      if self.offset < self.filtered.len() - 1 {
        self.offset += 1;
        self.rendered = Vec::from(&self.filtered[(self.offset)..]);
      }
    }
  }

  fn apply_filter(&mut self) {
    self.filtered = if self.query.len() == 0 {
      self.stdin.clone()
    } else {
      let re = Regex::new(self.query.as_str()).unwrap();
      self.stdin.iter().filter(|&input| re.is_match(input)).cloned().collect()
    };

    self.rendered = self.filtered.clone();
    self.selected = 0;
    self.cursor = 0;
    self.offset = 0;
  }

  fn render_items(&self) {
    self.term.clear();

    let query_str = self.query_str();
    self.print_line(0, &query_str, Color::White, Color::Black);
    self.term.print_char(query_str.len(),
                         0,
                         rustbox::RB_NORMAL,
                         Color::White,
                         Color::White,
                         ' ');

    for (y, item) in self.rendered.iter().enumerate() {
      if y == self.selected_coord() {
        self.print_line(y + self.coord_offset(), item, Color::Red, Color::White);
      } else {
        self.print_line(y + self.coord_offset(), item, Color::White, Color::Black);
      }
    }

    self.term.present();
  }

  fn print_line(&self, y: usize, item: &str, fg: Color, bg: Color) {
    for x in 0..(self.term.width()) {
      let ch = item.chars().nth(x).unwrap_or(' ');
      self.term.print_char(x, y, rustbox::RB_NORMAL, fg, bg, ch);
    }
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

  let term = RustBox::init(Default::default()).unwrap();

  let selected = PecorsClient::new(inputs, term).select_item();
  match selected {
    Some(selected) => println!("{}", selected),
    None => (),
  }
}

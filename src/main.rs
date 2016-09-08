extern crate rustbox;
extern crate regex;

use std::default::Default;
use std::io::BufRead;
use rustbox::{Color, RustBox, Key};
use rustbox::Event::KeyEvent;

enum Status {
  Selected(String),
  Escaped,
  Continue,
}
use Status::*;


struct PecorsClient {
  term: RustBox,
  inputs: Vec<String>,
  render_items: Vec<String>,
  filtered: Vec<String>,
  query: String,
  query_header: String,
  selected: isize,
  cursor: isize,
  offset: usize,
}

impl PecorsClient {
  fn new(inputs: Vec<String>, term: RustBox) -> PecorsClient {
    PecorsClient {
      term: term,
      inputs: inputs,
      render_items: Vec::new(),
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
      self.render();

      let selected = self.handle_event();
      match selected {
        Selected(s) => return Some(s),
        Escaped => break,
        Continue => continue,
      }
    }
    None
  }

  fn handle_event(&mut self) -> Status {
    match self.term.poll_event(false) {
      Ok(KeyEvent(Key::Enter)) => {
        return Selected(self.render_items[self.offset + self.selected as usize].clone())
      }
      Ok(KeyEvent(Key::Esc)) => return Escaped,
      Ok(KeyEvent(Key::Up)) => self.cursor_up(),
      Ok(KeyEvent(Key::Down)) => self.cursor_down(),
      Ok(KeyEvent(Key::Backspace)) => self.remove_query(),
      Ok(KeyEvent(Key::Char(c))) => self.append_query(c),
      Ok(_) => (),
      Err(err) => panic!("Error during handle event: {:?}", err),
    }
    Continue
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
        self.render_items = Vec::from(&self.filtered[(self.offset as usize)..]);
      }
    }
  }

  fn cursor_down(&mut self) {
    if self.cursor < (self.render_items.len() - 1) as isize {
      self.cursor += 1;
    }
    if (self.render_items.len() < self.term.height() - 1) &&
       (self.selected < self.render_items.len() as isize) {
      self.selected += 1;
    } else if (self.render_items.len() > self.term.height() - 1) &&
              (self.selected < (self.term.height() - 1) as isize) {
      self.selected += 1;
    }

    if self.selected == (self.term.height() - 1) as isize {
      self.selected -= 1;
      if self.offset < self.filtered.len() - 1 {
        self.offset += 1;
        self.render_items = Vec::from(&self.filtered[(self.offset as usize)..]);
      }
    }
  }

  fn apply_filter(&mut self) {
    self.filtered = self.filter_by_regex();
    self.render_items = self.filtered.clone();
    self.selected = 0;
    self.cursor = 0;
    self.offset = 0;
  }

  fn filter_by_regex(&self) -> Vec<String> {
    if self.query.len() == 0 {
      self.inputs.clone()
    } else {
      let re = regex::Regex::new(self.query.as_str()).unwrap();
      self.inputs.iter().filter(|&input| re.is_match(input)).cloned().collect()
    }
  }

  fn render(&self) {
    self.term.clear();

    self.print_query();
    for (y, item) in self.render_items.iter().enumerate() {
      self.print_line(y, item.clone(), y as isize == self.selected);
    }

    self.term.present();
  }

  fn print_query(&self) {
    let query_str: String = format!("{}{}", self.query_header, self.query);

    for x in 0..(self.term.width()) {
      let ch = query_str.chars().nth(x).unwrap_or(' ');
      let (fg, bg) = if x == query_str.len() {
        (Color::White, Color::White)
      } else {
        (Color::White, Color::Black)
      };
      self.term.print_char(x, 0, rustbox::RB_NORMAL, fg, bg, ch);
    }
  }

  fn print_line(&self, y: usize, item: String, selected: bool) {
    let y_offset = 1;
    let (fg, bg) = if selected {
      (Color::Red, Color::White)
    } else {
      (Color::White, Color::Black)
    };

    for x in 0..(self.term.width()) {
      let ch = item.chars().nth(x).unwrap_or(' ');
      self.term.print_char(x, y + y_offset, rustbox::RB_NORMAL, fg, bg, ch);
    }
  }
}


fn main() {
  // make filterd list from stdin.
  let stdin = std::io::stdin();
  let inputs = stdin.lock().lines().map(|line| line.unwrap()).collect();

  let term = RustBox::init(Default::default()).unwrap();

  let selected = PecorsClient::new(inputs, term).select_item();
  match selected {
    Some(selected) => println!("{}", selected),
    None => (),
  }
}

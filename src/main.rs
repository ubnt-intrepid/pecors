extern crate rustbox;
extern crate regex;

use std::default::Default;
use rustbox::{Color, RustBox, Event, Key};
use std::io::BufRead;

struct PecorsClient {
  term: Option<RustBox>,
  inputs: Vec<String>,
  render_items: Vec<String>,
  filtered: Vec<String>,
  query: String,
  selected: isize,
  cursor: isize,
  offset: usize,
  selected_str: Option<String>,
}

impl PecorsClient {
  fn new() -> PecorsClient {
    PecorsClient {
      term: None,
      inputs: Vec::new(),
      render_items: Vec::new(),
      filtered: Vec::new(),
      query: String::new(),
      selected: 0,
      cursor: 0,
      offset: 0,
      selected_str: None,
    }
  }

  fn init(&mut self) {
    self.inputs = {
      let stdin = std::io::stdin();
      let mut inputs = Vec::new();
      for line in stdin.lock().lines() {
        let line = line.unwrap();
        inputs.push(line);
      }
      inputs
    };

    self.term = Some(RustBox::init(Default::default()).unwrap());
    self.apply_filter();
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


  fn select_item(&mut self) -> Option<String> {
    self.init();

    loop {
      if self.handle_event() {
        break;
      }
    }

    self.term = None;
    self.selected_str.clone()
  }

  fn handle_event(&mut self) -> bool {
    self.update_items();

    let mut is_update = false;

    let quited = match self.term {
      Some(ref mut term) => {
        match term.poll_event(false) {
          Ok(Event::KeyEvent(key)) => {
            match key {
              Key::Enter => {
                self.selected_str = Some(self.render_items[self.offset + self.selected as usize]
                  .clone());
                true
              }
              Key::Esc => true,
              Key::Backspace => {
                if ! self.query.is_empty() {
                  let idx = self.query.len() - 1;
                  self.query.remove(idx);
                  is_update = true;
                }
                false
              }

              Key::Up => {
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
                false
              }

              Key::Down => {
                if self.cursor < (self.render_items.len() - 1) as isize {
                  self.cursor += 1;
                }
                if (self.render_items.len() < term.height() - 1) &&
                   (self.selected < self.render_items.len() as isize) {
                  self.selected += 1;
                } else if (self.render_items.len() > term.height() - 1) &&
                          (self.selected < (term.height() - 1) as isize) {
                  self.selected += 1;
                }

                if self.selected == (term.height() - 1) as isize {
                  self.selected -= 1;
                  if self.offset < self.filtered.len() - 1 {
                    self.offset += 1;
                    self.render_items = Vec::from(&self.filtered[(self.offset as usize)..]);
                  }
                }
                false
              }

              Key::Char(c) => {
                self.query.push(c);
                is_update = true;
                false
              }

              _ => false,
            }
          }
          _ => false,
        }
      }
      None => true,
    };

    if is_update {
      self.apply_filter();
    }

    quited
  }

  fn update_items(&self) {
    if let Some(ref term) = self.term {
      term.clear();

      self.print_query();
      for (y, item) in self.render_items.iter().enumerate() {
        self.print_line(y, item.clone(), y as isize == self.selected);
      }

      term.present();
    }
  }

  fn print_query(&self) {
    match self.term {
      Some(ref t) => {
        let query_str: String = format!("QUERY> {}", self.query);

        let width = t.width();
        for x in 0..width {
          let c = query_str.chars().nth(x).unwrap_or(' ');
          if x == query_str.len() {
            t.print_char(x, 0, rustbox::RB_NORMAL, Color::White, Color::White, c);
          } else {
            t.print_char(x, 0, rustbox::RB_NORMAL, Color::White, Color::Black, c);
          }
        }
      }
      _ => panic!(""),
    }
  }

  fn print_line(&self, y: usize, item: String, selected: bool) {
    match self.term {
      Some(ref t) => {
        let y_offset = 1;

        let width = t.width();
        for x in 0..width {
          let c = item.chars().nth(x).unwrap_or(' ');
          if selected {
            t.print_char(x,
                         y + y_offset,
                         rustbox::RB_NORMAL,
                         Color::Red,
                         Color::White,
                         c);
          } else {
            t.print_char(x,
                         y + y_offset,
                         rustbox::RB_NORMAL,
                         Color::White,
                         Color::Black,
                         c);
          }
        }
      }
      _ => panic!(""),
    }

  }
}

fn main() {
  let mut cli = PecorsClient::new();
  if let Some(selected) = cli.select_item() {
    println!("{}", selected);
  }
}

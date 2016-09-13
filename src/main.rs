extern crate rustbox;
extern crate regex;

use std::cmp::{min, max};
use std::io::BufRead;
use rustbox::{Color, RustBox, Key};
use rustbox::Event::KeyEvent;
use regex::Regex;

#[derive(Debug)]
enum PecorsError {
  RustBox(rustbox::InitError),
  Regex(regex::Error),
}

impl From<rustbox::InitError> for PecorsError {
  fn from(err: rustbox::InitError) -> PecorsError {
    PecorsError::RustBox(err)
  }
}

impl From<regex::Error> for PecorsError {
  fn from(err: regex::Error) -> PecorsError {
    PecorsError::Regex(err)
  }
}

type PecorsResult<T> = Result<T, PecorsError>;


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
  lines: Vec<String>,

  filtered: Vec<String>,
  query: String,
  cursor: usize,
  offset: usize,
}

impl PecorsClient {
  fn new(lines: Vec<String>) -> PecorsClient {
    let filtered = lines.clone();
    PecorsClient {
      lines: lines,
      filtered: filtered,
      query: String::new(),
      prompt: "QUERY> ".to_owned(),
      y_offset: 1,
      cursor: 0,
      offset: 0,
    }
  }

  // choices a line from `lines` interactively.
  fn select_line(&mut self) -> PecorsResult<Option<String>> {
    let term = try!(RustBox::init(Default::default()));

    self.render_items(&term);
    loop {
      match term.poll_event(false) {
        Err(err) => panic!("Error during handle event: {:?}", err),
        Ok(event) => {
          match try!(self.handle_event(&term, event)) {
            Selected(s) => return Ok(Some(s)),
            Escaped => break,
            _ => (),
          }
        }
      }
      self.render_items(&term);
    }
    Ok(None)
  }

  fn handle_event(&mut self, term: &RustBox, event: rustbox::Event) -> PecorsResult<Status> {
    match event {
      KeyEvent(Key::Enter) => {
        if self.filtered.len() > 0 {
          Ok(Selected(self.filtered[self.cursor + self.offset].clone()))
        } else {
          Ok(Escaped)
        }
      }
      KeyEvent(Key::Esc) => Ok(Escaped),
      KeyEvent(Key::Up) => {
        self.cursor_up();
        Ok(Continue)
      }
      KeyEvent(Key::Down) => {
        self.cursor_down(term.height());
        Ok(Continue)
      }
      KeyEvent(Key::Backspace) => self.remove_query().and(Ok(Continue)),
      KeyEvent(Key::Char(c)) => self.append_query(c).and(Ok(Continue)),
      _ => Ok(Continue),
    }
  }

  fn append_query(&mut self, c: char) -> PecorsResult<()> {
    self.query.push(c);
    self.apply_filter()
  }

  fn remove_query(&mut self) -> PecorsResult<()> {
    if self.query.is_empty() {
      return Ok(());
    }

    let idx = self.query.len() - 1;
    self.query.remove(idx);
    self.apply_filter()
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

  fn apply_filter(&mut self) -> PecorsResult<()> {
    self.filtered = if self.query.len() == 0 {
      self.lines.clone()
    } else {
      let re = try!(Regex::new(self.query.as_str()));
      self.lines.iter().filter(|&input| re.is_match(input)).cloned().collect()
    };

    self.cursor = 0;
    self.offset = 0;

    Ok(())
  }

  fn render_items(&self, term: &RustBox) {
    term.clear();

    let query_str = format!("{}{}", self.prompt, self.query);
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

  let selected = cli.select_line().unwrap();
  match selected {
    Some(line) => println!("{}", line),
    None => (),
  }
}

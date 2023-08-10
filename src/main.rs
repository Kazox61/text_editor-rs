use std::result;
use std::io::{Write, stdout};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use crossterm::{ExecutableCommand, QueueableCommand};
use crossterm::event::{poll, read, Event, self, KeyEvent, KeyCode, KeyModifiers, KeyEventKind};
use crossterm::terminal::{self, ClearType};
use crossterm::style::{self, Stylize, SetBackgroundColor, Print, style};
use crossterm::cursor;

struct Ui {
    text: Vec<String>,
    cursor_x: usize,
    cursor_y: usize,
    stdout: io::Stdout
}

impl Ui {
    fn init() -> Ui {
        Ui {
            text: vec![
                "Hello World".to_string(),
                "Test123".to_string(),
                "Finish123".to_string()
            ],
            cursor_x: 0,
            cursor_y: 0,
            stdout: io::stdout()
        }
    }

    fn render(&mut self) {
        self.stdout.queue(terminal::Clear(ClearType::All));
        self.stdout.queue(cursor::MoveTo(0, 0));

        for (row_num, row) in self.text.iter().enumerate() {
            self.stdout.queue(style::Print(format!("{}{}", row, "\n")));
        }
        self.stdout.flush();
    }

    fn update_current_line(&mut self) {
        let mut before_cursor = String::new();
        let mut cursor_char = String::new();
        let mut after_cursor = String::new();
        let mut cursor_found = false;
        for (col_num, c) in self.text[self.cursor_y].chars().enumerate() {
            if !cursor_found {
                if col_num != self.cursor_x {
                    before_cursor.push(c);
                }
                else {
                    cursor_char.push(c);
                    cursor_found = true;
                }
            }
            else {
                after_cursor.push(c);
            }
        }
        self.stdout.queue(style::ResetColor);
        self.stdout.queue(style::Print(before_cursor));
        self.stdout.queue(style::SetBackgroundColor(style::Color::White));
        if cursor_char.len() > 0 {
            self.stdout.queue(style::PrintStyledContent(cursor_char.black()));
        }
        else {
            self.stdout.queue(style::PrintStyledContent(" ".black()));
        }
        self.stdout.queue(style::SetBackgroundColor(style::Color::Reset));
        self.stdout.queue(style::Print(format!("{}{}", after_cursor, "\n")));
    }

    fn move_cursor_up(&mut self) {
        if self.cursor_y == 0 {
            return;
        }
        self.stdout.queue(cursor::MoveTo(0, self.cursor_y as u16));
        self.stdout.queue(terminal::Clear(terminal::ClearType::CurrentLine));
        self.stdout.queue(style::Print(format!("{}{}", self.text[self.cursor_y], "\n")));

        self.cursor_y = self.cursor_y-1;
        self.cursor_x = std::cmp::min(self.cursor_x, self.text[self.cursor_y].len());

        self.stdout.queue(cursor::MoveTo(0, self.cursor_y as u16));
        self.update_current_line();
        self.stdout.flush();
    }

    fn move_cursor_down(&mut self) {
        if self.cursor_y == self.text.len() - 1 {
            return;
        }
        self.stdout.queue(cursor::MoveTo(0, self.cursor_y as u16));
        self.stdout.queue(terminal::Clear(terminal::ClearType::CurrentLine));
        self.stdout.queue(style::Print(format!("{}{}", self.text[self.cursor_y], "\n")));

        self.cursor_y = std::cmp::min(self.text.len()-1, self.cursor_y+1);
        self.cursor_x = std::cmp::min(self.cursor_x, self.text[self.cursor_y].len());

        self.update_current_line();
        self.stdout.flush();
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_x == 0 {
            return;
        }

        self.stdout.queue(cursor::MoveTo(0, self.cursor_y as u16));
        self.stdout.queue(terminal::Clear(terminal::ClearType::CurrentLine));
        
        self.cursor_x = self.cursor_x-1;

        self.update_current_line();
        self.stdout.flush();
    }

    fn move_cursor_right(&mut self) {
        self.stdout.queue(cursor::MoveTo(0, self.cursor_y as u16));
        self.stdout.queue(terminal::Clear(terminal::ClearType::CurrentLine));

        self.cursor_x = std::cmp::min(self.text[self.cursor_y].len(), self.cursor_x+1);

        self.update_current_line();
        self.stdout.flush();
    }

    fn insert_char(&mut self, c: char) {
        let mut row = self.text[self.cursor_y].clone();
        row.insert(self.cursor_x, c);
        self.text[self.cursor_y] = row;
        self.cursor_x += 1;

        self.stdout.queue(cursor::MoveTo(0, self.cursor_y as u16));
        self.stdout.queue(terminal::Clear(terminal::ClearType::CurrentLine));
        
        self.update_current_line();
        self.stdout.flush();
    }

    fn enter(&mut self) {
        let mut current_row = self.text[self.cursor_y].clone();
        let (first, last) = current_row.split_at(self.cursor_x);
        self.text[self.cursor_y] = first.to_string();
        self.text.insert(self.cursor_y+1, last.to_string());

        self.render();
        self.cursor_y += 1;
        self.cursor_x = 0;

        self.stdout.queue(cursor::MoveTo(0, self.cursor_y as u16));
        self.update_current_line();
        self.stdout.flush();
    }
}

fn on_start() {

}

fn on_update(ui: &mut Ui) {
    if let event::Event::Key(KeyEvent { code, state, modifiers, kind }) = event::read().unwrap() {

        match code {
            KeyCode::Char(c) => {
                if kind == KeyEventKind::Release {
                    ui.insert_char(c);
                }
            }
            _ => {}
        }

        if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('q')  {
            std::process::exit(0);
        }
        else if code == KeyCode::Enter && kind == KeyEventKind::Release {
            ui.enter();
        }
        else if code == KeyCode::Up && kind == KeyEventKind::Release {
            ui.move_cursor_up();
        }
        else if code == KeyCode::Down && kind == KeyEventKind::Release {
            ui.move_cursor_down();
        }
        else if code == KeyCode::Left && kind == KeyEventKind::Release {
            ui.move_cursor_left();
        }
        else if code == KeyCode::Right && kind == KeyEventKind::Release {
            ui.move_cursor_right();
        }
    }
}


fn main()  {
    terminal::enable_raw_mode().unwrap();
    let mut ui = Ui::init();
    ui.render();
    ui.stdout.execute(cursor::MoveTo(0, ui.cursor_y as u16));
    ui.update_current_line();

    loop {
        on_update(&mut ui);
    }
}

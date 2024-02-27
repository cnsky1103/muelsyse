use std::io::{Stdout, Write};

use anyhow::Result;
use crossterm::{
    cursor::{self, SetCursorStyle},
    event::{self, read, KeyCode},
    style::{self},
    terminal, ExecutableCommand, QueueableCommand,
};

enum Mode {
    Normal,
    Insert,
}

impl Mode {
    pub fn get_cursor_style(&self) -> SetCursorStyle {
        match self {
            Self::Normal => SetCursorStyle::SteadyBlock,
            Self::Insert => SetCursorStyle::BlinkingBar,
        }
    }
}

enum Action {
    Quit,
    ChangeMode(Mode),
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    AddChar(char),
}

struct Cursor {
    x: u16,
    y: u16,
}

struct Editor {
    mode: Mode,
    stdout: Stdout,
    cursor: Cursor,
    size: (u16, u16),
}

impl Editor {
    pub fn new() -> Self {
        Self {
            mode: Mode::Normal,
            stdout: std::io::stdout(),
            cursor: Cursor { x: 0, y: 0 },
            size: terminal::size().unwrap(),
        }
    }

    pub fn draw(&mut self) -> Result<()> {
        self.stdout
            .queue(cursor::MoveTo(self.cursor.x, self.cursor.y))?;
        self.stdout.flush()?;
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;

        self.stdout
            .execute(terminal::EnterAlternateScreen)?
            .execute(terminal::Clear(terminal::ClearType::All))?
            .execute(self.mode.get_cursor_style())?;

        loop {
            self.draw()?;
            if let Some(action) = self.handle_event(read()?)? {
                match action {
                    Action::Quit => break,
                    Action::ChangeMode(m) => {
                        self.stdout.queue(m.get_cursor_style())?;
                        self.mode = m;
                    }
                    Action::MoveUp => self.cursor.y = self.cursor.y.saturating_sub(1),
                    Action::MoveDown => self.cursor.y += 1,
                    Action::MoveLeft => self.cursor.x = self.cursor.x.saturating_sub(1),
                    Action::MoveRight => self.cursor.x += 1,
                    Action::AddChar(c) => {
                        self.stdout
                            .queue(cursor::MoveTo(self.cursor.x, self.cursor.y))?;
                        self.stdout.queue(style::Print(c))?;
                        self.cursor.x += 1;
                        if self.cursor.x > self.size.0 {
                            self.cursor.y += 1;
                            self.cursor.x = 0;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_event(&mut self, e: event::Event) -> Result<Option<Action>> {
        match self.mode {
            Mode::Normal => self.handle_normal_event(e),
            Mode::Insert => self.handle_insert_event(e),
        }
    }

    fn handle_insert_event(&mut self, e: event::Event) -> Result<Option<Action>> {
        match e {
            event::Event::Key(e) => match e.code {
                KeyCode::Esc => Ok(Some(Action::ChangeMode(Mode::Normal))),
                KeyCode::Char(c) => Ok(Some(Action::AddChar(c))),
                KeyCode::Up => Ok(Some(Action::MoveUp)),
                KeyCode::Down => Ok(Some(Action::MoveDown)),
                KeyCode::Left => Ok(Some(Action::MoveLeft)),
                KeyCode::Right => Ok(Some(Action::MoveRight)),
                _ => Ok(None),
            },
            _ => Ok(None),
        }
    }

    fn handle_normal_event(&mut self, e: event::Event) -> Result<Option<Action>> {
        match e {
            event::Event::Key(e) => match e.code {
                KeyCode::Char('q') => Ok(Some(Action::Quit)),
                KeyCode::Char('i') => Ok(Some(Action::ChangeMode(Mode::Insert))),
                KeyCode::Up | KeyCode::Char('k') => Ok(Some(Action::MoveUp)),
                KeyCode::Down | KeyCode::Char('j') => Ok(Some(Action::MoveDown)),
                KeyCode::Left | KeyCode::Char('h') => Ok(Some(Action::MoveLeft)),
                KeyCode::Right | KeyCode::Char('l') => Ok(Some(Action::MoveRight)),

                _ => Ok(None),
            },
            _ => Ok(None),
        }
    }
}

fn main() -> Result<()> {
    Editor::new().run()
}

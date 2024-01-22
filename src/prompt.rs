use arboard::Clipboard;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Text,
    widgets::{Block, BorderType, Borders},
    Frame,
};
use tui_textarea::{CursorMove, TextArea};
use unicode_width::UnicodeWidthStr;

use crate::app::FocusedBlock;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
}

pub struct Prompt<'a> {
    pub mode: Mode,
    pub previous_key: KeyCode,
    pub formatted_prompt: Text<'a>,
    pub editor: TextArea<'a>,
    pub block: Block<'a>,
}

impl Default for Prompt<'_> {
    fn default() -> Self {
        let mut editor = TextArea::default();
        editor.remove_line_number();
        editor.set_cursor_line_style(Style::default());
        editor.set_selection_style(Style::default().bg(Color::DarkGray));

        let block = Block::default()
            .border_type(BorderType::Thick)
            .borders(Borders::ALL)
            .style(Style::default());

        Self {
            mode: Mode::Normal,
            previous_key: KeyCode::Null,
            formatted_prompt: Text::raw(""),
            editor,
            block,
        }
    }
}

impl Prompt<'_> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.formatted_prompt = Text::raw("");
        self.editor.select_all();
        self.editor.cut();
    }

    pub fn height(&self, frame_size: &Rect) -> u16 {
        let prompt_block_max_height = (0.4 * frame_size.height as f32) as u16;

        let height: u16 = 1 + self
            .editor
            .lines()
            .iter()
            .map(|line| 1 + line.width() as u16 / frame_size.width)
            .sum::<u16>();

        std::cmp::min(height, prompt_block_max_height)
    }

    pub fn update(&mut self, focused_block: &FocusedBlock) {
        self.block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default())
            .border_type(match focused_block {
                FocusedBlock::Prompt => BorderType::Thick,
                _ => BorderType::Rounded,
            })
            .border_style(match focused_block {
                FocusedBlock::Prompt => match self.mode {
                    Mode::Insert => Style::default().fg(Color::Green),
                    Mode::Normal => Style::default(),
                    Mode::Visual => Style::default().fg(Color::Yellow),
                },
                _ => Style::default(),
            });
    }

    pub fn key_binding(&mut self, key_event: KeyEvent, clipboard: Option<&mut Clipboard>) {
        match self.mode {
            Mode::Insert => match key_event.code {
                KeyCode::Enter => {
                    self.editor.insert_newline();
                }

                KeyCode::Char(c) => {
                    self.editor.insert_char(c);
                }

                KeyCode::Backspace => {
                    self.editor.delete_char();
                }

                KeyCode::Esc => {
                    self.mode = Mode::Normal;
                    self.update(&FocusedBlock::Prompt);
                }
                _ => {}
            },
            Mode::Normal | Mode::Visual => match key_event.code {
                KeyCode::Char('i') => {
                    self.mode = Mode::Insert;
                    self.update(&FocusedBlock::Prompt);
                }

                KeyCode::Esc => {
                    self.mode = Mode::Normal;
                    self.update(&FocusedBlock::Prompt);
                    self.editor.cancel_selection();
                }

                KeyCode::Char('v') => {
                    self.mode = Mode::Visual;
                    self.update(&FocusedBlock::Prompt);
                    self.update(&FocusedBlock::Prompt);
                    self.editor.start_selection();
                }

                KeyCode::Char('h') | KeyCode::Left if key_event.modifiers == KeyModifiers::NONE => {
                    self.editor.move_cursor(CursorMove::Back);
                }

                KeyCode::Char('j') | KeyCode::Down if key_event.modifiers == KeyModifiers::NONE => {
                    self.editor.move_cursor(CursorMove::Down);
                }

                KeyCode::Char('k') | KeyCode::Up if key_event.modifiers == KeyModifiers::NONE => {
                    self.editor.move_cursor(CursorMove::Up);
                }

                KeyCode::Char('l') | KeyCode::Right
                    if key_event.modifiers == KeyModifiers::NONE =>
                {
                    self.editor.move_cursor(CursorMove::Forward);
                }

                KeyCode::Char('w') => {
                    if self.previous_key == KeyCode::Char('d') {
                        self.editor.delete_next_word();
                    }
                    self.editor.move_cursor(CursorMove::WordForward);
                }

                KeyCode::Char('b') => {
                    if self.previous_key == KeyCode::Char('d') {
                        self.editor.delete_word();
                    }
                    self.editor.move_cursor(CursorMove::WordBack);
                }

                KeyCode::Char('$') => {
                    if self.previous_key == KeyCode::Char('d') {
                        self.editor.delete_line_by_end();
                    }
                    self.editor.move_cursor(CursorMove::End);
                }

                KeyCode::Char('0') => {
                    if self.previous_key == KeyCode::Char('d') {
                        self.editor.delete_line_by_head();
                    }
                    self.editor.move_cursor(CursorMove::Head);
                }

                KeyCode::Char('^') => self.editor.move_cursor(CursorMove::Head),

                KeyCode::Char('D') => {
                    self.editor.move_cursor(CursorMove::Head);
                    self.editor.delete_line_by_end();
                    self.editor.delete_line_by_head();
                }

                KeyCode::Char('d') => {
                    if self.previous_key == KeyCode::Char('d') {
                        self.editor.move_cursor(CursorMove::Head);
                        self.editor.delete_line_by_end();
                        self.editor.delete_line_by_head();
                    }
                }

                KeyCode::Char('C') => {
                    self.editor.delete_line_by_end();
                    self.mode = Mode::Insert;
                    self.update(&FocusedBlock::Prompt);
                }

                KeyCode::Char('u') => {
                    self.editor.undo();
                }

                KeyCode::Char('x') => {
                    self.editor.delete_next_char();
                }

                KeyCode::Char('a') => {
                    self.editor.move_cursor(CursorMove::Forward);
                    self.mode = Mode::Insert;
                    self.update(&FocusedBlock::Prompt);
                }

                KeyCode::Char('A') => {
                    self.editor.move_cursor(CursorMove::End);
                    self.mode = Mode::Insert;
                    self.update(&FocusedBlock::Prompt);
                }

                KeyCode::Char('o') => {
                    self.editor.move_cursor(CursorMove::End);
                    self.editor.insert_newline();
                    self.mode = Mode::Insert;
                    self.update(&FocusedBlock::Prompt);
                }

                KeyCode::Char('O') => {
                    self.editor.move_cursor(CursorMove::Head);
                    self.editor.insert_newline();
                    self.editor.move_cursor(CursorMove::Up);
                    self.mode = Mode::Insert;
                    self.update(&FocusedBlock::Prompt);
                }

                KeyCode::Char('I') => {
                    self.editor.move_cursor(CursorMove::Head);
                    self.mode = Mode::Insert;
                    self.update(&FocusedBlock::Prompt);
                }

                KeyCode::Char('G') => self.editor.move_cursor(CursorMove::Bottom),

                KeyCode::Char('g') => {
                    if self.previous_key == KeyCode::Char('g') {
                        self.editor.move_cursor(CursorMove::Jump(0, 0))
                    }
                }

                KeyCode::Char('y') => {
                    self.editor.copy();
                    if let Some(clipboard) = clipboard {
                        let text = self.editor.yank_text();
                        let _ = clipboard.set_text(text);
                    }
                }

                KeyCode::Char('p') => {
                    if !self.editor.paste() {
                        if let Some(clipboard) = clipboard {
                            if let Ok(text) = clipboard.get_text() {
                                self.editor.insert_str(text);
                            }
                        }
                    }
                }

                _ => {}
            },
        }

        self.previous_key = key_event.code;
    }

    pub fn render(&mut self, frame: &mut Frame, block: Rect) {
        self.editor.set_block(self.block.clone());
        frame.render_widget(self.editor.widget(), block);
    }
}

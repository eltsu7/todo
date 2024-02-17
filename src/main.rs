use crossterm::{
    event::{self, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use home::{self, home_dir};
use ratatui::{
    layout::{self, Constraint, Layout},
    prelude::{CrosstermBackend, Terminal},
    style::{Color, Style},
    widgets::{Borders, Paragraph},
};
use std::io::{stdout, Result};
use std::{
    fs::{self, create_dir_all},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};
use serde_json::{self};
use Direction::*;

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Serialize, Deserialize)]
struct TaskList {
    name: String,
    tasks: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct Todos {
    lists: Vec<TaskList>,
    current_list: usize,
    current_task: usize,
    editing: bool,
}

impl Todos {
    fn new() -> Todos {
        Todos {
            lists: Vec::new(),
            current_list: 0,
            current_task: 0,
            editing: false,
        }
    }

    fn get_file_path(&self) -> PathBuf {
        let mut path = match home_dir() {
            Some(path) => path,
            None => panic!("Home dir not found"),
        };

        path.push(".todo");
        if !path.exists() {
            create_dir_all(&path).unwrap();
        }
        path.push("tasks.json");
        if !path.exists() {
            fs::write(&path, serde_json::to_string_pretty(&self.lists).unwrap()).unwrap();
        }

        path
    }

    fn save_to_file(&self) -> std::io::Result<()> {
        fs::write(
            self.get_file_path(),
            serde_json::to_string_pretty(&self.lists)?,
        )?;
        Ok(())
    }

    fn load_file(&mut self) -> std::io::Result<()> {
        self.lists = serde_json::from_str(&fs::read_to_string(self.get_file_path())?)?;
        Ok(())
    }

    fn add_task(&mut self) {
        let task_index = self.current_task.clone();
        let list = self.get_current_list();

        if list.tasks.len() == 0 {
            list.tasks.push(String::new());
        } else {
            list.tasks.insert(task_index, String::new())
        }
    }

    fn delete_task(&mut self) -> std::io::Result<()> {
        let task_index = self.current_task.clone();
        self.get_current_list().tasks.remove(task_index);
        let list = self.get_current_list();
        if list.tasks.len() > 0 && task_index + 1 > list.tasks.len() {
            self.current_task = list.tasks.len() - 1;
        }
        self.save_to_file()?;
        Ok(())
    }

    fn get_list(&mut self, list_index: usize) -> &mut TaskList {
        self.lists.get_mut(list_index).expect("Lists out of range!")
    }

    fn get_current_list(&mut self) -> &mut TaskList {
        self.get_list(self.current_list)
    }

    fn _check_cursor_position(&mut self) {
        if self.lists[self.current_list].tasks.len() == 0 {
            self.current_task = 0;
        } else if self.current_task + 1 > self.lists[self.current_list].tasks.len() {
            self.current_task = self.lists[self.current_list].tasks.len() - 1;
        }
    }

    fn move_cursor(&mut self, direction: Direction) {
        match direction {
            Left => {
                if self.current_list != 0 {
                    self.current_list -= 1;
                }
            }
            Right => {
                if self.current_list + 1 < self.lists.len() {
                    self.current_list += 1;
                }
            }
            Up => {
                if self.current_task != 0 {
                    self.current_task -= 1;
                }
            }
            Down => {
                if self.current_task + 1 < self.get_current_list().tasks.len() {
                    self.current_task += 1;
                }
            }
        }
        self._check_cursor_position();
    }

    fn move_task(&mut self, direction: Direction) {
        let task_index = self.current_task.clone();
        match direction {
            Up => {
                if self.current_task != 0 {
                    self.get_current_list()
                        .tasks
                        .swap(task_index, task_index - 1);
                }
                self.current_task -= 1;
            }
            Down => {
                if self.get_current_list().tasks.len() > task_index + 1 {
                    self.get_current_list()
                        .tasks
                        .swap(task_index, task_index + 1);
                }
                self.current_task += 1;
            }
            Left => {
                if self.current_list != 0 {
                    let item = self.get_current_list().tasks.remove(task_index);
                    self.get_list(self.current_list - 1).tasks.push(item);
                }
            }
            Right => {
                if self.lists.len() > self.current_list + 1 {
                    let item = self.get_current_list().tasks.remove(task_index);
                    self.get_list(self.current_list + 1).tasks.push(item);
                }
            }
        }
        self._check_cursor_position();
    }
}

fn input_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    todos: &mut Todos,
) -> Result<()> {
    let title_default = Style {
        fg: Some(Color::White),
        bg: Some(Color::Black),
        ..Default::default()
    };

    let title_selected = Style {
        fg: Some(Color::Cyan),
        bg: Some(Color::Black),
        ..Default::default()
    };

    let task_default = Style {
        fg: Some(Color::White),
        bg: Some(Color::Black),
        ..Default::default()
    };

    let task_selected = Style {
        fg: Some(Color::White),
        bg: Some(Color::DarkGray),
        ..Default::default()
    };

    let task_editing = Style {
        fg: Some(Color::White),
        bg: Some(Color::Green),
        ..Default::default()
    };

    loop {
        // Drawing
        terminal.draw(|frame| {
            let outer_layout = Layout::default()
                .direction(layout::Direction::Vertical)
                .constraints(vec![Constraint::Length(3), Constraint::Fill(1)])
                .split(frame.size());

            // Render titles
            let mut title_constraints = Vec::new();
            for _ in 0..todos.lists.len() {
                title_constraints.push(Constraint::Fill(1));
            }

            let title_layout = Layout::default()
                .direction(layout::Direction::Horizontal)
                .constraints(title_constraints)
                .split(outer_layout[0]);

            for (i, list) in todos.lists.iter_mut().enumerate() {
                let style = if i == todos.current_list {
                    title_selected
                } else {
                    title_default
                };

                let title_text = format!("{} ({})", list.name, list.tasks.len());
                frame.render_widget(
                    Paragraph::new(title_text)
                        .style(style)
                        .centered()
                        .block(ratatui::widgets::Block::new().borders(Borders::ALL)),
                    title_layout[i],
                )
            }

            // Render tasks
            let mut task_constraints = Vec::new();
            for _ in 0..todos.get_current_list().tasks.len() {
                task_constraints.push(Constraint::Length(1));
            }

            let task_layout = Layout::default()
                .direction(layout::Direction::Vertical)
                .constraints(task_constraints)
                .split(outer_layout[1]);

            let task_index = todos.current_task.clone();
            let editing = todos.editing.clone();
            for (i, task_text) in todos.get_list(todos.current_list).tasks.iter().enumerate() {
                let style = if task_index == i {
                    if editing {
                        task_editing
                    } else {
                        task_selected
                    }
                } else {
                    task_default
                };

                frame.render_widget(
                    Paragraph::new(task_text.clone()).style(style),
                    task_layout[i],
                );
            }
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // Move task
                if key.modifiers.contains(KeyModifiers::ALT) {
                    if todos.get_current_list().tasks.len() == 0 || todos.editing {
                        continue;
                    }
                    match key.code {
                        KeyCode::Right => {
                            todos.move_task(Right);
                        }
                        KeyCode::Left => {
                            todos.move_task(Left);
                        }
                        KeyCode::Up => {
                            todos.move_task(Up);
                        }
                        KeyCode::Down => {
                            todos.move_task(Down);
                        }
                        KeyCode::Char('d') => {
                            todos.delete_task()?;
                        }
                        _ => (),
                    }
                } else {
                    // Move cursor
                    if !todos.editing {
                        match key.code {
                            KeyCode::Up => {
                                todos.move_cursor(Up);
                            }
                            KeyCode::Down => {
                                todos.move_cursor(Down);
                            }
                            KeyCode::Left => {
                                todos.move_cursor(Left);
                            }
                            KeyCode::Right => {
                                todos.move_cursor(Right);
                            }
                            _ => (),
                        }
                    }

                    match key.code {
                        KeyCode::Char(key_char) => {
                            if todos.editing {
                                let task_index = todos.current_task.clone();
                                todos.get_current_list().tasks[task_index].push(key_char);
                            } else {
                                match key_char {
                                    'q' => {
                                        todos.save_to_file()?;
                                        break;
                                    }
                                    'i' => {
                                        todos.add_task();
                                        todos.editing = true;
                                    }
                                    _ => (),
                                }
                            }
                        }
                        KeyCode::Backspace => {
                            if todos.editing {
                                let task_index = todos.current_task.clone();
                                todos.get_current_list().tasks[task_index].pop();
                            }
                        }
                        KeyCode::Enter => {
                            let task_index = todos.current_task.clone();
                            if todos.get_current_list().tasks.len() == 0 {
                                continue;
                            }
                            todos.editing = !todos.editing;
                            if todos.get_current_list().tasks[task_index] == "" {
                                todos.get_current_list().tasks.remove(task_index);
                            }
                            if !todos.editing {
                                todos.save_to_file()?;
                            }
                        }
                        _ => (),
                    }
                }
            }
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match &args[1][..] {
            "-h" | "--help" => {
                println!("Arrows: move cursor, alt + arrows: move tasks, q: quit, i: insert task, alt + d: delete task.");
                return Ok(());
            }
            _ => {
                println!("-h / --help for help.");
                return Ok(());
            }
        }
    }

    let mut t: Todos = Todos::new();

    t.lists.push(TaskList {
        name: "Backlog".to_string(),
        tasks: Vec::new(),
    });
    t.lists.push(TaskList {
        name: "In Progress".to_string(),
        tasks: Vec::new(),
    });
    t.lists.push(TaskList {
        name: "Done".to_string(),
        tasks: Vec::new(),
    });

    t.current_list = 1;

    t.load_file().unwrap();

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal: Terminal<CrosstermBackend<std::io::Stdout>> =
        Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    input_loop(&mut terminal, &mut t)?;

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

use crossterm::{
    event::{self, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use home::{self, home_dir};
use ratatui::{
    layout::Rect,
    prelude::{CrosstermBackend, Terminal},
    style::{Color, Style},
    widgets::Paragraph,
};
use std::{fs::create_dir_all, io::Write, path::PathBuf};
use std::fs::File;
use std::io::{stdout, Result};
use std::io::{BufReader, BufWriter};

use serde::{Deserialize, Serialize};
use serde_json::{self, json};
use TodoType::{Backlog, Done, InProgress};

enum TodoType {
    Backlog,
    InProgress,
    Done,
}
#[derive(Serialize, Deserialize)]
struct Tasks {
    backlog: Vec<String>,
    in_progress: Vec<String>,
    done: Vec<String>,
}

struct Todos {
    tasks: Tasks,
}

impl Todos {
    fn new() -> Todos {
        Todos {
            tasks: Tasks {
                backlog: Vec::new(),
                in_progress: Vec::new(),
                done: Vec::new(),
            },
        }
    }

    fn get_file_path(&self) -> PathBuf{
        let mut path = match home_dir() {
            Some(path) => path,
            None => panic!("Home dir not found")
        };

        path.push(".todo");
        if !path.exists() {
            create_dir_all(&path).unwrap();
        }
        path.push("tasks.json");
        if !path.exists() {
            let writer = BufWriter::new(File::create(&path).unwrap());    
            serde_json::to_writer_pretty(writer, &self.tasks).unwrap();
        }

        path
    }

    fn save_to_file(&self) -> std::io::Result<()> {

        let writer = BufWriter::new(File::create(self.get_file_path())?);
        serde_json::to_writer_pretty(writer, &self.tasks)?;
        Ok(())
    }

    fn load_file(&mut self) -> std::io::Result<()> {
        let reader = BufReader::new(File::open(self.get_file_path())?);
        let tasks_json: Tasks = serde_json::from_reader(reader)?;
        self.tasks = tasks_json;
        Ok(())
    }

    fn add_task(&mut self, todo_type: &TodoType, index: usize) {
        self.get_list(&todo_type).insert(index, String::new());
    }

    fn delete_task(&mut self, todo_type: &TodoType, index: usize) -> std::io::Result<()> {
        self.get_list(todo_type).remove(index);
        self.save_to_file()?;
        Ok(())
    }

    fn _print_todos(&self) {
        fn print_list(list: &Vec<String>) {
            for (i, text) in list.iter().enumerate() {
                println!("  {}: {}", i, text)
            }
        }

        println!("Backlog:");
        print_list(&self.tasks.backlog);

        println!("In Progress:");
        print_list(&self.tasks.in_progress);

        println!("Done:");
        print_list(&self.tasks.done);

        println!();
    }

    fn get_list(&mut self, todo_type: &TodoType) -> &mut Vec<String> {
        match todo_type {
            Backlog => &mut self.tasks.backlog,
            InProgress => &mut self.tasks.in_progress,
            Done => &mut self.tasks.done,
        }
    }

    fn swap(&mut self, todo_type: &TodoType, i: usize, j: usize) {
        self.get_list(todo_type).swap(i as usize, j as usize);
    }

    fn move_task(&mut self, todo_type: &TodoType, index: usize, right: bool) {
        let destination = if right {
            match todo_type {
                Backlog => InProgress,
                InProgress => Done,
                Done => return,
            }
        } else {
            match todo_type {
                Backlog => return,
                InProgress => Backlog,
                Done => InProgress,
            }
        };
        let item = self.get_list(todo_type).remove(index);
        self.get_list(&destination).push(item);
    }
}

fn input_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    todos: &mut Todos,
) -> Result<()> {
    let mut chosen_list: TodoType = TodoType::InProgress;
    let mut current_index: usize = 0;
    let mut editing: bool = false;

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
        let current_list: &mut Vec<String> = todos.get_list(&chosen_list);

        if current_list.len() > 0 && current_list.len() < current_index + 1 {
            current_index = current_list.len() - 1;
        }

        terminal.draw(|frame| {
            let mut area = frame.size();
            area.height = area.height / 2;

            let mut paragraphs: Vec<Paragraph<'_>> = Vec::<Paragraph>::new();

            let third_length = frame.size().width / 3;

            // Render titles
            frame.render_widget(
                Paragraph::new("Backlog")
                    .style(match chosen_list {
                        Backlog => title_selected,
                        _ => title_default,
                    })
                    .centered(),
                Rect::new(0, 0, third_length, 1),
            );
            frame.render_widget(
                Paragraph::new("In Progress")
                    .style(match chosen_list {
                        InProgress => title_selected,
                        _ => title_default,
                    })
                    .centered(),
                Rect::new(third_length, 0, third_length, 1),
            );
            frame.render_widget(
                Paragraph::new("Done")
                    .style(match chosen_list {
                        Done => title_selected,
                        _ => title_default,
                    })
                    .centered(),
                Rect::new(third_length * 2, 0, third_length, 1),
            );

            // Render tasks
            for (i, item) in current_list.iter().enumerate() {
                if i == current_index {
                    paragraphs.push(Paragraph::new(item.clone()).style(if editing {
                        task_editing
                    } else {
                        task_selected
                    }));
                } else {
                    paragraphs.push(Paragraph::new(item.clone()).style(task_default));
                }
            }

            for (i, item) in paragraphs.iter().enumerate() {
                frame.render_widget(item, Rect::new(0, 1 + i as u16, frame.size().width, 1));
            }
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                // Move task
                if key.modifiers.contains(KeyModifiers::ALT) {
                    if current_list.len() == 0 || editing {
                        continue;
                    }
                    match key.code {
                        KeyCode::Right | KeyCode::Left => {
                            todos.move_task(&chosen_list, current_index, key.code == KeyCode::Right)
                        }
                        KeyCode::Up => {
                            if current_index != 0 {
                                todos.swap(&chosen_list, current_index, current_index - 1);
                                current_index -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if current_index != current_list.len() - 1 {
                                todos.swap(&chosen_list, current_index, current_index + 1);
                                current_index += 1;
                            }
                        }
                        KeyCode::Char('d') => {
                            todos.delete_task(&chosen_list, current_index)?;
                        }
                        _ => (),
                    }
                } else {
                    // Move cursor
                    if !editing {
                        match key.code {
                            KeyCode::Up => {
                                if current_index > 0 {
                                    current_index -= 1;
                                }
                            }
                            KeyCode::Down => {
                                if current_index + 1 < current_list.len() {
                                    current_index += 1;
                                }
                            }
                            KeyCode::Left => match chosen_list {
                                InProgress => chosen_list = Backlog,
                                Done => chosen_list = InProgress,
                                _ => (),
                            },
                            KeyCode::Right => match chosen_list {
                                InProgress => chosen_list = Done,
                                Backlog => chosen_list = InProgress,
                                _ => (),
                            },
                            _ => (),
                        }
                    }

                    match key.code {
                        KeyCode::Char(key_char) => {
                            if editing {
                                current_list[current_index].push(key_char);
                            } else {
                                match key_char {
                                    'q' => {
                                        todos.save_to_file()?;
                                        break;
                                    }
                                    'i' => {
                                        let insert_index = if current_list.len() > 0 {
                                            current_index + 1
                                        } else {
                                            current_index
                                        };
                                        todos.add_task(&chosen_list, insert_index);
                                        current_index += 1;
                                        editing = true;
                                    }
                                    _ => (),
                                }
                            }
                        }
                        KeyCode::Backspace => {
                            if editing {
                                current_list[current_index].pop();
                            }
                        }
                        KeyCode::Enter => {
                            if current_list.len() == 0 {
                                continue;
                            }
                            editing = !editing;
                            if current_list[current_index] == "" {
                                current_list.remove(current_index);
                            }
                            if !editing {
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

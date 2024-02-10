use crossterm::{
    event::{self, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::Rect,
    prelude::{CrosstermBackend, Stylize, Terminal},
    style::{Color, Style},
    widgets::Paragraph,
};
use std::{
    cmp,
    io::{stdout, Result},
};
use TodoType::{Backlog, Done, InProgress};

enum TodoType {
    Backlog,
    InProgress,
    Done,
}

struct Todos {
    backlog: Vec<String>,
    in_progress: Vec<String>,
    done: Vec<String>,
}

impl Todos {
    fn new() -> Todos {
        Todos {
            backlog: Vec::new(),
            in_progress: Vec::new(),
            done: Vec::new(),
        }
    }

    fn add_todo(&mut self, new_item: &str, todo_type: TodoType) {
        match todo_type {
            TodoType::Backlog => self.backlog.push(new_item.to_string()),
            TodoType::InProgress => self.in_progress.push(new_item.to_string()),
            TodoType::Done => self.done.push(new_item.to_string()),
        }
    }

    fn print_todos(&self) {
        fn print_list(list: &Vec<String>) {
            for (i, text) in list.iter().enumerate() {
                println!("  {}: {}", i, text)
            }
        }

        println!("Backlog:");
        print_list(&self.backlog);

        println!("In Progress:");
        print_list(&self.in_progress);

        println!("Done:");
        print_list(&self.done);

        println!();
    }

    fn swap(&mut self, todo_type: &TodoType, i: i32, j: i32) {
        let list: &mut Vec<String> = match todo_type {
            TodoType::Backlog => &mut self.backlog,
            TodoType::InProgress => &mut self.in_progress,
            TodoType::Done => &mut self.done,
        };

        list.swap(i as usize, j as usize);
    }

    fn move_to(&mut self, from_type: TodoType, to_type: TodoType, index: usize) {
        let from_list: &mut Vec<String> = match from_type {
            TodoType::Backlog => &mut self.backlog,
            TodoType::InProgress => &mut self.in_progress,
            TodoType::Done => &mut self.done,
        };
        let item: String = from_list.remove(index);

        let to_list: &mut Vec<String> = match to_type {
            TodoType::Backlog => &mut self.backlog,
            TodoType::InProgress => &mut self.in_progress,
            TodoType::Done => &mut self.done,
        };

        to_list.push(item);
    }
}

fn input_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    todos: &mut Todos,
) -> Result<()> {
    let mut chosen_list: TodoType = TodoType::InProgress;
    let mut current_index: usize = 0;
    let mut editing: bool = false;

    let title_selected = Style {
        fg: Some(Color::Cyan),
        bg: Some(Color::Black),
        ..Default::default()
    };

    let title_default = Style {
        fg: Some(Color::White),
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
        let current_list: &mut Vec<String> = match chosen_list {
            Backlog => &mut todos.backlog,
            InProgress => &mut todos.in_progress,
            Done => &mut todos.done,
        };

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
                    if current_list.len() == 0 {
                        continue;
                    }
                    match key.code {
                        KeyCode::Right => match chosen_list {
                            Backlog => todos.move_to(Backlog, InProgress, current_index),
                            InProgress => todos.move_to(InProgress, Done, current_index),
                            _ => (),
                        },
                        KeyCode::Left => match chosen_list {
                            InProgress => todos.move_to(InProgress, Backlog, current_index),
                            Done => todos.move_to(Done, InProgress, current_index),
                            _ => (),
                        },
                        KeyCode::Up => {
                            if current_index != 0 {
                                todos.swap(
                                    &chosen_list,
                                    current_index as i32,
                                    current_index as i32 - 1,
                                );
                                current_index -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if current_index != current_list.len() - 1 {
                                todos.swap(
                                    &chosen_list,
                                    current_index as i32,
                                    current_index as i32 + 1,
                                );
                                current_index += 1;
                            }
                        }
                        _ => (),
                    }
                } else {
                    // Move cursor
                    match key.code {
                        KeyCode::Char(key_char) => {
                            if editing {
                                current_list[current_index].push(key_char);
                            } else {
                                match key_char {
                                    'q' => break,
                                    'i' => {
                                        let insert_index = if current_list.len() > 0 {
                                            current_index + 1
                                        } else {
                                            current_index
                                        };
                                        current_list.insert(insert_index, String::new());
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
                            editing = !editing;
                            if current_list[current_index] == "" {
                                current_list.remove(current_index);
                            }
                        }
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
            }
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let mut t: Todos = Todos::new();

    t.add_todo("eka", Backlog);
    t.add_todo("toka", Backlog);
    t.add_todo("kolmas", Backlog);
    t.add_todo("Eka taski", InProgress);
    t.add_todo("Toka taski", InProgress);
    t.add_todo("Kolmas", InProgress);
    t.add_todo("Neljäs", InProgress);
    t.add_todo("Valmis :D", Done);

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

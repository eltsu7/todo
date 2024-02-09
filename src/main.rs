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
    }

    fn swap(&mut self, todo_type: TodoType, i: i32, j: i32) {
        let list: &mut Vec<String> = match todo_type {
            TodoType::Backlog => &mut self.backlog,
            TodoType::InProgress => &mut self.in_progress,
            TodoType::Done => &mut self.done,
        };

        list.swap(i as usize, j as usize);
    }

    fn move_to(&mut self, from_type: TodoType, to_type: TodoType) {
        // TODO
    }

}

fn main() {
    let mut t = Todos::new();

    t.add_todo("eka", TodoType::Backlog);
    t.add_todo("toka", TodoType::Backlog);
    t.add_todo("kolmas", TodoType::Backlog);
    t.add_todo("heippa", TodoType::InProgress);
    t.add_todo("hola", TodoType::Done);
    t.print_todos();
    println!("");
    t.swap(TodoType::Backlog, 0, 1);
    t.print_todos();
}

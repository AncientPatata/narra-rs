use std::fs;
use std::path::Path;

pub struct NarraState {
    action_stack: Vec<String>,
}

impl NarraState {
    pub fn new() -> NarraState {
        NarraState {
            action_stack: Vec::<String>::new(),
        }
    }

    pub fn push_action(&mut self, action_id: String) {
        self.action_stack.push(action_id);
    }

    pub fn seen_action(&self, action_id: String) -> bool {
        self.action_stack
            .iter()
            .find(|x| x == &&action_id)
            .is_some()
    }

    pub fn save_action_history<P>(&self, file_path: P)
    where
        P: AsRef<Path>,
    {
        let mut file_contents = String::new();
        for action in &self.action_stack {
            file_contents += format!("{}\n", action).as_str();
        }
        fs::write(file_path, file_contents).unwrap();
    }
}

impl Clone for NarraState {
    fn clone(&self) -> NarraState {
        NarraState {
            action_stack: self.action_stack.clone(),
        }
    }
}

use serde_json as json;

pub struct NarraInstance {
    pub action_stack: Vec<json::Value>,
    pub current_tree: String,
    pub narra_tree: json::Value,
    pub end_of_file: bool,
    pub blocked: bool,
}

impl NarraInstance {
    pub fn from_json(narra_json: &json::Value) -> NarraInstance {
        let mut instance = NarraInstance {
            current_tree: "main".to_string(),
            narra_tree: narra_json.clone(),
            action_stack: Vec::<json::Value>::new(),
            end_of_file: false,
            blocked: true,
        };
        for obj in narra_json.as_array().unwrap() {
            if obj["tree"] == instance.current_tree {
                let mut actions = obj["tree_body"].as_array().unwrap().clone();
                instance.append_action_sequence(actions);
                break;
            }
        }
        instance
    }

    pub fn new() -> NarraInstance {
        NarraInstance {
            current_tree: "main".to_string(),
            narra_tree: json::json!({}),
            action_stack: Vec::<json::Value>::new(),
            end_of_file: true,
            blocked: false,
        }
    }

    pub fn append_action_sequence(&mut self, mut action_seq: Vec<json::Value>) {
        action_seq.reverse();
        self.action_stack.append(&mut action_seq);
    }

    pub fn perform_jump(&mut self, jump_to: String) {
        for obj in self.narra_tree.as_array().unwrap() {
            if obj["tree"] == jump_to {
                self.current_tree = jump_to;
                let mut actions = obj["tree_body"].as_array().unwrap().clone();
                self.append_action_sequence(actions);
                break;
            }
        }
    }

    pub fn eot(&self) -> bool {
        self.action_stack.len() == 0
    }
}

impl Clone for NarraInstance {
    fn clone(&self) -> Self {
        NarraInstance {
            action_stack: self.action_stack.clone(),
            current_tree: self.current_tree.clone(),
            narra_tree: self.narra_tree.clone(),
            end_of_file: self.end_of_file,
            blocked: self.blocked,
        }
    }
}

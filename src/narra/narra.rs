use serde_json as json;
use std::fs::{self, File};
// rhai
use rhai::{Engine, Scope, AST};
// ss
// use rand::Rng;
// use regex::Regex;
// use std::collections::HashMap;
use std::io::{self, BufRead, Read};
use std::path::Path;

type DialogueTextCallback = fn(
    character_name: Option<String>,
    dialogue_text: String,
    modifiers: json::Map<String, json::Value>,
);

type ChoiceCallback = fn(choice_texts: Vec<String>, modifiers: Vec<json::Value>) -> usize; // returns clicked choice
                                                                                           // TODO : Uniquely hash choice texts to track choices that were made ? use a #id modifier to assign a custom id to a choice which is then mapped to the hashed value.

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

pub struct NarraRuntime {
    pub current_tree: String,
    pub modifiers: json::Value,
    narra_tree: json::Value,
    action_stack: Vec<json::Value>,
    end_of_file: bool,
    // Action Handlers
    dialogue_callback: DialogueTextCallback,
    choice_callback: ChoiceCallback,
    // Save state
    narra_state: NarraState,
    pub engine: Engine, // Engine for the Rhai document.
    scope: Scope<'static>,
    rhai_ast: AST,
}

impl NarraRuntime {
    pub fn new() -> NarraRuntime {
        NarraRuntime {
            current_tree: "main".to_string(),
            modifiers: json::json!([]),
            narra_tree: json::json!({}),
            action_stack: Vec::<json::Value>::new(),
            end_of_file: true,
            // default callbacks:
            dialogue_callback: |character, text, _| {
                if character.is_none() {
                    println!("{}", text);
                } else {
                    println!("{} : {}", character.unwrap(), text);
                }
            },
            choice_callback: |choices, _| {
                println!("Input in a number for a choice :");
                let mut i = 1;
                for choice in choices.clone() {
                    println!("{}. {}", i, choice);
                    i += 1;
                }
                let mut str_in: String = String::new();
                io::stdin().read_line(&mut str_in);
                println!("Selected : {}", str_in);
                (str_in.trim_end().parse::<u32>().unwrap() - 1) as usize
            },
            narra_state: NarraState::new(),
            engine: Engine::new(),
            scope: Scope::new(),
            rhai_ast: AST::default(),
        }
    }

    pub fn set_dialogue_callback(&mut self, callback: DialogueTextCallback) {
        self.dialogue_callback = callback;
    }

    pub fn set_choice_callback(&mut self, callback: ChoiceCallback) {
        self.choice_callback = callback;
    }

    pub fn set_narra_file(&mut self, narra_json: json::Value) {
        // println!(
        //     "NARRA JSON : ::::::: : \n {} \n : ::::::::: : \n",
        //     narra_json
        // );
        self.narra_tree = narra_json.clone();
        self.end_of_file = false;
        self.current_tree = "main".to_string();
        for obj in narra_json.as_array().unwrap() {
            if obj["tree"] == self.current_tree {
                let mut actions = obj["tree_body"].as_array().unwrap().clone();
                self.append_action_sequence(actions);
                break;
            }
        }
    }

    pub fn set_narra_state(&mut self, new_state: NarraState) {
        self.narra_state = new_state;
    }

    pub fn get_narra_state(&self) -> &NarraState {
        &self.narra_state
    }

    pub fn read_file(&mut self, narra_file: String) {
        let split_file = narra_file.split("<<--SPLIT-->>").collect::<Vec<&str>>();
        // println!("SPLIT 0 \n {} \n \n", split_file[0]);
        // println!("SPLIT 1 \n {} \n \n", split_file[1]);

        // Parse NARRA file

        self.set_narra_file(json::to_value(&split_file[1]).unwrap());
        self.rhai_ast = self
            .engine
            .compile_with_scope(&mut self.scope, &split_file[0])
            .unwrap();
    }

    pub fn next(&self) -> bool {
        self.action_stack.len() > 0
    }

    fn append_action_sequence(&mut self, mut action_seq: Vec<json::Value>) {
        action_seq.reverse();
        self.action_stack.append(&mut action_seq);
    }

    fn handle_choices(&mut self, choice_action: json::Value) {
        let choices: Vec<String> = choice_action["action"]["choice_body"]
            .clone()
            .as_array()
            .unwrap()
            .iter()
            .map(|choice| self.handle_value(&choice["choice_text"]))
            .collect();
        let selected_choice = (self.choice_callback)(
            choices,
            choice_action["modifiers"].as_array().unwrap().clone(),
        );
        let resultant_actions = choice_action["action"]["choice_body"].as_array().unwrap()
            [selected_choice]["action_sequence"]
            .as_array()
            .unwrap();
        self.append_action_sequence(resultant_actions.clone());
    }

    fn handle_eval(&mut self, eval_action: json::Value) {
        let func_name = eval_action["action"]["eval_value"]["func_id"]
            .as_str()
            .unwrap();
        self.engine
            .call_fn::<()>(&mut self.scope, &self.rhai_ast, func_name, ());
    }

    fn handle_value(&mut self, value: &json::Value) -> String
// where
    //     //        P: From<json::Value> + 'static,
    //     P: Clone,
    //     P: Default,
    //     P: std::ops::Add<Output = P>,
    {
        match value["value_type"].as_str().unwrap() {
            "static" => value["value"].as_str().unwrap().to_string(),
            "dynamic" => {
                let str1 = self.handle_value(&value["str1"]);
                let str2 = self.handle_value(&value["str2"]);
                let val: String = str1 + str2.as_str();
                val
            }
            "eval" => self
                .engine
                .call_fn::<String>(
                    &mut self.scope,
                    &self.rhai_ast,
                    value["func_id"].as_str().unwrap(),
                    (),
                )
                .unwrap(),
            _ => "".to_string(),
        }
    }

    fn perform_jump(&mut self, jump_to: String) {
        for obj in self.narra_tree.as_array().unwrap() {
            if obj["tree"] == jump_to {
                self.current_tree = jump_to;
                let mut actions = obj["tree_body"].as_array().unwrap().clone();
                self.append_action_sequence(actions);
                break;
            }
        }
    }

    fn handle_modifiers(&self, modifiers_json: &json::Value) -> json::Map<String, json::Value> {
        let mut modifiers = json::Map::new();
        for modifier in modifiers_json.as_array().unwrap() {
            modifiers.insert(
                modifier["modifier"].as_str().unwrap().to_string(),
                match modifier["value"]["type"].as_str().unwrap() {
                    "int" => json::Value::from(modifier["value"]["value"].as_i64().unwrap()),
                    // Handle different values ...
                    _ => json::Value::Null,
                },
            );
        }
        modifiers
    }

    pub fn handle_action(&mut self) {
        let action = self.action_stack.pop().unwrap();
        self.narra_state
            .push_action(action["id"].as_str().unwrap().to_string());
        let modifiers = self.handle_modifiers(&action["modifiers"]);
        //println!("ACTION ::::::::::::::::::: \n {} \n", action);
        match action["action"]["action_type"].as_str().unwrap() {
            "dialogue_action" => (self.dialogue_callback)(
                action["action"]["character_name"]
                    .as_str()
                    .map(|x| x.to_string()),
                self.handle_value(&action["action"]["dialogue"]),
                modifiers,
            ),
            "choice_action" => self.handle_choices(action),
            "eval_action" => self.handle_eval(action),
            "jump_action" => {
                self.perform_jump(action["action"]["jump_to"].as_str().unwrap().to_string());
            }
            _ => println!("Unexpected action : {}", action),
        }
    }

    pub fn get_current_action(&mut self) -> json::Value {
        self.action_stack.last().unwrap().clone()
    }
}

// fn preprocess_narra<P>(filename: P) -> io::Result<(String, HashMap<String, String>, String)>
// where
//     P: AsRef<Path>,
// {
//     let mut file = fs::File::open(&filename)?;
//     let mut content = String::new();
//     file.read_to_string(&mut content)?;

//     // Get declaration :
//     let mut generated_script = "".to_string();

//     let declare_re = Regex::new(r"(?s)@declare\s*<<(.*?)>>").unwrap();
//     for cap in declare_re.captures_iter(&content.clone()) {
//         let code = &cap[1];

//         generated_script += code;
//         // println!("-----------");
//         // println!("{}", generated_script);
//         // println!("-----------");

//         // Replace SOME_CODE with the random name
//         content = content.replace(&cap[0], "");
//     }

//     let mut code_map = HashMap::new();
//     let re = Regex::new(r"(?s)<<(.*?)>>").unwrap();
//     for cap in re.captures_iter(&content.clone()) {
//         let code = &cap[1];

//         // Generate random name
//         let random_name: String = rand::thread_rng()
//             .sample_iter(&rand::distributions::Alphanumeric)
//             .take(10)
//             .map(char::from)
//             .filter(|c| c.is_alphabetic())
//             .collect();

//         code_map.insert(random_name.clone(), code.to_string());

//         // Replace SOME_CODE with the random name
//         content = content.replace(&cap[0], &format!("<<{}>>", random_name));
//     }

//     //println!("{}", content);

//     Ok((content, code_map, generated_script))
// }

// pub fn compile_narra_file<P>(filename: P) -> io::Result<(String)>
// where
//     P: AsRef<Path>,
// {
//     let (mut pure_narra, functions, mut generated_script) = preprocess_narra(filename)?;

//     for (func_name, func_code) in functions {
//         generated_script += &format!("fn {}() \n{{\n{}\n}}\n", func_name, func_code);
//     }
//     // Consider compiling the "generated script" and returning that instead.
//     Ok(format!("{}\n<<--SPLIT-->>\n{}", generated_script, pure_narra).to_string())
// }

use crate::lua_serialize::*;
use crate::narra_extern;
use crate::narra_front::NarraEventHandler;
use crate::narra_instance;
use crate::narra_instance::SharedNarraInstanceWrapper;
use crate::narra_state;

use narra_extern::*;
use narra_instance::NarraInstance;
use narra_state::NarraState;
use rlua;
use rlua::LightUserData;
use rlua::{Function, Lua, MetaMethod, Result, UserData, UserDataMethods, Variadic};
use serde_json as json;

use std::os::raw::c_void;
use std::sync::Mutex;
use std::{cell::RefCell, rc::Rc};

// TODO : Uniquely hash choice texts to track choices that were made ? use a #id modifier to assign a custom id to a choice which is then mapped to the hashed value.

pub struct NarraRuntime<T>
where
    T: NarraEventHandler,
{
    pub instance: NarraInstance,
    // Action Handlers
    // dialogue_callback: DialogueTextCallback,
    // choice_callback: ChoiceCallback,
    // Tracking
    currently_handled_action: json::Value,
    //user_event: bool,
    // Code vars
    event_handler: Rc<RefCell<T>>,
    pub lua: Lua, // Engine for the Rhai document.
}

impl<T> NarraRuntime<T>
where
    T: NarraEventHandler,
{
    pub fn new(narra_event_handler: Rc<RefCell<T>>) -> NarraRuntime<T> {
        NarraRuntime {
            instance: NarraInstance::new(),

            event_handler: narra_event_handler,
            lua: Lua::new(),

            currently_handled_action: json::Value::Null,
        }
    }

    pub fn init(&mut self) {
        // let wrapper = SharedNarraInstanceWrapper {
        //     instance: Mutex::new(Rc::new(RefCell::new(self.instance))),
        // };
        let instance_ptr = NarraInstanceHandle(&mut self.instance as *mut NarraInstance);
        self.lua.context(|ctx| {
            let user_data = ctx.create_userdata(instance_ptr).unwrap();
            ctx.globals().set("script", user_data);
        });
    }

    pub fn add_plugin(&mut self, callback: fn(&mut Lua)) {
        callback(&mut self.lua);
    }

    pub fn set_narra_state(&mut self, new_state: NarraState) {
        self.instance.state = new_state;
    }

    pub fn get_narra_state(&self) -> &NarraState {
        &self.instance.state
    }

    pub fn read_file(&mut self, narra_file: String) {
        let split_file = narra_file.split("<<--SPLIT-->>").collect::<Vec<&str>>();
        let json_tree: json::Value = json::from_str(&split_file[1]).unwrap();
        self.instance = NarraInstance::from_json(&json_tree);
        self.lua.context(|ctx| {
            let chunk = ctx.load(&split_file[0]);
            chunk.exec();
        });
    }

    fn handle_choices(&mut self, choice_action: json::Value) {
        let modifiers = self.handle_modifiers(&choice_action["modifiers"]);
        let choices: Vec<String> = choice_action["action"]["choice_body"]
            .clone()
            .as_array()
            .unwrap()
            .iter()
            .map(|choice| {
                self.handle_value(&choice["choice_text"])
                    .as_str()
                    .unwrap()
                    .to_string()
            })
            .collect();

        self.event_handler
            .borrow_mut()
            .handle_choice(choices, modifiers);
    }

    pub fn perform_choice(&mut self, selected_choice: usize) {
        let resultant_actions = self.currently_handled_action["action"]["choice_body"]
            .as_array()
            .unwrap()[selected_choice]["action_sequence"]
            .as_array()
            .unwrap();
        self.instance
            .append_action_sequence(resultant_actions.clone());
    }

    fn handle_match(&mut self, match_action: json::Value) {
        //println!("{match_action}");
        let match_val = self.handle_value(&match_action["action"]["match_value"]);
        for m_option in match_action["action"]["match_body"].as_array().unwrap() {
            let opt_ev = self.handle_value(&m_option["match_value"]);
            println!("match option : {opt_ev} \n \n match value : {match_val}");
            if match_val == opt_ev {
                let actions = m_option["action_sequence"].as_array().unwrap();
                self.instance.append_action_sequence(actions.clone());
                break;
            }
        }
    }

    fn handle_eval(&mut self, eval_action: json::Value) {
        //println!("EVAL : \n {eval_action} \n");
        let code = eval_action["action"]["eval_value"]["func_id"]
            .as_str()
            .unwrap();
        println!("EVAL ACTION : {code}");
        self.lua.context(|ctx| {
            let chunk = ctx.load(code);
            chunk.exec();
        })
    }

    fn handle_value(&mut self, value: &json::Value) -> json::Value
// where
    //     //        P: From<json::Value> + 'static,
    //     P: Clone,
    //     P: Default,
    //     P: std::ops::Add<Output = P>,
    {
        //println!("VALUE {value}");
        match value["value_type"].as_str().unwrap() {
            "static" => value["value"].clone(),
            "dynamic" => {
                let str1 = self.handle_value(&value["str1"]);
                let str2 = self.handle_value(&value["str2"]);
                let val = str1.as_str().unwrap().to_string() + str2.as_str().unwrap();
                json::Value::String(val)
            }
            "eval" => {
                let val = self
                    .lua
                    .context::<_, rlua::Result<json::Value>>(|ctx| {
                        let chunk = ctx.load(value["func_id"].as_str().unwrap());
                        println!("CODE : {}", value["func_id"].as_str().unwrap());
                        let v = chunk.eval::<JsonWrapperValue>().unwrap();
                        Ok(v.into())
                    })
                    .unwrap();
                val
            }
            _ => json::Value::Null,
        }
    }

    fn handle_modifiers(&mut self, modifiers_json: &json::Value) -> json::Map<String, json::Value> {
        let mut modifiers = json::Map::new();
        for modifier in modifiers_json.as_array().unwrap() {
            modifiers.insert(
                modifier["modifier"].as_str().unwrap().to_string(),
                self.handle_value(&modifier["value"]),
            );
        }
        modifiers
    }

    pub fn handle_dialogue(&mut self, action: json::Value) {
        let modifiers = self.handle_modifiers(&action["modifiers"]);
        let dlg_text = self
            .handle_value(&action["action"]["dialogue"])
            .as_str()
            .unwrap()
            .to_string();
        self.event_handler.borrow_mut().handle_dialogue(
            action["action"]["character_name"]
                .as_str()
                .map(|x| x.to_string()),
            dlg_text,
            modifiers,
        )
    }

    pub fn handle_action(&mut self) {
        // println!("BLOCKED ?{}", self.instance.blocked);
        if !self.instance.blocked {
            let action = self.instance.action_stack.pop().unwrap();
            // println!("ACTION : {}", action);
            self.currently_handled_action = action.clone();
            self.instance
                .state
                .push_action(action["id"].as_str().unwrap().to_string());
            let blocking = action["blocking"].is_boolean() && action["blocking"].as_bool().unwrap();
            //println!("ACTION ::::::::::::::::::: \n {} \n", action);
            match action["action"]["action_type"].as_str().unwrap() {
                "dialogue_action" => self.handle_dialogue(action),
                "choice_action" => self.handle_choices(action),
                "eval_action" => self.handle_eval(action),
                "match_action" => self.handle_match(action),
                "jump_action" => {
                    self.instance
                        .perform_jump(action["action"]["jump_to"].as_str().unwrap().to_string());
                }
                "end_action" => self.instance.end_of_file = true,

                _ => println!("Unexpected action : {}", action),
            }
            if !blocking {
                self.handle_action();
            }
        }
    }

    pub fn get_current_action(&mut self) -> json::Value {
        self.instance.action_stack.last().unwrap().clone()
    }
}

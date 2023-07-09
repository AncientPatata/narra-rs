use serde_json as json;
use std::io;
pub trait NarraEventHandler {
    fn handle_dialogue(
        &mut self,
        character_name: Option<String>,
        text: String,
        modifiers: json::Map<String, json::Value>,
    ) -> () {
        if character_name.is_none() {
            println!("{}", text);
        } else {
            println!("{} : {}", character_name.unwrap(), text);
        }
    }

    fn handle_choice(
        &mut self,
        choice_texts: Vec<String>,
        modifiers: json::Map<String, json::Value>,
    ) {
        println!("Input in a number for a choice :");
        let mut i = 1;
        for choice in choice_texts.clone() {
            println!("{}. {}", i, choice);
            i += 1;
        }
        let mut str_in: String = String::new();
        io::stdin().read_line(&mut str_in);
        println!("Selected : {}", str_in);
        //(str_in.trim_end().parse::<u32>().unwrap() - 1) as usize
    }
}

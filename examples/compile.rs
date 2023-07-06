use js_sandbox::Script;
use rand::Rng;
use regex::Regex;
use serde_json as json;
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;

fn preprocess_narra<P>(filename: P) -> io::Result<(String, HashMap<String, String>, String)>
where
    P: AsRef<Path>,
{
    let mut file = fs::File::open(&filename)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    // Get declaration :
    let mut generated_script = "".to_string();

    let declare_re = Regex::new(r"(?s)@declare\s*<<(.*?)>>").unwrap();
    for cap in declare_re.captures_iter(&content.clone()) {
        let code = &cap[1];

        generated_script += code;
        // println!("-----------");
        // println!("{}", generated_script);
        // println!("-----------");

        // Replace SOME_CODE with the random name
        content = content.replace(&cap[0], "");
    }

    let mut code_map = HashMap::new();
    let re = Regex::new(r"(?s)<<(.*?)>>").unwrap();
    for cap in re.captures_iter(&content.clone()) {
        let code = &cap[1];

        // Generate random name
        let random_name: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(10)
            .map(char::from)
            .filter(|c| c.is_alphabetic())
            .collect();

        code_map.insert(random_name.clone(), code.to_string());

        // Replace SOME_CODE with the random name
        content = content.replace(&cap[0], &format!("<<{}>>", random_name));
    }

    //println!("{}", content);

    Ok((content, code_map, generated_script))
}

pub fn compile_narra_file<P>(filename: P) -> io::Result<(String)>
where
    P: AsRef<Path>,
{
    let (mut pure_narra, functions, mut generated_script) = preprocess_narra(filename)?;

    for (func_name, func_code) in functions {
        generated_script += &format!("fn {}() \n{{\n{}\n}}\n", func_name, func_code);
    }

    let code: &'static str = include_str!("parser.js");
    let mut script = Script::from_string(code).expect("init succeeds");
    let result: json::Value = script.call("parser.parse", &pure_narra).unwrap();
    // Consider compiling the "generated script" and returning that instead.
    Ok(format!(
        "{}\n<<--SPLIT-->>\n{}",
        generated_script,
        &result.to_string()
    )
    .to_string())
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() > 1 {
        fs::write("./output_narra.nb", compile_narra_file(&args[1]).unwrap()).unwrap();
    }
}
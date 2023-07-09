use js_sandbox::Script;
use rand::Rng;
use regex::Regex;
use serde_json as json;
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;
use std::str::FromStr;

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

    Ok((content, code_map, generated_script))
}

pub fn compile_narra_file<P>(filename: P) -> io::Result<(String)>
where
    P: AsRef<Path>,
{
    let (pure_narra, functions, generated_script) = preprocess_narra(filename)?;

    let code: &'static str = include_str!("parser.js");
    let mut script = Script::from_string(code).expect("init succeeds");
    let result: json::Value = script.call("parser.parse", &pure_narra).unwrap();
    let mut narra_str = result.to_string();
    for (func_name, func_code) in functions {
        let fson = json::json!({ "func_code": func_code });
        let escaped_func = fson["func_code"].to_string();
        let fescaped_func = escaped_func.get(1..(escaped_func.len() - 1)).unwrap();
        narra_str = narra_str.replace(&func_name, &fescaped_func);
    }

    Ok(format!("{}\n<<--SPLIT-->>\n{}", generated_script, &narra_str).to_string())
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() > 2 {
        fs::write(&args[2], compile_narra_file(&args[1]).unwrap()).unwrap();
    } else {
        fs::write("output_file.nb", compile_narra_file(&args[1]).unwrap()).unwrap();
    }
}

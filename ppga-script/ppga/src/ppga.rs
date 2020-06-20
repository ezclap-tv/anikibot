extern crate ppga;

use std::env;

use ppga::ppga_to_lua;

type BoxedError = Box<dyn std::error::Error + Send + Sync>;

pub fn main() -> Result<(), BoxedError> {
    let args: Vec<String> = env::args().collect();
    let mut output = None;
    let mut disable_comments: bool = true;

    if args.len() < 2 {
        eprintln!("Error: Missing a ppga script path.");
        std::process::exit(1);
    }
    if args.len() > 2 && args[2] != "-c" {
        output = Some(args[2].clone());
    }
    if args.iter().find(|a| *a == "-c").is_some() {
        disable_comments = false;
    }

    println!("--> File `{}`", args[1]);

    let lua = std::fs::read_to_string(&args[1])
        .map_err(BoxedError::from)
        .and_then(|source| {
            ppga_to_lua(&source, !disable_comments).map_err(|e| {
                eprintln!("Failed to transpile the script:\n{}", e.report_to_string());
                BoxedError::from("")
            })
        })?;

    if let Some(path) = output {
        println!("--> Writing the transpiled code to `{}`", path);
        std::fs::write(path, lua)?;
    } else {
        println!("{}", lua);
    }

    Ok(())
}

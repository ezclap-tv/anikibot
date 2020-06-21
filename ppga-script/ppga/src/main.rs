extern crate clap;
extern crate ppga;

use clap::{App, Arg};
use ppga::{ppga_to_lua, PPGAConfig, DEFAULT_INDENT_SIZE};

type BoxedError = Box<dyn std::error::Error + Send + Sync>;

pub fn main() -> Result<(), BoxedError> {
    let args = App::new("ppga")
        .about("PPGA Script to Lua transpiler")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::with_name("input")
                .required(true)
                .takes_value(true)
                .help("Path to a .ppga script or a directory."),
        )
        .arg(
            Arg::with_name("output")
                .required(false)
                .takes_value(true)
                .help("File to write the generated code to."),
        )
        .arg(Arg::with_name("batch-output").long("batch-output").help(
            "Output the generated lua sources alongside the original .ppga files. \
            This flag works only when program input is a directory.",
        ))
        .arg(
            Arg::with_name("lint-only")
                .short("l")
                .long("lint-only")
                .help("Only check for errors and do not write/output lua code"),
        )
        .arg(
            Arg::with_name("comments")
                .short("c")
                .long("emit-comments")
                .help("Emit comments into the resulting lua code."),
        )
        .arg(Arg::with_name("no-std").short("s").long("no-std").help(
            "Do not emit the standard library helpers that are required by some PPGA features.",
        ))
        .arg(
            Arg::with_name("indent")
                .short("i")
                .long("indent-size")
                .takes_value(true)
                .help("The indentation in the generated lua code."),
        )
        .get_matches();

    let input = args.value_of("input").unwrap();
    let output = args.value_of("output");
    let batch_output = args.is_present("batch-output");
    let lint_only = args.is_present("lint-only");
    let config = PPGAConfig {
        emit_comments: args.is_present("comments"),
        include_ppga_std: !args.is_present("no-std"),
        indent_size: match args.value_of("indent") {
            Some(i) => i.parse()?,
            None => DEFAULT_INDENT_SIZE,
        },
    };

    let meta = std::fs::metadata(input)
        .map_err(|e| BoxedError::from(e))
        .and_then(|meta| {
            if output.is_some() && meta.is_dir() {
                Err(BoxedError::from(
                    "Output to a file is not supported when program input is a directory.",
                ))
            } else {
                Ok(meta)
            }
        })?;

    if meta.is_file() {
        return transpile_and_print_or_write(&input, output, lint_only, &config)
            .map(|_| ())
            .map_err(BoxedError::from);
    }

    visit_dirs(
        &std::fs::canonicalize(input)?,
        &|file: &std::fs::DirEntry| {
            if file.path().extension() != Some(std::ffi::OsStr::new("ppga")) {
                return;
            }
            let input = format!("{}", file.path().display());
            let output = if batch_output {
                std::fs::canonicalize(&input)
                    .map(|output| Some(format!("{}.lua", output.display())))
            } else {
                Ok(None)
            };
            match output.map(|output| {
                transpile_and_print_or_write(
                    &input,
                    output.as_ref().map(|s| &s[..]),
                    lint_only,
                    &config,
                )
            }) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Failed to transpile {}: {}", file.path().display(), e);
                }
            }
        },
    )
    .map_err(BoxedError::from)
}

fn transpile_file(input: &str, config: &PPGAConfig) -> Result<String, ()> {
    let source = std::fs::read_to_string(&input).unwrap();
    ppga_to_lua(&source, config.clone()).map_err(|e| {
        eprintln!("Failed to transpile the script:\n{}", e.report_to_string());
        ()
    })
}

fn transpile_and_print_or_write(
    input: &str,
    output: Option<&str>,
    lint_only: bool,
    config: &PPGAConfig,
) -> std::io::Result<bool> {
    println!("--> File `{}`", input);
    let lua = match transpile_file(&input, &config) {
        Ok(lua) => lua,
        Err(_) => return Ok(false),
    };
    if !lint_only {
        if let Some(path) = output {
            println!("--> Writing the transpiled code to `{}`", path);
            std::fs::write(path, lua)?;
        } else {
            println!("{}", lua);
        }
    }
    Ok(true)
}

fn visit_dirs(dir: &std::path::Path, cb: &dyn Fn(&std::fs::DirEntry)) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

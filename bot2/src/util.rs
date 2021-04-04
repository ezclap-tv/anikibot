// TODO: optimize this (remove allocs)
pub fn split_cmd<'a>(prefix: &str, text: &'a str) -> Option<(&'a str, &'a str)> {
    if let Some(cmd) = text.strip_prefix(prefix) {
        let (name, args) = match cmd.split_once(' ') {
            Some((name, args)) => (name, args),
            None => (cmd, ""),
        };

        Some((name, args))
    } else {
        None
    }
}

pub fn parse_args(args: &str, multi: bool) -> script::Variadic {
    lazy_static::lazy_static! {
        static ref RE: regex::Regex = regex::Regex::new(r#"("[^"]*")|[^\s]+"#).unwrap();
    }
    if multi {
        RE.find_iter(args)
            .map(|v| v.as_str())
            .map(|v| {
                if v.starts_with('\"') && v.ends_with('\"') {
                    v.strip_prefix('\"').unwrap().strip_suffix('\"').unwrap()
                } else {
                    v
                }
            })
            .map(|v| v.to_string())
            .collect()
    } else {
        args.split_whitespace().map(String::from).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_command_multi_one_arg() {
        let test = "!name arg0";
        let (name, args) = split_cmd("!", test).unwrap();
        let args = parse_args(args, true);
        assert_eq!(name, "name");
        assert_eq!(&(*args), &["arg0".to_string()]);
    }

    #[test]
    fn parse_command_multi_one_quoted_arg() {
        let test = "!name \"arg0 arg0 arg0\"";
        let (name, args) = split_cmd("!", test).unwrap();
        let args = parse_args(args, true);
        assert_eq!(name, "name");
        assert_eq!(&(*args), &["arg0 arg0 arg0".to_string()]);
    }

    #[test]
    fn parse_command_multi_two_args() {
        let test = "!name arg0 \"arg1 arg1 arg1\"";
        let (name, args) = split_cmd("!", test).unwrap();
        let args = parse_args(args, true);
        assert_eq!(name, "name");
        assert_eq!(&(*args), &["arg0".to_string(), "arg1 arg1 arg1".to_string()]);
    }

    #[test]
    fn parse_command_raw() {
        let test = "!name arg0 arg1 \"arg2 arg3\"";
        let (name, args) = split_cmd("!", test).unwrap();
        let args = parse_args(args, false);
        assert_eq!(name, "name");
        assert_eq!(&(*args), &["arg0", "arg1", "\"arg2", "arg3\""]);
    }

    #[test]
    fn parse_command_different_prefix() {
        let test = "cmd name arg0 arg1 \"arg2 arg3\"";
        let (name, args) = split_cmd("cmd ", test).unwrap();
        let args = parse_args(args, false);
        assert_eq!(name, "name");
        assert_eq!(&(*args), &["arg0", "arg1", "\"arg2", "arg3\""]);
    }
}

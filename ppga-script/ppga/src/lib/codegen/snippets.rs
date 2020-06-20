pub const DEFAULT_OP_NAME: &'static str = "__PPGA_INTERNAL_DEFAULT";

pub const fn default_op_definition() -> &'static str {
    r#"function __PPGA_INTERNAL_DEFAULT(x, default) 
    if x ~= nil then return (x) end
    return (default)
end"#
}

pub const SNIPPETS: [&'static str; 1] = [default_op_definition()];

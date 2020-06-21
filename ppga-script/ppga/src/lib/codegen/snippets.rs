pub const DEFAULT_OP_NAME: &'static str = "__PPGA_INTERNAL_DEFAULT";
pub const ERR_HANDLER_NAME: &'static str = "__PPGA_INTERNAL_HANDLE_ERR";

pub const fn default_op_definition() -> &'static str {
    r#"local function __PPGA_INTERNAL_DEFAULT(x, default) 
    if x ~= nil then return (x) end
    return (default)
end"#
}

pub const fn handle_err_definition() -> &'static str {
    r#"local function __PPGA_INTERNAL_HANDLE_ERR(cb, ...)
    local ok, err = ...
    if err ~= nil then
        ok, err = cb(err)
    end
    return (ok), (err)
end"#
}

pub const SNIPPETS: [&'static str; 2] = [default_op_definition(), handle_err_definition()];

-- PPGA STD SYMBOLS
local function __PPGA_INTERNAL_DEFAULT(x, default) 
    if x ~= nil then return (x) end
    return (default)
end
local function __PPGA_INTERNAL_HANDLE_ERR(cb, ...)
    local ok, err = ...
    if err ~= nil then
        ok, err = cb(err)
    end
    return (ok), (err)
end
local function __PPGA_INTERNAL_DFLT_ERR_CB(err)
    error(err)
end
-- END PPGA STD SYMBOLS


local args = util:get_args(...)
if args.length == 0 then
    return ("gachiHYPER Slapp " .. tostring(args.user))
end
return ("gachiHYPER Slapp " .. tostring(args[0]))
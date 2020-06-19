local args = util:get_args(...)

util:info("args:", args)
util:debug("args:", args)

local first = "None"
if args.length > 0 then
    first = args[0]
end

return "test: " .. args.length .. " args. first arg = " .. first
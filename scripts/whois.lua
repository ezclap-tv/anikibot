local se = api:streamelements()
if se == nil then
    util:error("StreamElements API is unavailable")
    return "WAYTOODANK something broke"
end

local args = util:get_args(...)

local target = args[0]
if target == nil then
    return "WAYTOODANK something broke"
end

util:info(target)

local channel_id, err = se:channels():channel_id(target)
if err ~= nil then
    util:error(err)
    return "WAYTOODANK something broke"
end

return "monkaS 👉 ID of " .. target .. " is " .. channel_id
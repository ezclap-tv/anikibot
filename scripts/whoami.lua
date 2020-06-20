local args = util:get_args(...)

local se = api:streamelements()
if se == nil then
    util:error("StreamElements API is unavailable")
    return "WAYTOODANK something broke"
end

local channel_id, err = se:channels():channel_id(args.user)
if err ~= nil then
    util:error(err)
    return "WAYTOODANK something broke"
end

return "monkaS 👉 your ID is " .. channel_id

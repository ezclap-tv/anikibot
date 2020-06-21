-- PPGA STD SYMBOLS
local function __PPGA_INTERNAL_DEFAULT(x, default) 
    if x ~= nil then return (x) end
    return (default)
end
-- END PPGA STD SYMBOLS


local args = util:get_args(...)

local se = api:streamelements()
if se == nil then
    util:error("StreamElements API is unavailable")
    return ("WAYTOODANK something broke")
end

local channel_id = nil
do
    local _ok_L8S225, _err_L8S225 = se:channels():channel_id(args.user)
    if _err_L8S225 ~= nil then
        util:error(_err_L8S225)
        return (_err_L8S225)
    end
    channel_id = _ok_L8S225
end

return ("monkaS 👉 your ID is " .. tostring(channel_id))
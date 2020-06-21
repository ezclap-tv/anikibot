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
-- END PPGA STD SYMBOLS


local args = util:get_args(...)
local se = nil
do
    local _ok_L1S58, _err_L1S58 = __PPGA_INTERNAL_HANDLE_ERR(function (err)
            util:error("WAYTOODANK something broke")
            return (err)
        end, api:streamelements())
    if _err_L1S58 ~= nil then
        return (_err_L1S58)
    end
    se = _ok_L1S58
end

local channel_id = nil
do
    local _ok_L3S114, _err_L3S114 = __PPGA_INTERNAL_HANDLE_ERR(function (err)
            util:error("WAYTOODANK something broke")
            return (err)
        end, se:channels():channel_id(args.user))
    if _err_L3S114 ~= nil then
        return (_err_L3S114)
    end
    channel_id = _ok_L3S114
end
return ("monkaS 👉 your ID is " .. tostring(channel_id))
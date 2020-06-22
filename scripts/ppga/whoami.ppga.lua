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
    util:error(err)
    return nil, "WAYTOODANK something broke"
end
-- END PPGA STD SYMBOLS


local args = util:get_args(...)
local se = nil
do
    local _ok_L1S59, _err_L1S59 = __PPGA_INTERNAL_HANDLE_ERR(__PPGA_INTERNAL_DFLT_ERR_CB, api:streamelements())
    if _err_L1S59 ~= nil then
        return (_err_L1S59)
    end
    se = _ok_L1S59
end
local channel_id = nil
do
    local _ok_L3S117, _err_L3S117 = __PPGA_INTERNAL_HANDLE_ERR(__PPGA_INTERNAL_DFLT_ERR_CB, se:channels():channel_id(args.user))
    if _err_L3S117 ~= nil then
        return (_err_L3S117)
    end
    channel_id = _ok_L3S117
end
return ("monkaS 👉 your ID is " .. tostring(channel_id))
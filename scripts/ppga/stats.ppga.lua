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


local se = nil
do
    local _ok_L0S29, _err_L0S29 = __PPGA_INTERNAL_HANDLE_ERR(__PPGA_INTERNAL_DFLT_ERR_CB, api:streamelements())
    if _err_L0S29 ~= nil then
        return (nil), (_err_L0S29)
    end
    se = _ok_L0S29
end
local stats = se:stats()
local settings = stats:settings()
local ok = nil
do
    local _ok_L3S114, _err_L3S114 = __PPGA_INTERNAL_HANDLE_ERR(__PPGA_INTERNAL_DFLT_ERR_CB, stats:my_stats())
    if _err_L3S114 ~= nil then
        return (nil), (_err_L3S114)
    end
    ok = _ok_L3S114
end
util:info(ok)
return ("FeelsDnakMan printed the stats for " .. tostring(settings.date) .. " [" .. tostring(settings.tz_name) .. "], interval=" .. tostring(settings.interval) .. " to the console")
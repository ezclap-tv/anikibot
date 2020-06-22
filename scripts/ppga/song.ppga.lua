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


local se = nil
do
    local _ok_L0S29, _err_L0S29 = __PPGA_INTERNAL_HANDLE_ERR(__PPGA_INTERNAL_DFLT_ERR_CB, api:streamelements())
    if _err_L0S29 ~= nil then
        return (_err_L0S29)
    end
    se = _ok_L0S29
end
local ok = nil
do
    local _ok_L1S92, _err_L1S92 = __PPGA_INTERNAL_HANDLE_ERR(__PPGA_INTERNAL_DFLT_ERR_CB, api:streamelements():song_requests():current_song())
    if _err_L1S92 ~= nil then
        return (_err_L1S92)
    end
    ok = _ok_L1S92
end
return ("CheemJam now playing " .. tostring(ok.title) .. " [ https://youtu.be/" .. tostring(ok.videoId) .. " ]")
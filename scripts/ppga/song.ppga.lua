-- PPGA STD SYMBOLS
local function __PPGA_INTERNAL_DEFAULT(x, default) 
    if x ~= nil then return (x) end
    return (default)
end
-- END PPGA STD SYMBOLS


local se = api:streamelements()
if se == nil then
    util:error("StreamElements API is unavailable")
    return ("StreamElements API is not configured")
end

local ok = nil
do
    local _ok_L6S213, _err_L6S213 = api:streamelements():song_requests():current_song()
    if _err_L6S213 ~= nil then
        util:error(_err_L6S213)
        return (_err_L6S213)
    end
    ok = _ok_L6S213
end
return ("CheemJam now playing " .. tostring(ok.title) .. " [ https://youtu.be/" .. tostring(ok.videoId) .. " ]")
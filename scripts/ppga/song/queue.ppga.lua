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


local yt = nil
do
    local _ok_L0S31, _err_L0S31 = __PPGA_INTERNAL_HANDLE_ERR(function (err)
            util:error("WAYTOODANK something broke")
            return (err)
        end, api:youtube_playlist())
    if _err_L0S31 ~= nil then
        return (_err_L0S31)
    end
    yt = _ok_L0S31
end
local se = nil
do
    local _ok_L1S63, _err_L1S63 = __PPGA_INTERNAL_HANDLE_ERR(function (err)
            util:error("WAYTOODANK something broke")
            return (err)
        end, api:streamelements())
    if _err_L1S63 ~= nil then
        return (_err_L1S63)
    end
    se = _ok_L1S63
end
local sr = se:song_requests()
local args = util:dbg(util:get_args(...))

local playlist = args[0]
if playlist == nil then
    return ("A playlist link was not provided")
end
local num_songs = __PPGA_INTERNAL_DEFAULT(args[1], 10)

util:info("Queueing " .. tostring(num_songs) .. " from " .. tostring(playlist))

local _ = nil
do
    local _ok_L13S355, _err_L13S355 = __PPGA_INTERNAL_HANDLE_ERR(function (err)
            util:error("WAYTOODANK something broke")
            return (err)
        end, yt:configure(playlist, num_songs))
    if _err_L13S355 ~= nil then
        return (_err_L13S355)
    end
    _ = _ok_L13S355
end
local song_urls = nil
do
    local _ok_L14S398, _err_L14S398 = __PPGA_INTERNAL_HANDLE_ERR(function (err)
            util:error("WAYTOODANK something broke")
            return (err)
        end, yt:get_playlist_videos())
    if _err_L14S398 ~= nil then
        return (_err_L14S398)
    end
    song_urls = _ok_L14S398
end
local _ = nil
do
    local _ok_L15S433, _err_L15S433 = __PPGA_INTERNAL_HANDLE_ERR(function (err)
            util:error("WAYTOODANK something broke")
            return (err)
        end, sr:queue_many(song_urls))
    if _err_L15S433 ~= nil then
        return (_err_L15S433)
    end
    _ = _ok_L15S433
end

return ("CheemJam successfully queued " .. tostring(#(song_urls)) .. " songs")
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


local yt = nil
do
    local _ok_L0S31, _err_L0S31 = __PPGA_INTERNAL_HANDLE_ERR(__PPGA_INTERNAL_DFLT_ERR_CB, api:youtube_playlist())
    if _err_L0S31 ~= nil then
        return (nil), (_err_L0S31)
    end
    yt = _ok_L0S31
end
local se = nil
do
    local _ok_L1S64, _err_L1S64 = __PPGA_INTERNAL_HANDLE_ERR(__PPGA_INTERNAL_DFLT_ERR_CB, api:streamelements())
    if _err_L1S64 ~= nil then
        return (nil), (_err_L1S64)
    end
    se = _ok_L1S64
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
    local _ok_L13S368, _err_L13S368 = __PPGA_INTERNAL_HANDLE_ERR(__PPGA_INTERNAL_DFLT_ERR_CB, yt:configure(playlist, num_songs))
    if _err_L13S368 ~= nil then
        return (nil), (_err_L13S368)
    end
    _ = _ok_L13S368
end
local song_urls = nil
do
    local _ok_L14S412, _err_L14S412 = __PPGA_INTERNAL_HANDLE_ERR(__PPGA_INTERNAL_DFLT_ERR_CB, yt:get_playlist_videos())
    if _err_L14S412 ~= nil then
        return (nil), (_err_L14S412)
    end
    song_urls = _ok_L14S412
end
local _ = nil
do
    local _ok_L15S448, _err_L15S448 = __PPGA_INTERNAL_HANDLE_ERR(__PPGA_INTERNAL_DFLT_ERR_CB, sr:queue_many(song_urls))
    if _err_L15S448 ~= nil then
        return (nil), (_err_L15S448)
    end
    _ = _ok_L15S448
end
return ("CheemJam successfully queued " .. tostring(#(song_urls)) .. " songs")
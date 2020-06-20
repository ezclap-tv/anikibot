local se = api:streamelements()
if se == nil then
    util:error("StreamElements API is unavailable")
    return ("StreamElements API is not configured")
end

local ok, err = api:streamelements():song_requests():current_song()
if err ~= nil then
    return ("WAYTOODANK something broke")
end

return ("CheemJam now playing " .. tostring(ok.title) .. " [ https://youtu.be/" .. tostring(ok.videoId) .. " ]")
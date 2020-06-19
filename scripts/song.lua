local se = api:streamelements()
if se == nil then
    util:error("StreamElements API is unavailable")
    return "WAYTOODANK somethin broke"
end

local response, error = se:song_requests():current_song()
if error ~= nil then
    return "WAYTOODANK something broke"
end

return "CheemJam now playing " .. response.title .. " [ https://youtu.be/" .. response.videoId .. " ]"
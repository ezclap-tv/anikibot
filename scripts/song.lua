local response, error = api:streamelements():song_requests():current_song()
if error ~= nil then
    return "WAYTOODANK something broke"
end

return "CheemJam now playing " .. response.title .. " [ https://youtu.be/" .. response.videoId .. " ]"
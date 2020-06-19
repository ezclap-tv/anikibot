
local yt = api:youtube_playlist()
if yt == nil then
    util:error("Youtube API unavailable!")
    return "FeelsDankMan something broke"
end
local se = api:streamelements()
if se == nil then
    util:error("StreamElements API unavailable!")
    return "FeelsDankMan something broke"
end
local sr = se:song_requests()

local args = util:get_args(...)
util:info(args)
local playlist = args[0]
if playlist == nil then
    return "A playlist link was not provided"
end
local num_songs = args[1] ~= nil and args[1] or 10

util:info("Queueing " .. num_songs .. " from " .. playlist)
local _, err = yt:configure(playlist, num_songs)
if err ~= nil then
    util:error(err)
    return "FeelsDankMan something broke"
end

local song_urls, err = yt:get_playlist_videos()
if err ~= nil then
    util:error(err)
    return "FeelsDankMan something broke"
end

local _, err = sr:queue_many(song_urls)
if err ~= nil then
    util:error(err)
    return "FeelsDankMan something broke"
end

return "CheemJam successfully queued " .. util:len(song_urls) .. " songs"
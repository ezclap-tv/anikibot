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


local VIDEO_ID = "PFyMhNZB-lc"
local STAGE_00 = "C'MON BOYS CHANNEL YOUR ♂ENERGY♂"
local STAGE_01 = "gachiHop . o O ( gachiHYPER )"
local STAGE_02 = "gachiPRIDE aaahhh..."
local STAGE_03 = "gachiHYPER AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAHHHHHHHHHH " .. "gachiHYPER AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAHHHHHHHHHH " .. "gachiHYPER AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAHHHHHHHHHH " .. "gachiHYPER AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAHHHHHHHHHH "
local DELAY_00 = 20000
local DELAY_01 = 20000
local DELAY_02 = 20000
local start = false
local args = util:get_args(...)
if args.length > 0 and args[0] == "force" then
    start = true
else
    local se = nil
    do
        local _ok_L23S965, _err_L23S965 = __PPGA_INTERNAL_HANDLE_ERR(__PPGA_INTERNAL_DFLT_ERR_CB, api:streamelements())
        if _err_L23S965 ~= nil then
            return (nil), (_err_L23S965)
        end
        se = _ok_L23S965
    end
    local ok = nil
    do
        local _ok_L24S1015, _err_L24S1015 = __PPGA_INTERNAL_HANDLE_ERR(__PPGA_INTERNAL_DFLT_ERR_CB, se:song_requests():current_song())
        if _err_L24S1015 ~= nil then
            return (nil), (_err_L24S1015)
        end
        ok = _ok_L24S1015
    end
    util:info("Current song is " .. tostring(ok.title) .. ", id = " .. tostring(ok.videoId) .. "; comparing with: " .. tostring(VIDEO_ID) .. ".")
    start = ok.videoId == VIDEO_ID
end
if not(start) then
    util:info("Song didn't match, stopping the script.")
    return 
end
bot:send(args.channel, STAGE_00)
util:wait(DELAY_00)
bot:send(args.channel, STAGE_01)
util:wait(DELAY_01)
bot:send(args.channel, STAGE_02)
util:wait(DELAY_02)
return (STAGE_03)
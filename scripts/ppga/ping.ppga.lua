-- PPGA STD SYMBOLS
local function __PPGA_INTERNAL_DEFAULT(x, default) 
    if x ~= nil then return (x) end
    return (default)
end
-- END PPGA STD SYMBOLS


local uptime = bot:uptime()
return ("FeelsDankMan 🕒 uptime is " .. tostring(uptime))
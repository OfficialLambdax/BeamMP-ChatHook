-- Made by Neverless @ BeamMP. Issues? Feel free to ask.

package.loaded["libs/Build"] = nil
package.loaded["libs/UDPClient"] = nil
package.loaded["libs/ServerConfig"] = nil

local Build = require("libs/Build")
local UDPClient = require("libs/UDPClient")
local ServerConfig = require("libs/ServerConfig")

local BOT_IP = "172.17.0.1"
local BOT_PORT = 30813

local Socket = nil

-- ----------------------------------------------------------------------
-- Common
local function filePath(string)
	local _, pos = string:find(".*/")
	if pos == nil then return nil end
	
	return string:sub(1, pos)
end

local function myPath()
	local source_path = debug.getinfo(2).source:gsub("\\", "/")
	if source_path:sub(1, 1) == '@' then return filePath(source_path:sub(2)) end
	return filePath(source_path)
end

-- ----------------------------------------------------------------------
-- Event stuff
function onChatMessage(player_id, player_name, message)
	if message:len() == 0 or message:sub(1, 1) == '/' then return end
	Socket:send(Build.playerMessage(player_id, message))
end

function onPlayerJoin(player_id)
	Socket:send(Build.playerJoin(player_id))
end

function onPlayerDisconnect(player_id)
	Socket:send(Build.playerLeft(player_id))
end

function onLoad()
	Socket:send(Build.serverOnline())
end

-- ----------------------------------------------------------------------
-- Init
function onInit()
	local bin_path = myPath() .. "bin/udp"
	local os_name = MP.GetOSName()
	if os_name == "Windows" then
		bin_path = bin_path .. '.exe'
		
	elseif os_name == "Linux" then
		os.execute('chmod +x "' .. bin_path .. '"')
		
	else
		print('ChatHook. Error. Unsupported Plattform')
		return
	end
	
	Socket = UDPClient(bin_path, BOT_IP, BOT_PORT)
	
	Build.setServerName(ServerConfig.Get("General", "Name"))
	
	MP.RegisterEvent("onChatMessage", "onChatMessage")
	MP.RegisterEvent("onPlayerJoin", "onPlayerJoin")
	MP.RegisterEvent("onPlayerDisconnect", "onPlayerDisconnect")
	onLoad()
end

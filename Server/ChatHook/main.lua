-- Made by Neverless @ BeamMP. Issues? Feel free to ask.
local VERSION = "0.1" -- 09.06.2025 (DD.MM.YYYY)

package.loaded["libs/Build"] = nil
package.loaded["libs/UDPClient"] = nil
package.loaded["libs/ServerConfig"] = nil

local Build = require("libs/Build")
local UDPClient = require("libs/UDPClient")
local ServerConfig = require("libs/ServerConfig")

local CHATHOOK_IP = "172.17.0.1"
--local CHATHOOK_IP = "127.0.0.1"
local UDP_PORT = 30813

local Socket = nil

-- ----------------------------------------------------------------------
-- Common
local function tableSize(table)
	local size = 0
	for _, _ in pairs(table) do
		size = size + 1
	end
	return size
end

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

function onScriptMessage(message)
	Socket:send(Build.scriptMessage(message))
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

function onReload()
	Socket:send(Build.serverReload())
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
	
	Socket = UDPClient(bin_path, CHATHOOK_IP, UDP_PORT)
	
	Build.setServerName(ServerConfig.Get("General", "Name"))
	Build.setMaxPlayers(ServerConfig.Get("General", "MaxPlayers"))
	
	MP.RegisterEvent("onChatMessage", "onChatMessage")
	MP.RegisterEvent("onPlayerJoin", "onPlayerJoin")
	MP.RegisterEvent("onPlayerDisconnect", "onPlayerDisconnect")
	MP.RegisterEvent("onScriptMessage", "onScriptMessage")
	
	if tableSize(MP.GetPlayers() or {}) == 0 then
		onLoad()
	else
		onReload()
	end
end

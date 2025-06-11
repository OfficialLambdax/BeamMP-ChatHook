-- Made by Neverless @ BeamMP. Issues? Feel free to ask.
local VERSION = "0.1" -- 10.06.2025 (DD.MM.YYYY)
local SCRIPT_REF = "ChatHook"

package.loaded["libs/Build"] = nil
package.loaded["libs/UDPClient"] = nil
package.loaded["libs/ServerConfig"] = nil
package.loaded["libs/PlayerCount"] = nil
package.loaded["libs/colors"] = nil
package.loaded["libs/Log"] = nil

local Build = require("libs/Build")
local UDPClient = require("libs/UDPClient")
local ServerConfig = require("libs/ServerConfig")
local PlayerCount = require("libs/PlayerCount")
local Color = require("libs/colors")
local Log = require("libs/Log").setCollectMode(true)

local CHATHOOK_IP = "172.17.0.1"
--local CHATHOOK_IP = "127.0.0.1"
local UDP_PORT = 30813

local Socket = nil

local IS_START = _G.IS_START == nil
_G.IS_START = true

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
-- Buf
local Buf = {_buf = {}}
function Buf:add(build)
	table.insert(self._buf, build)
end

function Buf:size()
	return #self._buf
end

function Buf:take()
	local ref = self._buf
	self._buf = {}
	return ref
end

-- ----------------------------------------------------------------------
-- Buf print
function bufPrint()
	Log.printCollect()
	if Buf:size() > 0 then Socket:send(Build.wrap(Buf:take())) end
end

-- ----------------------------------------------------------------------
-- Event stuff
function onChatMessage(player_id, player_name, message)
	if message:len() == 0 or message:sub(1, 1) == '/' then return end
	Buf:add(Build.playerMessage(player_id, message))
end

function onScriptMessage(message, script_ref)
	if message == nil or message:len() == 0 then return end
	Buf:add(Build.scriptMessage(script_ref, message))
end

function onPlayerJoin(player_id)
	PlayerCount.add(player_id)
	Buf:add(Build.playerJoin(player_id))
end

function onPlayerDisconnect(player_id)
	PlayerCount.remove(player_id)
	Buf:add(Build.playerLeft(player_id))
end

-- ----------------------------------------------------------------------
-- Init
function onInit()
	MP.CancelEventTimer("chathook_bufprint")
	MP.RegisterEvent("chathook_bufprint", "bufPrint")
	MP.CreateEventTimer("chathook_bufprint", 1000)
	
	Log.load("====. Loading ChatHook .====", SCRIPT_REF)
	Log.load("> Version: " .. VERSION, SCRIPT_REF)
	Log.load("> Build Packet Version: " .. Build.VERSION, SCRIPT_REF)
	Log.info("^ ^n(This must match with the ChatHook Container version)^r", SCRIPT_REF)
	
	-- eval server config parameters
	local server_name = ServerConfig.Get("General", "Name")
	if server_name == nil or server_name:len() == 0 then
		Log.fatal('Server doesnt contain a server name or has no ServerConfig.toml', SCRIPT_REF)
		return
	end
	
	local max_players = ServerConfig.Get("General", "MaxPlayers")
	if max_players == nil then
		Log.fatal('Server doesnt contain a MaxPlayers value or has no ServerConfig.toml', SCRIPT_REF)
		return
	end
	
	Log.ok('> Server Name: ' .. Color.convertToConsole(server_name), SCRIPT_REF)
	Log.ok('> Max Players: ' .. max_players, SCRIPT_REF)
	Build.setServerName(server_name)
	Build.setMaxPlayers(max_players)
	
	-- eval udp bin
	local bin_path = myPath() .. "bin/udp"
	local os_name = MP.GetOSName()
	Log.info('> Operating System: ' .. os_name, SCRIPT_REF)
	if os_name == "Windows" then
		Log.load('> Applying Windows patch', SCRIPT_REF)
		bin_path = bin_path .. '.exe'
		
	elseif os_name == "Linux" then
		Log.load('> Applying Linux patch', SCRIPT_REF)
		os.execute('chmod +x "' .. bin_path .. '"')
		
	else
		Log.fatal('Unsupported Operating System', SCRIPT_REF)
		return
	end
	if not FS.Exists(bin_path) then
		Log.fatal('Cannot find udp binary in "' .. bin_path .. '"', SCRIPT_REF)
		return
	end
	
	Log.load('> Building UDPSocket for ' .. CHATHOOK_IP .. ':' .. UDP_PORT, SCRIPT_REF)
	Socket = UDPClient(bin_path, CHATHOOK_IP, UDP_PORT)
	if Socket == nil then
		Log.fatal('Cannot create UDPSocket', SCRIPT_REF)
		return
	end
	Log.ok('> Initizalized UDPSocket', SCRIPT_REF)
	
	MP.RegisterEvent("onChatMessage", "onChatMessage")
	MP.RegisterEvent("onPlayerJoin", "onPlayerJoin")
	MP.RegisterEvent("onPlayerDisconnect", "onPlayerDisconnect")
	MP.RegisterEvent("onScriptMessage", "onScriptMessage")
	
	if IS_START then
		Buf:add(Build.serverOnline())
	else
		Buf:add(Build.serverReload())
	end
	
	-- hotreload
	for player_id, player_name in pairs(MP.GetPlayers() or {}) do
		PlayerCount.add(player_id)
		Log.ok('Hotreloaded "' .. player_name .. '"', SCRIPT_REF)
	end
	
	Log.load("=====. ChatHook Loaded .====", SCRIPT_REF)
	Log.printCollect()
end

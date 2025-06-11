-- Made by Neverless @ BeamMP. Issues? Feel free to ask.
local Base64 = require("libs/base64")
local PlayerCount = require("libs/PlayerCount")

local M = {
	SERVER_NAME = "",
	MAX_PLAYERS = 0,
	VERSION = 2,
}

local function tableSize(table)
	local size = 0
	for _, _ in pairs(table) do
		size = size + 1
	end
	return size
end

local function wrap(table)
	return Base64.encode(Util.JsonEncode(table))
end

local function baseBuild()
	return {
		server_name = M.SERVER_NAME,
		player_count = PlayerCount.count(),
		player_max = M.MAX_PLAYERS,
		version = M.VERSION
	}
end

local function base(into)
	local from = baseBuild()
	for k, v in pairs(from) do
		into[k] = v
	end
	return into
end


M.setServerName = function(server_name)
	M.SERVER_NAME = server_name
end

M.setMaxPlayers = function(max_players)
	M.MAX_PLAYERS = max_players
end

M.wrap = function(contents)
	return wrap(base({
		contents = contents
	}))
end

M.playerMessage = function(player_id, message)
	return {
		type = 1,
		content = {
			player_name = MP.GetPlayerName(player_id),
			chat_message = message
		}
	}
end

M.scriptMessage = function(script_ref, message)
	return {
		type = 6,
		content = {
			script_ref = script_ref or '',
			chat_message = message
		}
	}
end

M.serverOnline = function()
	return {
		type = 2,
	}
end

M.serverReload = function()
	return {
		type = 5,
	}
end

M.playerJoin = function(player_id)
	return {
		type = 3,
		content = {
			player_name = MP.GetPlayerName(player_id)
		}
	}
end

M.playerLeft = function(player_id)
	return {
		type = 4,
		content = {
			player_name = MP.GetPlayerName(player_id)
		}
	}
end

return M
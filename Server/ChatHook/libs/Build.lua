-- Made by Neverless @ BeamMP. Issues? Feel free to ask.
local Base64 = require("libs/base64")

local M = {
	SERVER_NAME = "",
	MAX_PLAYERS = 0,
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
		player_count = tableSize(MP.GetPlayers() or {}),
		player_max = M.MAX_PLAYERS
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


M.playerMessage = function(player_id, message)
	return wrap(base({
		type = 1,
		content = {
			player_name = MP.GetPlayerName(player_id),
			chat_message = message
		}
	}))
end

M.scriptMessage = function(message)
	return wrap(base({
		type = 6,
		content = {
			chat_message = message
		}
	}))
end

M.serverOnline = function()
	return wrap(base({
		type = 2,
	}))
end

M.serverReload = function()
	return wrap(base({
		type = 5,
	}))
end

M.playerJoin = function(player_id)
	return wrap(base({
		type = 3,
		content = {
			player_name = MP.GetPlayerName(player_id)
		}
	}))
end

M.playerJoinC = function(player_name)
	return wrap(base({
		type = 3,
		content = {
			player_name = player_name
		}
	}))
end

M.playerLeft = function(player_id)
	return wrap(base({
		type = 4,
		content = {
			player_name = MP.GetPlayerName(player_id)
		}
	}))
end

return M
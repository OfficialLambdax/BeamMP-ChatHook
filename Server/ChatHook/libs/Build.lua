-- Made by Neverless @ BeamMP. Issues? Feel free to ask.
local Base64 = require("libs/base64")

local M = {
	SERVER_NAME = ""
}

local function wrap(table)
	return Base64.encode(Util.JsonEncode(table))
end

M.setServerName = function(server_name)
	M.SERVER_NAME = server_name
end

M.playerMessage = function(player_id, message)
	return wrap({
		type = 1,
		server_name = M.SERVER_NAME,
		content = {
			player_name = MP.GetPlayerName(player_id),
			chat_message = message
		}
	})
end

M.serverOnline = function()
	return wrap({
		type = 2,
		server_name = M.SERVER_NAME
	})
end

M.playerJoin = function(player_id)
	return wrap({
		type = 3,
		server_name = M.SERVER_NAME,
		content = {
			player_name = MP.GetPlayerName(player_id)
		}
	})
end

M.playerLeft = function(player_id)
	return wrap({
		type = 4,
		server_name = M.SERVER_NAME,
		content = {
			player_name = MP.GetPlayerName(player_id)
		}
	})
end

return M
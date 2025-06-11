local M = {}

local PLAYERS = {}

local function tableSize(table)
	local size = 0
	for _, _ in pairs(table) do
		size = size + 1
	end
	return size
end

M.add = function(player_id)
	PLAYERS[player_id] = true
end

M.remove = function(player_id)
	if not M.exists(player_id) then return end
	PLAYERS[player_id] = nil
	return true
end

M.exists = function(player_id)
	return PLAYERS[player_id] ~= nil
end

M.dif = function()
	return tableSize(MP.GetPlayers() or {}) - M.count()
end

M.count = function()
	return tableSize(PLAYERS)
end

return M

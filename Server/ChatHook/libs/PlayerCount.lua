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
	PLAYERS[player_id] = nil
end

M.count = function()
	return tableSize(PLAYERS)
end

return M

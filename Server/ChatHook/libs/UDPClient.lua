-- Made by Neverless @ BeamMP. Issues? Feel free to ask.
--[[
	Windows/Linux must unfortunately be seperated atm.
	
	io.popen takes exceptionally long to execute on linux for some reason.
	So on windows the data is written to stdin, while on linux its given as an argument (with all its downsides of doing so)
	
	Use b64 encoding.
]]

local Log = require("libs/Log")
local RSocket -- loaded during init if Linux

local SCRIPT_REF = "libUDPClient"

local function getUdpHandle(self)
	return io.popen(string.format(
		'%s %s %s',
		self._bin, self._ip, self._port
	), "w")
end

local function sendWindows(self, data)
	local handle = getUdpHandle(self)
	if not handle then return end
	
	handle:write(data)
	handle:close()
end

local function sendLinux(self, data)
	if self._client then
		self._client:send(data)
	else
		os.execute(string.format(
			'%s %s %s %s',
			self._bin, self._ip, self._port, data
		))
	end
end

local function correctBinPath(os_name, bin_path)
	if os_name == "Linux" then return bin_path end
	if os_name == "Windows" then return bin_path:gsub("%/", "\\") end
	return nil
end

return function(bin_path, ip, port)
	local os_name = MP.GetOSName()
	bin_path = correctBinPath(os_name, bin_path)
	if not bin_path or not FS.Exists(bin_path) then return end

	local client
	if os_name == "Linux" then
		--package.loaded["rsocket"] = nil
		local use_lib, lib = pcall(require, "rsocket")

		if not use_lib then
			Log.error('Cannot initialize RSocket. Using fallback.', SCRIPT_REF)
		else
			Log.load('Successfully loaded Experimental RSocket', SCRIPT_REF)
			RSocket = lib
			local is_ok, socket = pcall(RSocket.udpClient, ip, port)
			if not is_ok then
				Log.error('Cannot create RSocket UDP client. Using fallback.', SCRIPT_REF)
			else
				Log.load("Using RSocket UDP client.", SCRIPT_REF)
				client = socket
			end
		end
	end
	local udp = {
		_os = os_name,
		_bin = bin_path,
		_ip = ip,
		_port = port,
		_client = client, -- nil/socket
	}
	function udp:send(data)
		if self._os == "Windows" then return sendWindows(self, data) end
		if self._os == "Linux" then return sendLinux(self, data) end
	end

	function udp:close()
		if self._client then self._client:close() end
	end
	
	return udp
end

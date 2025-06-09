--[[
	Windows/Linux must unfortunately be seperated atm.
	
	io.popen takes exceptionally long to execute on linux for some reason.
	So on windows the data is written to stdin, while on linux its given as an argument (with all its downsides of doing so)
	
	Use b64 encoding.
]]

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
	os.execute(string.format(
		'%s %s %s %s',
		self._bin, self._ip, self._port, data
	))
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
	local udp = {
		_os = os_name,
		_bin = bin_path,
		_ip = ip,
		_port = port
	}
	function udp:send(data)
		if self._os == "Windows" then return sendWindows(self, data) end
		if self._os == "Linux" then return sendLinux(self, data) end
	end
	
	return udp
end

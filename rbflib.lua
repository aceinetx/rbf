if rawget(_G, "rbf_throw") == nil then
	--[[
		rbf_throw
		throw a rbf exception
	]]--
	function rbf_throw(what) end
end
if rawget(_G, "rbf_getptr") == nil then
	--[[
		rbf_getptr
		get brainfuck pointer location
	]]--
	function rbf_getptr() end
end
if rawget(_G, "rbf_getmem") == nil then
	--[[
		rbf_getmem
		get all brainfuck memory stored in a array
	]]--
	function rbf_getmem() end
end
if rawget(_G, "rbf_setnobounds") == nil then
	--[[
		rbf_setnobounds
		whether the brainfuck memory should have bounds
	]]--
	function rbf_setnobounds(state) end
end
if rawget(_G, "rbf_setmemlen") == nil then
	--[[
		rbf_setmemlen
		resize brainfuck memory
	]]--
	function rbf_setmemlen(size) end
end
if rawget(_G, "rbf_readmem") == nil then
	--[[
		rbf_readmem
		read brainfuck memory from location
	]]--
	function rbf_readmem(location) end
end
if rawget(_G, "rbf_writemem") == nil then
	--[[
		rbf_writemem
		write to a location in brainfuck memory
	]]--
	function rbf_writemem(location, value) end
end
if rawget(_G, "rbf_setptr") == nil then
	--[[
		rbf_setptr
		sets a brainfuck pointer to x
	]]--
	function rbf_setptr(x) end
end
if rawget(_G, "rbf_custominstruction") == nil then
	--[[
		rbf_custominstruction
		create a custom instruction. Note that funcname is the string name of the function that will be called when custom instruction is invoked
	]]--
	function rbf_custominstruction(instruction, funcname) end
end
if rawget(_G, "rbf_customcmd") == nil then
	--[[
		rbf_customcmd
		create a custom command. Note that funcname is the string name of the function that will be called when custom command is invoked
	]]--
	function rbf_customcmd(cmd, funcname) end
end
if rawget(_G, "rbf_exec") == nil then
	--[[
		rbf_exec
		execute brainfuck code
	]]--
	function rbf_exec(code) end
end
if rawget(_G, "exit") == nil then
	--[[
		exit
		stop execution
	]]--
	function exit(code) end
end
if rawget(_G, "wait") == nil then
	--[[
		wait
		waits for N amount of seconds
	]]--
	function wait(seconds) end
end
if rawget(_G, "print_rgb") == nil then
	--[[
		print_rgb
		prints a colorful message
	]]--
	function print_rgb(msg, r, g, b) end
end
if rawget(_G, "err") == nil then
	--[[
		err
		prints a red highlited message
	]]--
	function err(msg) end
end
if rawget(_G, "warn") == nil then
	--[[
		warn (rbf)
		prints a orange highlited message
	]]--
	function warn(msg) end
end

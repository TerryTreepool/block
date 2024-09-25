function alalize_data(data)
	if(#data ~= 31)then return false
	end

	if(data[1] == 0x1e and data[2] == 0xff and data[9] == 0xf5 and 
	data[29]==0xf5 and data[30] == 0xf5 and data[31] == 0xf5)
	then
			thing_data:set("module", "vanhai")
			return true
	end

	return false

end

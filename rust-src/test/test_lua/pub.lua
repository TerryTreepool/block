
function array_concat(d1, d2) 

    local dataes = {d1, d2}
    local t = {}

    for i = 1, 2 do 
        local array = dataes[i] 
        if (type(array) == "table") then 
         for j = 1, #array do 
          t[#t+1] = array[j] 
         end 
        else 
         t[#t+1] = array 
        end 
       end 
    return t 
end 

function pairsByKeys(t)
    local key_array = {}
    local idx = 0
    for k, v in pairs(t) do
        key_array[#key_array+1] = k
        -- idx = idx + 1
        -- key_array[k] = idx
    end

    table.sort(key_array)
    local i = 0

    return function()
        i = i + 1
        return key_array[i], t[key_array[i]]
    end
end

function tableToVec(tbl)
    local str = {}
    local idx = 0

    for k, v in pairsByKeys(tbl) do
        if type(v) == "table" then
            str = array_concat(str, tableToVec(v))
        else
            str[#str+1] = v
        end
        idx = idx + 1
    end

    return str
end

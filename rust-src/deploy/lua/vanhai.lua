function get_it()
    return userdata:get_value()
end

function set_it(i)
    return userdata:set_value(i)
end

function get_constant()
    return userdata.get_constant()
end

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
function getBleData() 
    local bleData = {
        a = 0xf5,
        b = 0xf5,
        c = 0xf5,
        d = 0xf5,
        e = 0xf5,
        f = 0xf5,
        g = 0xf5,
        h = 0xf5,
        i = 0xf5,
        j = 0xf5,
        k = 0xf5,
        l = 0xf5,
        m = 0xf5,
        n = 0xf5,
        o = 0xf5,
        p = 0xf5,
    }
    return bleData
end
function getBle() 
    
    local ble =
    {
        aa_len = 0x1e,
        ab_type = 0xff,
        ac_pack_mark = 0xb0,
        ad_pack_order = 0x55,
        ae_pack_relay = 0xa0,
        af_serial_num = 0,
        ag_group = 0,
        ah_cmd = 0,
        ai_addr = {},
        aj_data,
    }
   return ble
    
end

function search_thing()
    local bleData = getBleData()
    local ble = getBle()
    ble.af_serial_num = configure_data:gen_serial_num()%255
    ble.ag_group = 0x00
    ble.ah_cmd = 0x03

    -- [8 - 14]地址(7) 
    ble.ai_addr[1] = 0xf5
    ble.ai_addr[2] = 0xf0
    ble.ai_addr[3] = 0xf0
    ble.ai_addr[4] = 0xf0
    ble.ai_addr[5] = 0xf0
    ble.ai_addr[6] = 0xf0
    ble.ai_addr[7] = 0xf0
    -- [15-30]数据(16)
    bleData.a = 0x00
    ble.aj_data = bleData

    return tableToVec(ble)
end
--搜索到设备后发送静默设备
function silence_thing()
    local bleData = getBleData()
    local ble = getBle()
    local deviceMac = stringToMac(thing_data["thing_mac"])
    local boxMac = stringToMac(configure_data:core_mac())
    ble.af_serial_num = configure_data:gen_serial_num()%255
    --盒子mac的最后一位作为群组
    ble.ag_group = boxMac[7]
    ble.ah_cmd = 0x04

    -- [8 - 14]地址(7) 
    bleData.b = boxMac[1]
    bleData.c = boxMac[2]
    bleData.d = boxMac[3]
    bleData.e = boxMac[4]
    bleData.f = boxMac[5]
    bleData.g = boxMac[6]
    bleData.h = boxMac[7]

    ble.aj_data = bleData

    return tableToVec(ble)
end
function stringToMac(str)
    
    return {tonumber(string.sub(str,1,2), 16),tonumber(string.sub(str,3,4), 16),tonumber(string.sub(str,5,6), 16),tonumber(string.sub(str,7,8), 16),tonumber(string.sub(str,9,10), 16),tonumber(string.sub(str,11,12), 16),tonumber(string.sub(str,13,14), 16)}
end

--添加设备就是绑定群组以让设备有中级功能
function add_thing()
    local bleData = getBleData()
    local ble = getBle()
    local thingMac = stringToMac(thing_data["thing_mac"])
    local boxMac = stringToMac(configure_data:core_mac())
    ble.af_serial_num = configure_data:gen_serial_num()%255
   
    
    --盒子mac的最后一位作为群组
    ble.ag_group = boxMac[7]
    
    ble.ah_cmd = 0x06

    -- [8 - 14]地址(7) 
    ble.ai_addr[1] = thingMac[1]
    ble.ai_addr[2] = thingMac[2]
    ble.ai_addr[3] = thingMac[3]
    ble.ai_addr[4] = thingMac[4]
    ble.ai_addr[5] = thingMac[5]
    ble.ai_addr[6] = thingMac[6]
    ble.ai_addr[6] = thingMac[7]

    -- [15-30]数据(16)
 
    bleData.a = 0x10
    bleData.b = boxMac[1]
    bleData.c = boxMac[2]
    bleData.d = boxMac[3]
    bleData.e = boxMac[4]
    bleData.f = boxMac[5]
    bleData.g = boxMac[6]
    bleData.h = boxMac[7]

    ble.aj_data = bleData

    return tableToVec(ble)
end
--删除设备就是取消与网关的绑定
function remove_thing()
    local bleData = getBleData()
    local ble = getBle()
    local thingMac = stringToMac(thing_data["thing_mac"])
    local boxMac = stringToMac(configure_data:core_mac())
    ble.af_serial_num = configure_data:gen_serial_num()%255
    --盒子mac的最后一位作为群组
    ble.ag_group = boxMac[7]
    
    ble.ah_cmd = 0x06

    -- [8 - 14]地址(7) 
    ble.ai_addr[1] = thingMac[1]
    ble.ai_addr[2] = thingMac[2]
    ble.ai_addr[3] = thingMac[3]
    ble.ai_addr[4] = thingMac[4]
    ble.ai_addr[5] = thingMac[5]
    ble.ai_addr[6] = thingMac[6]
    ble.ai_addr[6] = thingMac[7]

    -- [15-30]数据(16)
  
    bleData.a = 0x10
    bleData.b = boxMac[1]
    bleData.c = boxMac[2]
    bleData.d = boxMac[3]
    bleData.e = boxMac[4]
    bleData.f = boxMac[5]
    bleData.g = boxMac[6]
    bleData.h = boxMac[7]


    ble.aj_data = bleData

    return tableToVec(ble)
end
--设备与遥控器配对
function pair_thing()
    local bleData = getBleData()
    local ble = getBle()
    local thingMac = stringToMac(thing_data["thing_mac"])
    local boxMac = stringToMac(configure_data:core_mac())
    ble.af_serial_num = configure_data:gen_serial_num()%255
    --盒子mac的最后一位作为群组
    ble.ag_group = boxMac[7]
   
    ble.ah_cmd = 0x06

    -- [8 - 14]地址(7) 
    ble.ai_addr[1] = thingMac[1]
    ble.ai_addr[2] = thingMac[2]
    ble.ai_addr[3] = thingMac[3]
    ble.ai_addr[4] = thingMac[4]
    ble.ai_addr[5] = thingMac[5]
    ble.ai_addr[6] = thingMac[6]
    ble.ai_addr[7] = thingMac[7]

    -- [15-30]数据(16)
   
    bleData.a = 0x01
    bleData.b = 0x01
    bleData.c = 30
  
    ble.aj_data = bleData

    return tableToVec(ble)
end
--设备与遥控器解除配对
function remove_pair_thing()
    local bleData = getBleData()
    local ble = getBle()
    local thingMac = stringToMac(thing_data["thing_mac"])
    local boxMac = stringToMac(configure_data:core_mac())
    ble.af_serial_num = configure_data:gen_serial_num()%255
    --盒子mac的最后一位作为群组
    ble.ag_group = boxMac[7]
   
    ble.ah_cmd = 0x06

    -- [8 - 14]地址(7) 
    ble.ai_addr[1] = thingMac[1]
    ble.ai_addr[2] = thingMac[2]
    ble.ai_addr[3] = thingMac[3]
    ble.ai_addr[4] = thingMac[4]
    ble.ai_addr[5] = thingMac[5]
    ble.ai_addr[6] = thingMac[6]
    ble.ai_addr[7] = thingMac[7]


    -- [15-30]数据(16)
    bleData.a = 0x02
    bleData.b = 0x01
    bleData.c = 30
  
    ble.aj_data = bleData

    return tableToVec(ble)
end
--查询设备状态
function query_thing()
    local bleData = getBleData()
    local ble = getBle()
    local thingMac = stringToMac(thing_data["thing_mac"])
    local boxMac = stringToMac(configure_data:core_mac())
    ble.af_serial_num = configure_data:gen_serial_num()%255
    --盒子mac的最后一位作为群组
    ble.ag_group = boxMac[7]
   
    ble.ah_cmd = 0x02

    -- [8 - 14]地址(7) 
    ble.ai_addr[1] = thingMac[1]
    ble.ai_addr[2] = thingMac[2]
    ble.ai_addr[3] = thingMac[3]
    ble.ai_addr[4] = thingMac[4]
    ble.ai_addr[5] = thingMac[5]
    ble.ai_addr[6] = thingMac[6]
    ble.ai_addr[7] = thingMac[7]
    ble.aj_data = bleData

    return tableToVec(ble)
end
--查询设备状态
function reset_thing()
    local bleData = getBleData()
    local ble = getBle()
    local thingMac = stringToMac(thing_data["thing_mac"])
    local boxMac = stringToMac(configure_data:core_mac())
    ble.af_serial_num = configure_data:gen_serial_num()%255
    --盒子mac的最后一位作为群组
    ble.ag_group = boxMac[7]
    ble.ah_cmd = 0x11
    -- [8 - 14]地址(7) 
    ble.ai_addr[1] = thingMac[1]
    ble.ai_addr[2] = thingMac[2]
    ble.ai_addr[3] = thingMac[3]
    ble.ai_addr[4] = thingMac[4]
    ble.ai_addr[5] = thingMac[5]
    ble.ai_addr[6] = thingMac[6]
    ble.ai_addr[7] = thingMac[7]
    ble.aj_data = bleData

    return tableToVec(ble)
end
--设备控制
function control_thing()
    local thingMac = stringToMac(thing_data["thing_mac"])
    local boxMac = stringToMac(configure_data:core_mac())
    -- local boxMac = stringToMac("f5000000000001")
    local name = thing_data["thing_name"]

    if(name == "c") then--暖白控制
       return cControl(thingMac,boxMac,thing_data["ctrl_power"],thing_data["ctrl_brightness"])
    elseif(name == "cw") then--暖白控制
        return warmControl(thingMac,boxMac,thing_data["ctrl_power"],thing_data["ctrl_brightness"],thing_data["ctrl_color_temp"])
    elseif(name == "rgbcw") then --rgbcw控制
        return rgbCWControl(thingMac,boxMac,thing_data["ctrl_power"],thing_data["ctrl_brightness"],thing_data["ctrl_color_temp"],thing_data["ctrl_work_mode"],thing_data["ctrl_r"],thing_data["ctrl_g"],thing_data["ctrl_b"],thing_data["ctrl_mode"],thing_data["sctrl_peed"])
    elseif(name == "one_way_ctrl_box") then --一路开关控制
        return oneWaySwithControl(thingMac,boxMac,thing_data["ctrl_power"])
    elseif(name == "five_light_remote") then --五路开关控制
        return  fiveWayHandSetControl(thingMac,boxMac)
    else
        return nil
    end
--[[
elseif(control_type == 3) then --开合帘控制
    khcurtainControl(thingMac,boxMac,tbl)
elseif(control_type == 4) then --梦幻帘控制
    mhcurtainControl(thingMac,boxMac,tbl)
elseif(control_type == 5) then --卷帘控制
    jlcurtainControl(thingMac,boxMac,tbl)
    ]]
end

--添加场景
function set_scene() 
    local thingMac = stringToMac(thing_data["thing_mac"])
    local boxMac = stringToMac(configure_data:core_mac())
    local sceneId = stringToMac(thing_data["thing_scene_id"])
    local bleData = getBleData()
    local ble = getBle()
    ble.af_serial_num = configure_data:gen_serial_num()%255
    --盒子mac的最后一位作为群组
    ble.ag_group = boxMac[7]
    ble.ah_cmd = 0x0A
    
     -- [8 - 14]地址(7) 
     ble.ai_addr[1] = thingMac[1]
     ble.ai_addr[2] = thingMac[2]
     ble.ai_addr[3] = thingMac[3]
     ble.ai_addr[4] = thingMac[4]
     ble.ai_addr[5] = thingMac[5]
     ble.ai_addr[6] = thingMac[6]
     ble.ai_addr[7] = thingMac[7]
     bleData.a = 0x01
     bleData.b = sceneId[1]
     bleData.c = sceneId[2]
     bleData.d = sceneId[3]
     bleData.e = sceneId[4]
     bleData.f = sceneId[5]
     bleData.g = 0x07
     
     if(name == "c") then--暖白控制
         cScene(bleData)
     elseif(name == "cw") then--暖白控制
         cwScene(bleData)
     elseif(name == "rgbcw") then --rgbcw控制
         rgbCWScene(bleData)
     elseif(name == "one_way_ctrl_box") then --一路开关控制
         oneWaySwithScene(bleData)
     elseif(name == "five_light_remote") then --五路开关控制
         fiveWayHandSetScene(bleData)
     end
     ble.aj_data = bleData
     return tableToVec(ble)
end
function cScene(bleData)
    local isOn = thing_data["scene_power"]
    if (isOn == "1") then
        bleData.h = 1
        bleData.i = tonumber(thing_data["scene_brightness"])
    else
        bleData.h = 2
    end
   
end 
function cwScene(bleData)
    local isOn = thing_data["scene_power"]
    if (isOn == "1") then
        bleData.h = 1
        bleData.i = tonumber(thing_data["scene_brightness"])
        bleData.j = tonumber(thing_data["scene_color_temp"])
    else
        bleData.h = 2
    end
   
end
function rgbCWScene(bleData)

end
function oneWaySwithScene(bleData)
    
end
function fiveWayHandSetScene(bleData)
    
end
--删除场景
function remove_scene()
    local thingMac = stringToMac(thing_data["thing_mac"])
    local boxMac = stringToMac(configure_data:core_mac())
    local sceneId = stringToMac(thing_data["thing_scene_id"])
    local bleData = getBleData()
    local ble = getBle()
    ble.af_serial_num = configure_data:gen_serial_num()%255
    --盒子mac的最后一位作为群组
    ble.ag_group = boxMac[7]
    ble.ah_cmd = 0x0A
    
     -- [8 - 14]地址(7) 
     ble.ai_addr[1] = thingMac[1]
     ble.ai_addr[2] = thingMac[2]
     ble.ai_addr[3] = thingMac[3]
     ble.ai_addr[4] = thingMac[4]
     ble.ai_addr[5] = thingMac[5]
     ble.ai_addr[6] = thingMac[6]
     ble.ai_addr[7] = thingMac[7]
     bleData.a = 0x02
     bleData.b = 1
     bleData.c = sceneId[1]
     bleData.d = sceneId[2]
     bleData.e = sceneId[3]
     bleData.f = sceneId[4]
     bleData.g = sceneId[5]
     ble.aj_data = bleData

     return tableToVec(ble)
end
--执行场景
function execute_scene()
    local thingMac = stringToMac(thing_data["thing_mac"])
    local boxMac = stringToMac(configure_data:core_mac())
    local sceneId = stringToMac(thing_data["thing_scene_id"])
    local bleData = getBleData()
    local ble = getBle()
    ble.af_serial_num = configure_data:gen_serial_num()%255
    --盒子mac的最后一位作为群组
    ble.ag_group = boxMac[7]
    ble.ah_cmd = 0x0D
    
     -- [8 - 14]地址(7) 
     ble.ai_addr[1] = thingMac[1]
     ble.ai_addr[2] = thingMac[2]
     ble.ai_addr[3] = thingMac[3]
     ble.ai_addr[4] = thingMac[4]
     ble.ai_addr[5] = thingMac[5]
     ble.ai_addr[6] = thingMac[6]
     ble.ai_addr[7] = thingMac[7]
     bleData.a = 0x02
     bleData.b = 0x03 --指定网关广播
     bleData.c = sceneId[1]
     bleData.d = sceneId[2]
     bleData.e = sceneId[3]
     bleData.f = sceneId[4]
     bleData.g = sceneId[5]

     bleData.h = boxMac[2]
     bleData.i = boxMac[3]
     bleData.j = boxMac[4]
     bleData.k = boxMac[5]
     bleData.l = boxMac[6]
     bleData.m = boxMac[7]
     bleData.n = 0
     bleData.o = ble.af_serial_num

     ble.aj_data = bleData

     return tableToVec(ble)
end

--单色灯控制
function cControl(thingMac,boxMac,isOn,brightness)
    local bleData = getBleData()
    local ble = getBle()
    ble.af_serial_num = configure_data:gen_serial_num()%255
    --盒子mac的最后一位作为群组
    ble.ag_group = boxMac[7]
    ble.ah_cmd = 0x01

    -- [8 - 14]地址(7) 
    ble.ai_addr[1] = thingMac[1]
    ble.ai_addr[2] = thingMac[2]
    ble.ai_addr[3] = thingMac[3]
    ble.ai_addr[4] = thingMac[4]
    ble.ai_addr[5] = thingMac[5]
    ble.ai_addr[6] = thingMac[6]
    ble.ai_addr[7] = thingMac[7]


    -- [15-30]数据(16)
    bleData.a = 0xc1
    bleData.b = 0x01
    if isOn == "1" then
        bleData.c = 0x01
    else
        bleData.c = 0x02
        end
    bleData.d = brightness
  
    ble.aj_data = bleData

    return tableToVec(ble)
end
--暖白灯控制
function warmControl(thingMac,boxMac,isOn,brightness,color_temp)
    local bleData = getBleData()
    local ble = getBle()
    ble.af_serial_num = configure_data:gen_serial_num()%255
    --盒子mac的最后一位作为群组
    ble.ag_group = boxMac[7]
    ble.ah_cmd = 0x01

    -- [8 - 14]地址(7) 
    ble.ai_addr[1] = thingMac[1]
    ble.ai_addr[2] = thingMac[2]
    ble.ai_addr[3] = thingMac[3]
    ble.ai_addr[4] = thingMac[4]
    ble.ai_addr[5] = thingMac[5]
    ble.ai_addr[6] = thingMac[6]
    ble.ai_addr[7] = thingMac[7]


    -- [15-30]数据(16)
    bleData.a = 0xc1
    bleData.b = 0x01
    if isOn == "1" then
        bleData.c = 0x01
    else
        bleData.c = 0x02
        end
    bleData.d = tonumber(brightness)
    bleData.e = tonumber(color_temp)

    ble.aj_data = bleData

    return tableToVec(ble)
end
--RGBCW控制
function rgbCWControl(thingMac,boxMac,isOn,brightness,color_temp,workmode,r,g,b,mode,speed)
    local bleData = getBleData()
    local ble = getBle()
    ble.af_serial_num = configure_data:gen_serial_num()%255
    --盒子mac的最后一位作为群组
    ble.ag_group = boxMac[7]
    ble.ah_cmd = 0x01

    -- [8 - 14]地址(7) 
    ble.ai_addr[1] = thingMac[1]
    ble.ai_addr[2] = thingMac[2]
    ble.ai_addr[3] = thingMac[3]
    ble.ai_addr[4] = thingMac[4]
    ble.ai_addr[5] = thingMac[5]
    ble.ai_addr[6] = thingMac[6]
    ble.ai_addr[7] = thingMac[7]

    -- [15-30]数据(16)
    bleData.a = 0xc1
    bleData.b = 0x01
    if isOn == "0" then
        bleData.c = 0x02
    else
 
        if(workmode == 0x00) --冷暖工作模式
        then
            bleData.c = 0x10
        else    --RGB工作模式
            bleData.c = 0x03  
        end
    end
    bleData.d = brightness
    bleData.e = color_temp
    bleData.f = r
    bleData.g = g
    bleData.h = b
    bleData.j = mode
    bleData.k = speed

    ble.aj_data = bleData

    return tableToVec(ble)
end
--开合帘控制
function khcurtainControl(thingMac,boxMac)
    local bleData = getBleData()
    local ble = getBle()
    ble.af_serial_num = configure_data:gen_serial_num()%255
    --盒子mac的最后一位作为群组
    ble.ag_group = boxMac[7]
    ble.ah_cmd = 0x01

    -- [8 - 14]地址(7) 
    ble.ai_addr[1] = thingMac[1]
    ble.ai_addr[2] = thingMac[2]
    ble.ai_addr[3] = thingMac[3]
    ble.ai_addr[4] = thingMac[4]
    ble.ai_addr[5] = thingMac[5]
    ble.ai_addr[6] = thingMac[6]
    ble.ai_addr[7] = thingMac[7]

    -- [15-30]数据(16)
    --如果控制开关
    if(thing_data["ctrl_power"] ~= nil) then
        local isOn = tbl["ctrl_power"]
        if isOn == "1" then
           bleData.a = 0x01
           bleData.b = 0x01
        else
           bleData.a = 0x01
           bleData.b = 0x02
        end
    end

    --如果控制模式
    if(thing_data["ctrl_mode"] ~=  nil) then
        local mode = tbl["ctrl_mode"]
        bleData.a = 0x01
        bleData.b = mode
    end
  
    --如果控制位置
    if(thing_data["ctrl_position"] ~= nil) then
        local position = tbl["ctrl_position"]
        bleData.a = 0x02
        bleData.b = 0x04
        bleData.c = position
        
    end

    --如果控制方向
    if(thing_data["ctrl_direction"] ~= nil) then
        local direction = tbl["direction"]
        bleData.a = 0x02
        bleData.b = 0x09
        bleData.c = direction
       
    end
   
     --如果启动手拉
     if(thing_data["ctrl_hand_pull"] ~= nil) then
        local hand_pull = tbl["ctrl_hand_pull"]
        bleData.a = 0x02
        bleData.b = 0x0a
        bleData.c = hand_pull
       
    end
    
    ble.aj_data = bleData
    return tableToVec(ble)
end
--梦幻帘控制
function mhcurtainControl(thingMac,boxMac)
    local bleData = getBleData()
    local ble = getBle()
    ble.af_serial_num = configure_data:gen_serial_num()%255
    --盒子mac的最后一位作为群组
    ble.ag_group = boxMac[7]
    ble.ah_cmd = 0x01

    -- [8 - 14]地址(7) 
    ble.ai_addr[1] = thingMac[1]
    ble.ai_addr[2] = thingMac[2]
    ble.ai_addr[3] = thingMac[3]
    ble.ai_addr[4] = thingMac[4]
    ble.ai_addr[5] = thingMac[5]
    ble.ai_addr[6] = thingMac[6]
    ble.ai_addr[7] = thingMac[7]
    -- [15-30]数据(16)
    --如果控制开关
    if(thing_data["ctrl_power"] ~= nil) then
        local isOn = thing_data["ctrl_power"]
        if isOn == "1" then
           bleData.a = 0x01
           bleData.b = 0x01
        else
           bleData.a = 0x01
           bleData.b = 0x02
        end
    end


    --如果控制位置
    if(thing_data["ctrl_position"] ~= nil) then
        bleData.a = 0x02
        bleData.b = 0x04
        bleData.c = position
       
    end
     --如果控制角度
   
    if(thing_data["ctrl_angel"] ~= nil) then
        bleData.a = 0x02
     bleData.b = 0x17
     bleData.c = angel
       
    end
    --如果控制位置角度
    if(thing_data["ctrl_position"] ~= nil and thing_data["ctrl_angel"] ~= nil) then
        bleData.a = 0x03
        bleData.b = 0x19
        bleData.c = position
        bleData.d = angel   
    end
   
     --如果控制模式
     if(thing_data["ctrl_mode"] ~= nil) then
        bleData.a = 0x01
        bleData.b = mode
    end
  
    --如果控制方向
    if(thing_data["ctrl_direction"] ~= nil) then
        bleData.a = 0x02
        bleData.b = 0x09
        bleData.c = direction  
    end
   
    ble.aj_data = bleData
    return tableToVec(ble)
end
--卷帘控制
function jlcurtainControl(thingMac,boxMac)
    local bleData = getBleData()
    local ble = getBle()
    ble.af_serial_num = configure_data:gen_serial_num()%255
    --盒子mac的最后一位作为群组
    ble.ag_group = boxMac[7]
    ble.ah_cmd = 0x01

    -- [8 - 14]地址(7) 
    ble.ai_addr[1] = thingMac[1]
    ble.ai_addr[2] = thingMac[2]
    ble.ai_addr[3] = thingMac[3]
    ble.ai_addr[4] = thingMac[4]
    ble.ai_addr[5] = thingMac[5]
    ble.ai_addr[6] = thingMac[6]
    ble.ai_addr[7] = thingMac[7]

    -- [15-30]数据(16)
    --如果控制开关
    if(thing_data["ctrl_power"] ~= nil) then
        local isOn = tbl["ctrl_power"]
        if isOn == "1" then
           bleData.a = 0x01
           bleData.b = 0x01
        else
           bleData.a = 0x01
           bleData.b = 0x02
        end
    end
     --如果控制模式
     if(thing_data["ctrl_mode"] ~= nil) then
        bleData.a = 0x01
        bleData.b = mode
    end

    --如果控制位置
    if(thing_data["ctrl_position"] ~= nil) then
        bleData.a = 0x02
        bleData.b = 0x04
        bleData.c = position
       
    end

    --如果控制上行程
    if(thing_data["ctrl_limit_top"] ~= nil) then
        bleData.a = 0x02
        bleData.b = 0x05
        bleData.c = limit_top
       
    end
   

     --如果控制下行程
    if(thing_data["ctrl_limit_bottom"] ~= nil) then
        bleData.a = 0x02
        bleData.b = 0x06
        bleData.c = limit_bottom
       
    end
   

      --如果重置行程
    if(thing_data["ctrl_reset"] ~= nil) then
        bleData.a = 0x01
        bleData.b = 0x07
       
    end
   
    ble.aj_data = bleData
    return tableToVec(ble)
end

--一路控制盒控制
function oneWaySwithControl(thingMac,boxMac,isOn)
    local bleData = getBleData()
    local ble = getBle()
    ble.af_serial_num = configure_data:gen_serial_num()%255
    --盒子mac的最后一位作为群组
    ble.ag_group = boxMac[6]
    ble.ah_cmd = 0x01

    -- [8 - 14]地址(7) 
    ble.ai_addr[1] = thingMac[1]
    ble.ai_addr[2] = thingMac[2]
    ble.ai_addr[3] = thingMac[3]
    ble.ai_addr[4] = thingMac[4]
    ble.ai_addr[5] = thingMac[5]
    ble.ai_addr[6] = thingMac[6]
    ble.ai_addr[7] = thingMac[7]

    -- [15-30]数据(16)

    bleData.a = 0x02
    bleData.b = 0x01
    bleData.c = 0x80
    if isOn == "1" then
        bleData.d = 0x80
    else
        bleData.d = 0x00
    end

    ble.aj_data = bleData
    return tableToVec(ble)
end

--五路遥控器控制
function fiveWayHandSetControl(thingMac,boxMac,tbl)
    local bleData = getBleData()
    local ble = getBle()
    ble.af_serial_num = configure_data:gen_serial_num()%255
    --盒子mac的最后一位作为群组
    ble.ag_group = boxMac[5]
    ble.ah_cmd = 0x01

    -- [8 - 14]地址(7) 
    ble.ai_addr[1] = thingMac[1]
    ble.ai_addr[2] = thingMac[2]
    ble.ai_addr[3] = thingMac[3]
    ble.ai_addr[4] = thingMac[4]
    ble.ai_addr[5] = thingMac[5]
    ble.ai_addr[6] = thingMac[6]
    ble.ai_addr[7] = thingMac[7]

    -- [15-30]数据(16)
    --[[
    bleData.a = 0xc1
    if(tbl["brightness"] != nil ) then
        bleData.b = 0x00
        bleData.c = limit_bottom
       
    end
    bleData.a = 0x02
    bleData.b = 0x01
    bleData.c = 0x80
    if isOn then
        bleData.d = 0x80
    else
        bleData.d = 0x00
    end
    
    ble.aj_data = bleData
    -]]
    return tableToVec(ble)
end

--解析函数
function alalize_data(datas)
    
    --datas = {0x1E,0xFF,0xB2,0x51,0xA0,0x01,0xFF,0x34,0xF5,0x11,0x11,0x11,0x11,0x78,0x0E,0x20,0x22,0x00,0x0D,0x00,0x01,0xF5,0xF5,0x01,0x00,0xF5,0xF5}
    if (datas[8] == 0x34)--搜索设备返回
     then 
        output_data:set("cmd","search_thing")
       --mac
       local mac = hex2str({datas[9],datas[10],datas[11],datas[12],datas[13],datas[14],datas[15]})
       output_data:set("thing_mac",mac)
        --厂家码
        local manufactorData = {datas[16],datas[17]}
        local manufactor = hex2str(manufactorData)
        
        output_data:set("manufactor",manufactor)
         --设备码
        local thingIdData = {datas[18],datas[19]}
        local thingId = hex2str(thingIdData)
    
        output_data:set("thing_id",thingId)
        --设备子类型
        local thingTypeData = {datas[20],datas[21]}
        local thingType = hex2str(thingTypeData)
        output_data:set("thing_type",thingType)
          --随机秘钥
          
        local randomKeyData = {datas[22],datas[23]}
        local randomKey = hex2str(randomKeyData)
        output_data:set("random_key",randomKey)
        --程序版本
        local programVersionData = {datas[24],datas[25]}
        local programVersion = hex2str(programVersionData)
        output_data:set("program_version",programVersion)
     
        local name = ""
        if(manufactor == "2022") then
            if(thingId == "000B" and thingType == "0001") then
              name = "rgbcw"
            elseif(thingId == "0004" and thingType == "0001") then
              name = "c"
            elseif(thingId == "000D" and thingType == "0001") then
              name = "cw"
            elseif(thingId == "0002" and thingType == "0001") then
              name = "one_way_ctrl_box"
            elseif(thingId == "0002" and thingType == "0002") then
              name = "two_way_ctrl_box"
            elseif(thingId == "0002" and thingType == "0003") then
              name = "three_way_ctrl_box"
            elseif(thingId == "0002" and thingType == "0004") then
              name ="four_way_ctrl_box"
            elseif(thingId == "0029" and thingType == "0002") then
              name = "four_scene_panel"
            elseif(thingId == "0029" and thingType == "0005") then
              name = "six_scene_panel"
            elseif(thingId == "0029" and thingType == "0006") then
              name = "eight_scene_panel"
            elseif(thingId == "0029" and thingType == "0007") then
              name = "two_scene_panel"
            elseif(thingId == "00C1" and thingType == "2711") then
              name = "one_power"
            elseif(thingId == "00C1" and thingType == "2712") then
              name = "two_power"
            elseif(thingId == "00C1" and thingType == "2713") then
              name = "three_power"
            elseif(thingId == "00C1" and thingType == "2714") then
              name = "four_power"
            elseif(thingId == "00C1" and thingType == "271E") then
              name = "five_light_remote"
            end
        end
        output_data:set("thing_name",name)
         return true
      
    elseif (datas[8] == 0x30)  then
         --mac
         local mac = hex2str({datas[9],datas[10],datas[11],datas[12],datas[13],datas[14],datas[15]})
         local name = input_data["thing_name"]
         local statusData = {datas[16],datas[17],datas[18],datas[19],datas[20],datas[21],datas[22],datas[23],datas[24],datas[25],datas[26],datas[27],datas[28],datas[29],datas[30],datas[31]}
        --读取设备状态此地放能不能获取到thingname判断出设备类型，根据类型去解析协议
        if(name == "c") then--暖白控制
            if (statusData[3] == 0x01 or statusData[3] == 0x60) then
                output_data:set("thing_power_status","1")
            elseif (statusData[3] == 0x02 or statusData[3] == 0x61) then
                output_data:set("thing_power_status","0")
            end
            
            if (statusData[3] == 0x01 or statusData[3] == 0x02) then
                output_data:set("thing_brightness_status",statusData[4].."")
            elseif (devdata[3] == 0x04 or devdata[3] == 0x05) then
                output_data:set("thing_brightness_status",statusData[4].."")
            end
         elseif(name == "cw") then--暖白控制
            if (statusData[3] == 0x01 or statusData[3] == 0x60) then
                output_data:set("thing_power_status","1")
            elseif (statusData[3] == 0x02 or statusData[3] == 0x61) then
                output_data:set("thing_power_status","0")
            end
            if (devdata[3] == 0x01 or devdata[3] == 0x02) then
                output_data:set("thing_brightness_status",statusData[4].."")
                output_data:set("thing_color_temp_status",statusData[5].."")
            elseif (devdata[3] == 0x04 or devdata[3] == 0x05) then
                output_data:set("thing_brightness_status",statusData[4].."")
            elseif (devdata[3] == 0x06 or devdata[3] == 0x07) then
                output_data:set("thing_color_temp_status",statusData[5].."")
            end
         elseif(name == "rgbcw") then --rgbcw控制
            if (devdata[3] == 0x60) then
                output_data:set("thing_power_status","1")
            elseif (devdata[3] == 0x61) then
                output_data:set("thing_power_status","0")
            elseif (devdata[3] == 0x01) then
                output_data:set("thing_power_status","1")
                output_data:set("thing_work_mode_status","0")
            elseif (devdata[3] == 0x03) then
                output_data:set("thing_power_status","1")
                output_data:set("thing_work_mode_status","1")
            elseif (devdata[3] == 0x09) then
                output_data:set("thing_power_status","1")
                output_data:set("thing_work_mode_status","0")
            elseif (devdata[3] == 0x02) then
                output_data:set("thing_power_status","0")
            elseif (devdata[3] == 0x10) then
                output_data:set("thing_mode_status",statusData[9].."")
                output_data:set("thing_speed_status",statusData[10].."")
            end
            output_data:set("thing_brightness_status",statusData[4].."")
            output_data:set("thing_color_temp_status",statusData[5].."")
            output_data:set("thing_r_status",statusData[6].."")
            output_data:set("thing_g_status",statusData[7].."")
            output_data:set("thing_b_status",statusData[8].."")
         elseif(name == "one_way_ctrl_box") then --一路开关控制
             
         elseif(name == "five_light_remote") then --五路开关控制
             
         else
         end
      
       
        return true
    end
    return false
end

---将16进制串转换为字符串
function hex2str(hex)
	--拼接字符串
	local index=1
	local ret=""
	for index=1,#hex do
		ret=ret..string.format("%02X",hex[index])
	end
 
	return ret
end
-- --将字符串按格式转为16进制串
-- function str2hex(str)
-- 	--判断输入类型	
-- 	if (type(str)~="string") then
-- 	    return nil,"str2hex invalid input type"
-- 	end
	
-- 	--拼接字符串
-- 	local index=1
-- 	local ret=""
-- 	for index=1,str:len(),2 do
-- 	    ret=ret..string.char(tonumber(str:sub(index,index+1),16))
-- 	end
 
-- 	return ret
-- end
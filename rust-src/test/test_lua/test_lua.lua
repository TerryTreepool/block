

function get_it()
    return userdata:get_value()
end

function set_it(i)
    return userdata:set_value(i)
end

function get_constant()
    return userdata.get_constant()
end

function belData() 
    local bleData = {
        a = 0,
        b = 0,
        c = 0,
        d = 0,
        e = 0,
        f = 0,
        g = 0,
        h = 0,
        i = 0,
        j = 0,
        k = 0,
        l = 0,
        m = 0,
        n = 0,
        o = 0,
        p = 0,
    }

    return bleData
end

function search(data)
    local project_path = configure_data:get_project_path()
    print(project_path)

    package.path = package.path..";"..project_path.."/?.lua"
    print(package.path)

    local pub = require("pub")

    -- local bleData = {
    --     a = 0,
    --     b = 0,
    --     c = 0,
    --     d = 0,
    --     e = 0,
    --     f = 0,
    --     g = 0,
    --     h = 0,
    --     i = 0,
    --     j = 0,
    --     k = 0,
    --     l = 0,
    --     m = 0,
    --     n = 0,
    --     o = 0,
    --     p = 0,
    -- }
    local bleData = belData()

    local ble =
    {
        aa_len = 0x1e,
        ab_type = 0xff,
        ac_pack_mark = "",
        ad_pack_order = 0,
        ae_pack_relay = 0,
        af_serial_num = 0,
        ag_group = 0,
        ah_cmd = 0,
        ai_addr = {},
        aj_data,
    }

    ble.aa_len = 0x1e
    ble.ab_type = 0xff
    ble.ac_pack_mark = 0x53
    ble.ad_pack_order = 0x55
    ble.ae_pack_relay = 0x05
    ble.af_serial_num = configure_data:gen_serial_num()%255
    ble.ag_group = 0x00
    ble.ah_cmd = 0x03

    -- [8 - 14]地址(7) 
    ble.ai_addr[0] = 0xf5
    ble.ai_addr[1] = 0xf0
    ble.ai_addr[2] = 0xf0
    ble.ai_addr[3] = 0xf0
    ble.ai_addr[4] = 0xf0
    ble.ai_addr[5] = 0xf0
    ble.ai_addr[6] = 0xf0

    -- [15-30]数据(16)
    bleData.a = 0xff

    ble.aj_data = bleData

    return tableToVec(ble)
end

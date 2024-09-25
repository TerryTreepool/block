
use near_base::{DeviceObjectSubCode, ObjectTypeCode, };

// 扩展索引分界
const HARDENED_OFFSET: u32 = 0x80000000;

// BIP0044指定了包含5个预定义树状层级的结构： m / purpose' / coin' / account' / change / address_index

// 1. purpose和btc/etc采用统一值 44(或者 0x8000002C)，表示bip44， bitcoin的bit49定义了隔离见证，所以隔离见证地址使用了49值
// 如果需要为bfc定义新的bip规范，那么需要修改此值,目前暂定BIP1314
// 规范列表见 https://wiki.trezor.io/Cryptocurrency_standards
pub const NEAR_BIP: u32 = 1314;

// 2. Coin type 这个代表的是币种，0代表比特币，1代表比特币测试链，60代表以太坊
// 币种列表见 https://github.com/satoshilabs/slips/blob/master/slip-0044.md
// 我们在这里用来表示object的obj_type, 目前只有people和device两种object存在私钥
// pub const NEAR_CHAIN: u32 = 0x80201608;

// 3. account对应了people实体的索引，从0开始，对于属于people下的所有二级密钥，account采用此索引

// 4. change用以指定网络类型，0为正式网，1为测试网，可扩展

// 5. address_index 地址索引，从0开始

#[derive(Clone)]
pub enum ChainObjectType {
    Device(ObjectTypeCode),
    People(ObjectTypeCode),
}

impl ChainObjectType {
    pub fn to_u16(&self) -> u16 {
        match self {
            Self::People(v) => v.into_u16(),
            Self::Device(v) => v.into_u16(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum ChainNetwork {
    Main = 0,
    Test = 1,
    Beta = 2,
}

impl std::default::Default for ChainNetwork {
    fn default() -> Self {
        #[cfg(debug_assertions)]
        {
            Self::Test
        }
        #[cfg(not(debug_assertions))]
        {
            Self::Main
        }
    }
}

impl Into<u32> for ChainNetwork {
    fn into(self) -> u32 {
        unsafe { std::mem::transmute(self as u32) }
    }
}


#[derive(Clone)]
pub struct ChainBipPath {
    purpose: u32,
    coin: ChainObjectType,
    account: u32,
    network: ChainNetwork,
    address_index: u32,
}

impl ChainBipPath {
    // 由于people目前默认没owner，所以account为0
    // 如果创建多个people，那么address_index从0开始增加
    //  m / 809' / coin' / 0' / 0 / 0|2|3...
    pub fn new_people(
        network: ChainNetwork, 
        address_index: Option<u32>
    ) -> Self {
        let address_index = address_index.unwrap_or(0);
        assert!(address_index < HARDENED_OFFSET);

        Self {
            purpose: NEAR_BIP,
            coin: ChainObjectType::People(ObjectTypeCode::People),

            account: 0,
            network,
            address_index,
        }
    }

    // 创建一个device，account指定了owner(一般是people)的索引，多个device，address_index从0开始增加
    pub fn new_device(
        network: ChainNetwork,
        account: u32,
        address_index: Option<u32>,
    ) -> Self {
        let address_index = address_index.unwrap_or(0);
        assert!(address_index < HARDENED_OFFSET);

        Self {
            purpose: NEAR_BIP,
            coin: ChainObjectType::Device(ObjectTypeCode::Device(DeviceObjectSubCode::OBJECT_TYPE_DEVICE_CORE as u8)),

            account,
            network,
            address_index,
        }
    }
}

impl ToString for ChainBipPath {
    fn to_string(&self) -> String {
        assert!(self.address_index < HARDENED_OFFSET);

        let network: u32 = self.network.clone().into();

        format!(
            "m/{}'/{}'/{}'/{}/{}",
            NEAR_BIP,
            self.coin.to_u16(),
            self.account,
            network,
            self.address_index
        )
    }
}

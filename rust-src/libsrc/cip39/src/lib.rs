
pub mod profile;

#[allow(unused)]
mod cip39;
#[allow(unused)]
mod cip39_p;
#[allow(unused)]
mod seed_key_bip;
#[allow(unused)]
mod seed;
#[allow(unused)]
mod path;

#[derive(Clone, Copy)]
pub enum CipPrivateKey {
    Rsa1024,
    Rsa2048,
}

impl std::fmt::Display for CipPrivateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rsa1024 => write!(f, "ras1024"),
            Self::Rsa2048 => write!(f, "ras2048"),
        }
    }
}

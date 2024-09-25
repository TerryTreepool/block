
pub mod thing_components;

#[derive(Clone)]
pub enum ThingStatus {
    Offline(near_base::Timestamp, crate::lua::data::Data),
    Online(near_base::Timestamp, crate::lua::data::Data),
    Disable,
}

impl std::fmt::Display for ThingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Offline(_, _) => write!(f, "offline"),
            Self::Online(_, _) => write!(f, "online"),
            Self::Disable => write!(f, "disable"),
        }
    }
}



mod manager;
mod process;

pub use manager::NdsManager;
pub use process::*;

pub struct NdsFileArticle {
    pub file_id: String,
    pub file_path: std::path::PathBuf,
}

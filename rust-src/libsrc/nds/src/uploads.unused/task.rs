
use crate::files::{FileArticlePtr};

struct FileTaskImpl {
    file_article: FileArticlePtr,
}

impl FileTaskImpl {
    pub fn new(file_article: FileArticlePtr) -> Self {
        Self {
            file_article 
        }
    }
}

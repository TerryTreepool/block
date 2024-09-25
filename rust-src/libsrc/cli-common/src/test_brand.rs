
pub struct TestBrand {
    pub brand_id: ::std::string::String,
    pub brand_name: ::std::string::String,
    pub begin_time: ::std::string::String,
    pub update_time: ::std::string::String,
    pub status: u32,

}

impl TestBrand {
    #[inline]
    pub fn brand_id(&self) -> String {
        self.brand_id.clone()
    }

    #[inline]
    pub fn brand_name(&self) -> String {
        self.brand_name.clone()
    }

    #[inline]
    pub fn begin_time(&self) -> String {
        self.begin_time.clone()
    }

    #[inline]
    pub fn update_time(&self) -> String {
        self.update_time.clone()
    }

    #[inline]
    pub fn status(&self) -> u32 {
        self.status
    }
}

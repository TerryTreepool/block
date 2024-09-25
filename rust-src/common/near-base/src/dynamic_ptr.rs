
use std::sync::Arc;

#[derive(Clone)]
pub struct DynamicPtr<T: PartialEq + PartialOrd + Ord + Eq>(Arc<T>);

impl<T: PartialEq + PartialOrd + Ord + Eq> DynamicPtr<T> {
    pub fn new(ptr: T) -> Self {
        Self(Arc::new(ptr))
    }
}

impl<T: PartialEq + PartialOrd + Ord + Eq> DynamicPtr<T> {
    fn as_ref(&self) -> &T {
        self.0.as_ref()
    }
}

impl<T: PartialEq + PartialOrd + Ord + Eq> std::cmp::PartialOrd<T> for DynamicPtr<T> {
    fn partial_cmp(&self, other: &T) -> Option<std::cmp::Ordering> {
        self.as_ref().partial_cmp(other)
    }
}

impl<T: PartialEq + PartialOrd + Ord + Eq> std::cmp::PartialEq<T> for DynamicPtr<T> {
    fn eq(&self, other: &T) -> bool {
        self.as_ref().eq(other)
    }
}

impl<T: PartialEq + PartialOrd + Ord + Eq> std::cmp::PartialOrd<DynamicPtr<T>> for DynamicPtr<T> {
    fn partial_cmp(&self, other: &DynamicPtr<T>) -> Option<std::cmp::Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl<T: PartialEq + PartialOrd + Ord + Eq> std::cmp::PartialEq<DynamicPtr<T>> for DynamicPtr<T> {
    fn eq(&self, other: &DynamicPtr<T>) -> bool {
        self.as_ref().eq(other.as_ref())
    }
}

impl<T: PartialEq + PartialOrd + Ord + Eq> std::cmp::Ord for DynamicPtr<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl<T: PartialEq + PartialOrd + Ord + Eq> std::cmp::Eq for DynamicPtr<T> { }

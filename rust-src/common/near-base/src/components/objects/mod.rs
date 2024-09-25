
mod object_impl;
mod object_type;
mod object_builder;

pub mod device;
pub mod thing;
pub mod extention;
pub mod people;
pub mod file;
pub mod proof_of_data;
pub mod any;

pub use object_type::{ObjectId, ObjectTypeCode, *};
pub use object_builder::{ObjectDescTrait, ObjectDescBuilder,
                         ObjectBodyTrait, ObjectBodyBuilder,
                         ObjectBuilder};
pub use object_impl::{NamedObject, NamedObjectDesc, NamedObjectBody};
pub use device::{DeviceDesc, DeviceBody, DeviceObject};
pub use extention::{ExtentionDesc, ExtentionBody, ExtentionObject};

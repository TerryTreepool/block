
mod topic;
mod types;
mod build;

pub use build::{Builder as TopicBuilder, };

pub use topic::{Topic, TopicRef};
pub use types::*;


// #[macro_export]
// macro_rules! topic_build {
//     [$name1: expr] => {
//         $crate::topic::build::__topci_build_private__!($name1)
//     };
//     [$name1: expr, $name2: expr] => {
//         $crate::topic::build::__topci_build_private__!(topic_build!($name1), $name2)
//     };
//     [$name1: expr, $($name2: expr), +] => {{
//         $crate::topic::build::__topci_build_private__!($crate::topic::build::__topci_build_private__![$name1], $crate::topic::build::__topci_build_private__![$($name2), +])
//     }};
// }


// macro_rules! __topci_build_private__ {
//     ($name1: expr) => {
//         $crate::topic::build::Builder::from($name1).build()
//     };
//     ($name1: expr, $name2: expr) => {
//         $crate::topic::build::Builder::from(($name1, $name2)).build()
//     };    
//     ($name1: expr, $($name2: expr), +) => {{
//         $crate::topic::build::__topci_build_private__!($name1, $crate::topic::build::__topci_build_private__!($($name2), +))
//     }};
// }

// mod test {
//     #[test]
//     fn test() {
//         println!("{}", topic_build!["aa"]);
//         println!("{}", topic_build!["aa", "bb"]);
//         println!("{}", topic_build!["aa", "bb", "CC"]);
//         println!("{}", topic_build!["aa", "bb", "CC", "DD"]);
//         println!("{}", topic_build!["aa", "bb", "CC", "DD", "ee"]);
//         println!("{}", topic_build!["aa", "bb", "CC", "DD", "ee", "1", "2", "3"]);
//     }
// }

// lazy_static::lazy_static! {
//     pub static ref TOPIC_SUBSCRIBE: String = topic_build![PRIMARY_TOPIC_CORE_LABEL, SECONDARY_TOPIC_SUBSCRIBE_LABEL];
// }

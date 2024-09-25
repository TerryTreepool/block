
use near_base::{NearError, ErrorCode, NearResult, };

use super::types::TOPIC_SEPARATOR;

pub(super) struct TopicD<'a> {
    pub primary_label: &'a str,
    pub secondary_label: Option<&'a str>,
    pub thirdary_label: Option<Vec<&'a str>>,
}

impl Clone for TopicD<'_> {
    fn clone(&self) -> Self {
        Self {
            primary_label: self.primary_label,
            secondary_label: self.secondary_label,
            thirdary_label: if let Some(labels) = &self.thirdary_label {
                let mut v = vec![];
                labels.iter().for_each(| &str | { v.push(str); });
                Some(v)
            } else {
                None
            },
        }
    }
}

pub struct TopicRef<'a> {
    topic: &'a Topic,
    topid_d: TopicD<'a>,
}

impl<'a> TryFrom<&'a Topic> for TopicRef<'a> {
    type Error = NearError;

    fn try_from(topic: &'a Topic) -> Result<Self, Self::Error> {
        let value = topic.topic().as_str();

        if value.is_empty() {
            return Err(NearError::new(ErrorCode::NEAR_ERROR_FATAL, "Topic's empty"));
        }

        let value = match value.split_at(1) {
            ("/", remain_str) => {
                remain_str
            }
            _ => {
                return Err(NearError::new(ErrorCode::NEAR_ERROR_TOPIC_ROOT, "Topic's format must ([/topic1] or [/topic1/topic2])"));
            }
        };

        let (primary, secondary, thirdary) = {
            let array: Vec<&str> = value.split(|c| c == TOPIC_SEPARATOR).filter(|array| !array.is_empty()).collect();

            match array.len() {
                0 => { return Err(NearError::new(ErrorCode::NEAR_ERROR_TOPIC_PRIMARY, "Cloud not found primary topic.")); }
                1 => {
                    (
                        array.get(0).map(|&data| data).unwrap(), 
                        None, 
                        None
                    )
                }
                2 => {
                    (
                        array.get(0).map(|&data| data).unwrap(), 
                        array.get(1).map(|&data| data), 
                        None
                    )
                }
                _ => {
                    (
                        array.get(0).map(|&data| data).unwrap(), 
                        array.get(1).map(|&data| data), 
                        array.get(2..).map(|array| array.to_vec())
                    )
                }
            }
        };

        Ok(TopicRef{
            topic,
            topid_d: TopicD {
                primary_label: primary,
                secondary_label: secondary,
                thirdary_label: thirdary
            },
        })

    }
}

impl std::fmt::Display for TopicRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.topic.fmt(f)
    }
}

impl TopicRef<'_> {
    pub fn topic(&self) -> &Topic {
        self.topic
    }

    pub fn primary(&self) -> &str {
        self.topid_d.primary_label
    }

    pub fn secondary(&self) -> &Option<&str> {
        &self.topid_d.secondary_label
    }

    pub(super) fn topic_d(&self) -> &TopicD {
        &self.topid_d
    }

}

// impl TopicRef<'_> {
//     fn build(p: &str, s: Option<&str>) -> Topic {
//         Topic::from(
//             if let Some(s) = s {
//                 format!("{TOPIC_SEPARATOR}{}{TOPIC_SEPARATOR}{}", p, s)
//             } else {
//                 format!("{TOPIC_SEPARATOR}{}", p)
//             }
//         )
//     }
// }

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, )]
pub struct Topic {
    topic_lable: String,
}

unsafe impl Send for Topic {}
unsafe impl Sync for Topic {}

impl PartialEq<Topic> for &Topic {
    fn eq(&self, other: &Topic) -> bool {
        self.topic_lable == other.topic_lable
    }
}

impl Topic {

    pub fn topic(&self) -> &String {
        &self.topic_lable
    }

    pub fn topic_d(&self) -> NearResult<TopicRef> {
        TopicRef::try_from(self)
    }

}

impl From<String> for Topic {
    fn from(topic: String) -> Self {
        Self{
            topic_lable: topic
        }
    }
}

// impl From<(&str, &str)> for Topic {
//     fn from(cx: (&str, &str)) -> Self {
//         let (p, s) = cx;

//         TopicRef::build(p, Some(s))
//     }
// }

impl Into<String> for Topic {
    fn into(self) -> String {
        self.topic_lable
    }
}

impl std::fmt::Display for Topic {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "topic: {}", self.topic_lable)
    }
}

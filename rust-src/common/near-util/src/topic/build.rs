
use crate::TopicRef;

use super::{types::TOPIC_SEPARATOR, topic::TopicD, Topic, };

pub struct Builder<'a> {
    topic_d: TopicD<'a>,
}

impl<'a> Builder<'a> {
    pub fn new(primary: &'a str) -> Self {
        Self {
            topic_d: TopicD {
                primary_label: primary, 
                secondary_label: None, 
                thirdary_label: None 
            },
        }
    }

    pub fn secondary(mut self, secondary: &'a str) -> Self {
        self.topic_d.secondary_label = Some(secondary);
        self
    }

    pub fn add_thirdary(mut self, thirdary: &'a str) -> Self {
        if let Some(array) = &mut self.topic_d.thirdary_label {
            array.push(thirdary);
        } else {
            self.topic_d.thirdary_label = Some(vec![thirdary]);
        }
        self
    }
}

impl<'a> From<&'a TopicRef<'_>> for Builder<'a> {
    fn from(topic_ref: &'a TopicRef) -> Self {
        Self { topic_d: topic_ref.topic_d().clone() }
    }
}

impl Builder<'_> {
    pub fn build(self) -> Topic {
        let mut topic_string = format!("{TOPIC_SEPARATOR}{}", self.topic_d.primary_label);

        if let Some(secondary) = self.topic_d.secondary_label {
            topic_string.push(TOPIC_SEPARATOR);
            topic_string.push_str(secondary);
        } else {
            return Topic::from(topic_string);
        }

        if let Some(thirdary) = self.topic_d.thirdary_label {
            thirdary.iter().for_each(| &item | {
                topic_string.push(TOPIC_SEPARATOR);
                topic_string.push_str(item);
            });
        } else {
            return Topic::from(topic_string);
        }

        return Topic::from(topic_string);
    }
}

#[test]
fn test_topic_builder() {
    let t = Builder::new("test1")
                .secondary("test2")
                .add_thirdary("test3")
                .add_thirdary("test4")
                .add_thirdary("test5")
                .build();

    println!("{}", t);
}

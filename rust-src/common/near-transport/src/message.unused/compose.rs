
use std::{sync::{Arc, RwLock}, collections::BTreeMap};

use near_base::{NearResult, SequenceValue};

use crate::{network::DynamicInterface, package::{DynamicPackage, DynamicPackageGuard}};

struct Message {
    message_existed: u8,
    message_count: u8,
    message: Vec<Option<DynamicPackageGuard>>,
}

enum MessageResult {
    Finished(Vec<DynamicPackageGuard>),
    Unfinshed,
}

impl Message {
    fn new(message: DynamicPackageGuard) -> Self {
        let head = message.as_head();
        debug_assert!(head.index() < head.count(), "fatal message");

        let mut ret = Self {
            message_existed: 1,
            message_count: head.count(),
            message: vec![None; head.count() as usize],
        };

        let _ = ret.join(message);

        ret
    }

    pub(self) fn join(&mut self, message: DynamicPackageGuard) {
        let head = message.as_head();

        let item = self.message.get_mut(head.index() as usize).unwrap();
        match item {
            Some(_) => { }
            None => {
                *item = Some(message);
                self.message_existed += 1;
            }
        }

    }

    fn push(&mut self, message: DynamicPackageGuard) -> MessageResult {
        let head = message.as_head();

        debug_assert!(self.message_existed < self.message_count, "message full.");
        debug_assert!(head.index() < head.count(), "fatal message");

        self.join(message);

        if self.message_existed == self.message_count {
            MessageResult::Finished({
                let mut array = vec![];
                self.message.iter().for_each(| item | array.push(item.unwrap()));
                array
            })
        } else {
            MessageResult::Unfinshed
        }
    }
}

struct ComposeImp {
    interface: DynamicInterface,
    message_center: RwLock<BTreeMap<SequenceValue, Message>>,
}

#[derive(Clone)]
pub struct Compose(Arc<ComposeImp>);

impl Compose {
    pub fn new(interface: DynamicInterface) -> Self {
        Self (Arc::new(ComposeImp {
            interface
        }))
    }
}

impl Compose {

    pub(super) fn on_package(&self, package: DynamicPackage) -> NearResult<()> {
        Ok(())
    }

}

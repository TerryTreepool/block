
use std::{any::Any, sync::Arc};

const TIRE_CHILD_MAX: usize = 26;

type Value = Box<dyn Any + Send + Sync>;
// struct Value<V: Send + Sync> {
//     v: V
// }

// enum TireType {
//     Separator,
//     Children(ChildTire),
//     Data(DataTire),
// }

trait NodeTrait {
    fn id(&self) -> u8;
    fn data(&self) -> Option<&Value>;
    fn insert(&self, key: &[u8], value: Value);
    // fn children(&self) -> &dyn NodeTrait;
}

struct SeparatorTire {
    children: ChildTire,
}

impl std::default::Default for SeparatorTire {
    fn default() -> Self {
        Self {
            children: ChildTire::default(),
        }
    }
}

impl NodeTrait for SeparatorTire {
    fn id(&self) -> u8 { b'/' }
    fn data(&self) -> Option<&Value> { None }
    fn insert(&self, key: &[u8], value: Value) {
        if key.len() == 0 {
            // TODO: insert value
            return;
        }

        if self.id() != key[0] {
            return;
        }

        self.children.insert(&key[1..], value)
    }
    // fn children(&self) -> &dyn NodeTrait { &self.children as &dyn NodeTrait }
}

impl SeparatorTire {
    pub fn insert(&self, key: &[u8], value: Value) {
        if key.len() == 0 {
            // set value into this
        }

        if self.id() != key[0] {
            return;
        }

        // self.children.in
    }
}

struct DataTire {
    value: Value,
}

struct ChildTire{
    my: TireNode,
    children: Option<[Box<dyn NodeTrait>; TIRE_CHILD_MAX]>,
}

impl std::default::Default for ChildTire {
    fn default() -> Self {
        Self {
            my: TireNode::default(),
            children: None,
        }
    }
}

struct TireNode {
    c: u8,
    p: Option<Value>,
}

impl std::default::Default for TireNode {
    fn default() -> Self {
        Self {
            c: 0,
            p: None,
        }
    }
}

impl NodeTrait for ChildTire {
    fn id(&self) -> u8 { self.my.c }
    fn data(&self) -> Option<&Value> { self.my.p.as_ref() }
    fn insert(&self, key: &[u8], value: Value) {
        let c = &key[0];
        if self.my.c != key[0] {
            return;
        }

        let n = &key[1];
        if self.children.is_none() {
            let children = vec![]
            self.children = Some([Box<dyn NodeTrait>; TIRE_CHILD_MAX])
        }
    }
}

struct TireImpl {
    root: SeparatorTire,
        // tires: [TireValue; TIRE_CHILD_MAX],
}

pub struct Tire(Arc<TireImpl>);

impl Tire {
    pub fn new() -> Self {
        Self(Arc::new(TireImpl{
            root: SeparatorTire::default(),
        }))
    }

    pub fn insert(&self, key: &[u8], value: Value) {
        root.insert(key, value)
    }
}

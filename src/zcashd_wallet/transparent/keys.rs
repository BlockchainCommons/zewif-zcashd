use std::collections::HashMap;

use super::{Key, PubKey};

#[derive(Clone, PartialEq)]
pub struct Keys(HashMap<PubKey, Key>);

impl Keys {
    pub fn new(map: HashMap<PubKey, Key>) -> Self {
        Self(map)
    }

    pub fn keypairs(&self) -> impl Iterator<Item = &Key> {
        self.0.values()
    }
}

impl std::fmt::Debug for Keys {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut a = f.debug_list();
        for keypair in self.keypairs() {
            a.entry(keypair);
        }
        a.finish()
    }
}

use std::collections::HashMap;

use super::{KeyPair, PubKey};

#[derive(Clone, PartialEq)]
pub struct Keys(HashMap<PubKey, KeyPair>);

impl Keys {
    pub fn new(map: HashMap<PubKey, KeyPair>) -> Self {
        Self(map)
    }

    pub fn keypairs(&self) -> impl Iterator<Item = &KeyPair> {
        self.0.values()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn keypair_for_pubkey(&self, pubkey: &PubKey) -> Option<&KeyPair> {
        self.0.get(pubkey)
    }
}

impl std::fmt::Debug for Keys {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut a = f.debug_list();
        for keypair in self.keypairs() {
            a.entry(&keypair);
        }
        a.finish()
    }
}

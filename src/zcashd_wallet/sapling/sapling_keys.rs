use std::collections::HashMap;

use zewif::sapling::SaplingIncomingViewingKey;

use super::SaplingKey;

#[derive(Clone, PartialEq)]
pub struct SaplingKeys(HashMap<SaplingIncomingViewingKey, SaplingKey>);

impl SaplingKeys {
    pub fn new(map: HashMap<SaplingIncomingViewingKey, SaplingKey>) -> Self {
        Self(map)
    }

    pub fn keypairs(&self) -> impl Iterator<Item = &SaplingKey> {
        self.0.values()
    }

    pub fn get(&self, ivk: &SaplingIncomingViewingKey) -> Option<&SaplingKey> {
        self.0.get(ivk)
    }
}

impl std::fmt::Debug for SaplingKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut a = f.debug_list();
        for keypair in self.keypairs() {
            a.entry(keypair);
        }
        a.finish()
    }
}

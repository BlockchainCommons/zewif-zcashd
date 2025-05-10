use anyhow::{Context, Result, bail};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
};
use zcash_keys::keys::UnifiedFullViewingKey;
use zewif::{Bip39Mnemonic, SeedFingerprint, TxId, sapling::SaplingIncomingViewingKey};

use crate::{
    DBValue, ZcashdDump, ZcashdWallet, parse,
    parser::prelude::*,
    zcashd_dump::DBKey,
    zcashd_wallet::{
        Address, BlockLocator, ClientVersion, KeyMetadata, MnemonicHDChain, NetworkInfo,
        RecipientAddress, RecipientMapping, UfvkFingerprint, UnifiedAccountMetadata,
        UnifiedAccounts, UnifiedAddressMetadata,
        orchard::OrchardNoteCommitmentTree,
        sapling::{SaplingKey, SaplingKeys, SaplingZPaymentAddress},
        sprout::{SproutKeys, SproutPaymentAddress, SproutSpendingKey},
        transparent::{Key, KeyPoolEntry, Keys, PrivKey, PubKey},
        u252,
    },
};

#[derive(Debug)]
pub struct ZcashdParser<'a> {
    pub dump: &'a ZcashdDump,
    pub unparsed_keys: RefCell<HashSet<DBKey>>,
}

impl<'a> ZcashdParser<'a> {
    pub fn parse_dump(dump: &ZcashdDump) -> Result<(ZcashdWallet, HashSet<DBKey>)> {
        let parser = ZcashdParser::new(dump);
        parser.parse()
    }

    fn new(dump: &'a ZcashdDump) -> Self {
        let unparsed_keys = RefCell::new(dump.records().keys().cloned().collect());
        Self {
            dump,
            unparsed_keys,
        }
    }

    // Keep track of which keys have been parsed
    fn mark_key_parsed(&self, key: &DBKey) {
        self.unparsed_keys.borrow_mut().remove(key);
    }

    fn value_for_keyname(&self, keyname: &str) -> Result<&DBValue> {
        let key = self.dump.key_for_keyname(keyname);
        self.mark_key_parsed(&key);
        self.dump.value_for_keyname(keyname)
    }

    fn parse(&self) -> Result<(ZcashdWallet, HashSet<DBKey>)> {
        //
        // Since version 3
        //

        // ~~acc~~: Removed in 4.5.0
        // ~~acentry~~: Removed in 4.5.0

        // **bestblock**: Empty in 6.0.0
        let bestblock = self.parse_block_locator("bestblock")?;

        // ~~**chdseed**~~: Removed in 5.0.0

        // ckey

        // csapzkey

        // cscript

        // czkey

        // **defaultkey**
        let default_key = self.parse_default_key()?;

        // destdata

        // **hdchain**

        // ~~hdseed~~: Removed in 5.0.0

        // key
        // keymeta
        let keys = self.parse_keys()?;

        // **minversion**
        let min_version = self.parse_client_version("minversion")?;

        // **mkey**

        // name
        let address_names = self.parse_address_names()?;

        // **orderposnext**
        let orderposnext = self.parse_opt_i64("orderposnext")?;

        // pool
        let key_pool = self.parse_key_pool()?;

        // purpose
        let address_purposes = self.parse_address_purposes()?;

        // sapzaddr
        let sapling_z_addresses = self.parse_sapling_z_addresses()?;

        // sapextfvk

        // sapzkey
        let sapling_keys = self.parse_sapling_keys()?;

        // tx
        let transactions = self.parse_transactions()?;

        // **version**
        let client_version = self.parse_client_version("version")?;

        // vkey

        // watchs

        // **witnesscachesize**
        let witnesscachesize = self.parse_i64("witnesscachesize")?;

        // wkey

        // zkey
        // zkeymeta
        let sprout_keys = self.parse_sprout_keys()?;

        //
        // Since version 5
        //

        // **networkinfo**
        let network_info = self.parse_network_info()?;

        // **orchard_note_commitment_tree**
        let orchard_note_commitment_tree = self.parse_orchard_note_commitment_tree()?;

        // unifiedaccount

        // unifiedfvk

        // unifiedaddrmeta
        let unified_accounts = self.parse_unified_accounts()?;

        // **mnemonicphrase**
        let mnemonic_phrase = self.parse_mnemonic_phrase()?;

        // **cmnemonicphrase**

        // **mnemonichdchain**
        let mnemonic_hd_chain = self.parse_mnemonic_hd_chain()?;

        // recipientmapping
        let send_recipients = self.parse_send_recipients()?;

        //
        // Since version 6
        //

        // **bestblock_nomerkle**
        let bestblock_nomerkle = self.parse_opt_block_locator("bestblock_nomerkle")?;

        let wallet = ZcashdWallet::new(
            address_names,
            address_purposes,
            bestblock_nomerkle,
            bestblock,
            client_version,
            default_key,
            key_pool,
            keys,
            min_version,
            mnemonic_hd_chain,
            mnemonic_phrase,
            network_info,
            orchard_note_commitment_tree,
            orderposnext,
            sapling_keys,
            sapling_z_addresses,
            send_recipients,
            sprout_keys,
            transactions,
            unified_accounts,
            witnesscachesize,
        );

        Ok((wallet, self.unparsed_keys.borrow().clone()))
    }

    fn parse_i64(&self, keyname: &str) -> Result<i64> {
        let value = self.value_for_keyname(keyname)?;
        parse!(buf = value, i64, format!("i64 for keyname: {}", keyname))
    }

    fn parse_opt_i64(&self, keyname: &str) -> Result<Option<i64>> {
        if self.dump.has_value_for_keyname(keyname) {
            self.parse_i64(keyname).map(Some)
        } else {
            Ok(None)
        }
    }

    fn parse_client_version(&self, keyname: &str) -> Result<ClientVersion> {
        let value = self.value_for_keyname(keyname)?;
        parse!(
            buf = value,
            ClientVersion,
            format!("client version for keyname: {}", keyname)
        )
    }

    fn parse_block_locator(&self, keyname: &str) -> Result<BlockLocator> {
        let value = self.value_for_keyname(keyname)?;
        parse!(
            buf = value,
            BlockLocator,
            format!("block locator for keyname: {}", keyname)
        )
    }

    fn parse_opt_block_locator(&self, keyname: &str) -> Result<Option<BlockLocator>> {
        if self.dump.has_value_for_keyname(keyname) {
            self.parse_block_locator(keyname).map(Some)
        } else {
            Ok(None)
        }
    }

    fn parse_keys(&self) -> Result<Keys> {
        let key_records = self
            .dump
            .records_for_keyname("key")
            .context("Getting 'key' records")?;
        let keymeta_records = self
            .dump
            .records_for_keyname("keymeta")
            .context("Getting 'keymeta' records")?;
        if key_records.len() != keymeta_records.len() {
            bail!("Mismatched key and keymeta records");
        }
        let mut keys_map = HashMap::new();
        for (key, value) in key_records {
            let pubkey = parse!(buf = &key.data, PubKey, "pubkey")?;
            let privkey = parse!(buf = value.as_data(), PrivKey, "privkey")?;
            let metakey = DBKey::new("keymeta", &key.data);
            let metadata_binary = self
                .dump
                .value_for_key(&metakey)
                .context("Getting metadata")?;
            let metadata = parse!(buf = metadata_binary, KeyMetadata, "metadata")?;
            let keypair =
                Key::new(pubkey.clone(), privkey.clone(), metadata).context("Creating keypair")?;
            keys_map.insert(pubkey, keypair);

            self.mark_key_parsed(&key);
            self.mark_key_parsed(&metakey);
        }
        Ok(Keys::new(keys_map))
    }

    fn parse_sapling_keys(&self) -> Result<SaplingKeys> {
        let mut keys_map = HashMap::new();
        if !self.dump.has_keys_for_keyname("sapzkey") {
            return Ok(SaplingKeys::new(keys_map));
        }
        let key_records = self
            .dump
            .records_for_keyname("sapzkey")
            .context("Getting 'sapzkey' records")?;
        let keymeta_records = self
            .dump
            .records_for_keyname("sapzkeymeta")
            .context("Getting 'sapzkeymeta' records")?;
        if key_records.len() != keymeta_records.len() {
            bail!("Mismatched sapzkey and sapzkeymeta records");
        }
        for (key, value) in key_records {
            let ivk = parse!(buf = &key.data, SaplingIncomingViewingKey, "ivk")?;
            let spending_key = parse!(
                buf = value.as_data(),
                ::sapling::zip32::ExtendedSpendingKey,
                "spending_key"
            )?;
            let metakey = DBKey::new("sapzkeymeta", &key.data);
            let metadata_binary = self
                .dump
                .value_for_key(&metakey)
                .context("Getting sapzkeymeta metadata")?;
            let metadata = parse!(buf = metadata_binary, KeyMetadata, "sapzkeymeta metadata")?;
            let keypair = SaplingKey::new(ivk, spending_key.clone(), metadata)
                .context("Creating keypair")?;
            keys_map.insert(ivk, keypair);

            self.mark_key_parsed(&key);
            self.mark_key_parsed(&metakey);
        }
        Ok(SaplingKeys::new(keys_map))
    }

    fn parse_sprout_keys(&self) -> Result<Option<SproutKeys>> {
        if !self.dump.has_keys_for_keyname("zkey") {
            return Ok(None);
        }
        let zkey_records = self
            .dump
            .records_for_keyname("zkey")
            .context("Getting 'zkey' records")?;
        let zkeymeta_records = self
            .dump
            .records_for_keyname("zkeymeta")
            .context("Getting 'zkeymeta' records")?;
        if zkey_records.len() != zkeymeta_records.len() {
            bail!("Mismatched zkey and zkeymeta records");
        }
        let mut zkeys_map = HashMap::new();
        for (key, value) in zkey_records {
            let payment_address = parse!(buf = &key.data, SproutPaymentAddress, "payment_address")?;
            let spending_key = parse!(buf = value.as_data(), u252, "spending_key")?;
            let metakey = DBKey::new("zkeymeta", &key.data);
            let metadata_binary = self
                .dump
                .value_for_key(&metakey)
                .context("Getting metadata")?;
            let metadata = parse!(buf = metadata_binary, KeyMetadata, "metadata")?;
            let keypair = SproutSpendingKey::new(spending_key, metadata);
            zkeys_map.insert(payment_address, keypair);

            self.mark_key_parsed(&key);
            self.mark_key_parsed(&metakey);
        }
        Ok(Some(SproutKeys::new(zkeys_map)))
    }

    fn parse_default_key(&self) -> Result<PubKey> {
        let value = self.value_for_keyname("defaultkey")?;
        parse!(buf = value, PubKey, "defaultkey")
    }

    fn parse_mnemonic_hd_chain(&self) -> Result<MnemonicHDChain> {
        let value = self.value_for_keyname("mnemonichdchain")?;
        parse!(buf = value, MnemonicHDChain, "mnemonichdchain")
    }

    fn parse_send_recipients(&self) -> Result<HashMap<TxId, Vec<RecipientMapping>>> {
        let mut send_recipients: HashMap<TxId, Vec<RecipientMapping>> = HashMap::new();
        if !self.dump.has_keys_for_keyname("recipientmapping") {
            return Ok(send_recipients);
        }
        let records = self
            .dump
            .records_for_keyname("recipientmapping")
            .context("Getting 'recipientmapping' records")?;
        for (key, value) in records {
            let mut p = Parser::new(&key.data);
            let txid = parse!(&mut p, TxId, "txid")?;
            let recipient_address = parse!(&mut p, RecipientAddress, "recipient_address")?;
            p.check_finished()?;
            let unified_address = parse!(buf = &value, String, "unified_address")?;
            let recipient_mapping = RecipientMapping::new(recipient_address, unified_address);
            send_recipients
                .entry(txid)
                .or_default()
                .push(recipient_mapping);
            self.mark_key_parsed(&key);
        }

        Ok(send_recipients)
    }

    fn parse_unified_accounts(&self) -> Result<UnifiedAccounts> {
        if !self.dump.has_keys_for_keyname("unifiedaddrmeta") {
            return Ok(UnifiedAccounts::none());
        }
        let address_metadata_records = self.dump.records_for_keyname("unifiedaddrmeta")?;
        let mut address_metadata = vec![];
        for (key, value) in address_metadata_records {
            let metadata = parse!(
                buf = &key.data,
                UnifiedAddressMetadata,
                "UnifiedAddressMetadata key"
            )?;
            address_metadata.push(metadata);
            let v: u32 = parse!(buf = value.as_data(), u32, "UnifiedAddressMetadata value")?;
            if v != 0 {
                bail!("Unexpected value for UnifiedAddressMetadata: 0x{:08x}", v);
            }
            self.mark_key_parsed(&key);
        }

        let account_metadata_records = self.dump.records_for_keyname("unifiedaccount")?;
        let mut account_metadata = HashMap::new();
        for (key, value) in account_metadata_records {
            let metadata = parse!(
                buf = &key.data,
                UnifiedAccountMetadata,
                "UnifiedAccountMetadata key"
            )?;
            account_metadata.insert(*metadata.ufvk_fingerprint(), metadata);
            let v: u32 = parse!(buf = value.as_data(), u32, "UnifiedAccountMetadata value")?;
            if v != 0 {
                bail!("Unexpected value for UnifiedAccountMetadata: 0x{:08x}", v);
            }
            self.mark_key_parsed(&key);
        }

        let full_viewing_keys_records = self.dump.records_for_keyname("unifiedfvk")?;
        let mut full_viewing_keys = HashMap::new();
        for (key, value) in full_viewing_keys_records {
            let key_id = parse!(
                buf = &key.data,
                UfvkFingerprint,
                "UnifiedFullViewingKey key"
            )?;
            let fvk = parse!(
                buf = value.as_data(),
                UnifiedFullViewingKey,
                "UnifiedFullViewingKey value"
            )?;
            full_viewing_keys.insert(key_id, fvk);
            self.mark_key_parsed(&key);
        }

        Ok(UnifiedAccounts::new(
            address_metadata,
            full_viewing_keys,
            account_metadata,
        ))
    }

    fn parse_mnemonic_phrase(&self) -> Result<Bip39Mnemonic> {
        let (key, value) = self
            .dump
            .record_for_keyname("mnemonicphrase")
            .context("Getting 'mnemonicphrase' record")?;
        let fingerprint = parse!(buf = &key.data, SeedFingerprint, "seed fingerprint")?;
        let mut bip39_mnemonic = parse!(buf = &value, Bip39Mnemonic, "mnemonic phrase")?;
        bip39_mnemonic.set_fingerprint(fingerprint);
        self.mark_key_parsed(&key);
        Ok(bip39_mnemonic)
    }

    fn parse_address_names(&self) -> Result<HashMap<Address, String>> {
        let records = self
            .dump
            .records_for_keyname("name")
            .context("Getting 'name' records")?;
        let mut address_names = HashMap::new();
        for (key, value) in records {
            let address = parse!(buf = &key.data, Address, "address")?;
            let name = parse!(buf = value.as_data(), String, "name")?;
            if address_names.contains_key(&address) {
                bail!("Duplicate address found: {}", address);
            }
            address_names.insert(address, name);

            self.mark_key_parsed(&key);
        }
        Ok(address_names)
    }

    fn parse_address_purposes(&self) -> Result<HashMap<Address, String>> {
        let records = self
            .dump
            .records_for_keyname("purpose")
            .context("Getting 'purpose' records")?;
        let mut address_purposes = HashMap::new();
        for (key, value) in records {
            let address = parse!(buf = &key.data, Address, "address")?;
            let purpose = parse!(buf = value.as_data(), String, "purpose")?;
            if address_purposes.contains_key(&address) {
                bail!("Duplicate address found: {}", address);
            }
            address_purposes.insert(address, purpose);

            self.mark_key_parsed(&key);
        }
        Ok(address_purposes)
    }

    fn parse_sapling_z_addresses(
        &self,
    ) -> Result<HashMap<SaplingZPaymentAddress, SaplingIncomingViewingKey>> {
        let mut sapling_z_addresses = HashMap::new();
        if !self.dump.has_keys_for_keyname("sapzaddr") {
            return Ok(sapling_z_addresses);
        }
        let records = self
            .dump
            .records_for_keyname("sapzaddr")
            .context("Getting 'sapzaddr' records")?;
        for (key, value) in records {
            let payment_address =
                parse!(buf = &key.data, SaplingZPaymentAddress, "payment address")?;
            let viewing_key = parse!(
                buf = value.as_data(),
                SaplingIncomingViewingKey,
                "viewing key"
            )?;
            if sapling_z_addresses.contains_key(&payment_address) {
                bail!("Duplicate payment address found: {:?}", payment_address);
            }
            sapling_z_addresses.insert(payment_address, viewing_key);

            self.mark_key_parsed(&key);
        }
        Ok(sapling_z_addresses)
    }

    fn parse_network_info(&self) -> Result<NetworkInfo> {
        let value = self
            .value_for_keyname("networkinfo")
            .context("Getting 'networkinfo' record")?;
        let network_info = parse!(buf = value.as_data(), NetworkInfo, "network info")?;
        Ok(network_info)
    }

    fn parse_orchard_note_commitment_tree(&self) -> Result<OrchardNoteCommitmentTree> {
        let value = self
            .value_for_keyname("orchard_note_commitment_tree")
            .context("Getting 'orchard_note_commitment_tree' record")?;
        let orchard_note_commitment_tree = parse!(
            buf = value.as_data(),
            OrchardNoteCommitmentTree,
            "orchard note commitment tree"
        )?;
        Ok(orchard_note_commitment_tree)
    }

    fn parse_key_pool(&self) -> Result<HashMap<i64, KeyPoolEntry>> {
        let records = self
            .dump
            .records_for_keyname("pool")
            .context("Getting 'pool' records")?;
        let mut key_pool = HashMap::new();
        for (key, value) in records {
            let index = parse!(buf = &key.data, i64, "key pool index")?;
            let entry = parse!(buf = value.as_data(), KeyPoolEntry, "key pool entry")?;
            key_pool.insert(index, entry);

            self.mark_key_parsed(&key);
        }
        Ok(key_pool)
    }

    fn parse_transactions(&self) -> Result<HashMap<TxId, WalletTx>> {
        let mut transactions = HashMap::new();
        // Some wallet files don't have any transactions
        if self.dump.has_keys_for_keyname("tx") {
            let records = self
                .dump
                .records_for_keyname("tx")
                .context("Getting 'tx' records")?;
            let mut sorted_records: Vec<_> = records.into_iter().collect();
            sorted_records.sort_by(|(key1, _), (key2, _)| key1.data.cmp(&key2.data));
            for (key, value) in sorted_records {
                let txid = parse!(buf = &key.data, TxId, "transaction ID")?;
                let trace = false;
                let transaction = parse!(buf = value.as_data(), WalletTx, "transaction", trace)?;
                if transactions.contains_key(&txid) {
                    bail!("Duplicate transaction found: {:?}", txid);
                }
                transactions.insert(txid, transaction);

                self.mark_key_parsed(&key);
            }
        }
        Ok(transactions)
    }
}

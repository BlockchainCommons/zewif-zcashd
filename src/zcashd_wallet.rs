use zewif::mod_use;

mod_use!(address);
mod_use!(block_locator);
mod_use!(client_version);
mod_use!(compact_size);
mod_use!(key_metadata);
mod_use!(incremental_merkle_tree);
mod_use!(incremental_witness);
mod_use!(mnemonic_hd_chain);
mod_use!(network_info);
mod_use!(parseable_types);
mod_use!(receiver_type);
mod_use!(recipient_address);
mod_use!(recipient_mapping);
mod_use!(seconds_since_epoch);
mod_use!(unified_accounts);
mod_use!(unified_account_metadata);
mod_use!(unified_address_metadata);
mod_use!(u160_type);
mod_use!(u252_type);
mod_use!(u256_type);
mod_use!(wallet_tx);

pub mod orchard;
pub mod sapling;
pub mod sprout;
pub mod transparent;

use std::collections::HashMap;
use zewif::{Bip39Mnemonic, Network, TxId, sapling::SaplingIncomingViewingKey};

use orchard::OrchardNoteCommitmentTree;
use sapling::{SaplingKeys, SaplingZPaymentAddress};
use sprout::SproutKeys;
use transparent::{KeyPoolEntry, Keys, PubKey};

#[derive(Debug)]
pub struct ZcashdWallet {
    address_names: HashMap<Address, String>,
    address_purposes: HashMap<Address, String>,
    bestblock_nomerkle: Option<BlockLocator>,
    bestblock: BlockLocator,
    client_version: ClientVersion,
    default_key: PubKey,
    key_pool: HashMap<i64, KeyPoolEntry>,
    keys: Keys,
    min_version: ClientVersion,
    mnemonic_hd_chain: MnemonicHDChain,
    bip39_mnemonic: Bip39Mnemonic,
    network_info: NetworkInfo,
    orchard_note_commitment_tree: OrchardNoteCommitmentTree,
    orderposnext: Option<i64>,
    sapling_keys: SaplingKeys,
    sapling_z_addresses: HashMap<SaplingZPaymentAddress, SaplingIncomingViewingKey>,
    send_recipients: HashMap<TxId, Vec<RecipientMapping>>,
    sprout_keys: Option<SproutKeys>,
    transactions: HashMap<TxId, WalletTx>,
    unified_accounts: UnifiedAccounts,
    witnesscachesize: i64,
}

impl ZcashdWallet {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        address_names: HashMap<Address, String>,
        address_purposes: HashMap<Address, String>,
        bestblock_nomerkle: Option<BlockLocator>,
        bestblock: BlockLocator,
        client_version: ClientVersion,
        default_key: PubKey,
        key_pool: HashMap<i64, KeyPoolEntry>,
        keys: Keys,
        min_version: ClientVersion,
        mnemonic_hd_chain: MnemonicHDChain,
        bip39_mnemonic: Bip39Mnemonic,
        network_info: NetworkInfo,
        orchard_note_commitment_tree: OrchardNoteCommitmentTree,
        orderposnext: Option<i64>,
        sapling_keys: SaplingKeys,
        sapling_z_addresses: HashMap<SaplingZPaymentAddress, SaplingIncomingViewingKey>,
        send_recipients: HashMap<TxId, Vec<RecipientMapping>>,
        sprout_keys: Option<SproutKeys>,
        transactions: HashMap<TxId, WalletTx>,
        unified_accounts: UnifiedAccounts,
        witnesscachesize: i64,
    ) -> Self {
        ZcashdWallet {
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
            bip39_mnemonic,
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
        }
    }
    pub fn address_names(&self) -> &HashMap<Address, String> {
        &self.address_names
    }

    pub fn address_purposes(&self) -> &HashMap<Address, String> {
        &self.address_purposes
    }

    pub fn bestblock_nomerkle(&self) -> Option<&BlockLocator> {
        self.bestblock_nomerkle.as_ref()
    }

    pub fn bestblock(&self) -> &BlockLocator {
        &self.bestblock
    }

    pub fn client_version(&self) -> &ClientVersion {
        &self.client_version
    }

    pub fn default_key(&self) -> &PubKey {
        &self.default_key
    }

    pub fn key_pool(&self) -> &HashMap<i64, KeyPoolEntry> {
        &self.key_pool
    }

    pub fn keys(&self) -> &Keys {
        &self.keys
    }

    pub fn min_version(&self) -> &ClientVersion {
        &self.min_version
    }

    pub fn mnemonic_hd_chain(&self) -> &MnemonicHDChain {
        &self.mnemonic_hd_chain
    }

    pub fn bip39_mnemonic(&self) -> &Bip39Mnemonic {
        &self.bip39_mnemonic
    }

    pub fn network_info(&self) -> &NetworkInfo {
        &self.network_info
    }

    pub fn orchard_note_commitment_tree(&self) -> &OrchardNoteCommitmentTree {
        &self.orchard_note_commitment_tree
    }

    pub fn orderposnext(&self) -> Option<i64> {
        self.orderposnext
    }

    pub fn sapling_keys(&self) -> &SaplingKeys {
        &self.sapling_keys
    }

    pub fn sapling_z_addresses(
        &self,
    ) -> &HashMap<SaplingZPaymentAddress, SaplingIncomingViewingKey> {
        &self.sapling_z_addresses
    }

    pub fn send_recipients(&self) -> &HashMap<TxId, Vec<RecipientMapping>> {
        &self.send_recipients
    }

    pub fn sprout_keys(&self) -> Option<&SproutKeys> {
        self.sprout_keys.as_ref()
    }

    pub fn transactions(&self) -> &HashMap<TxId, WalletTx> {
        &self.transactions
    }

    pub fn unified_accounts(&self) -> &UnifiedAccounts {
        &self.unified_accounts
    }

    pub fn witnesscachesize(&self) -> i64 {
        self.witnesscachesize
    }
}

impl ZcashdWallet {
    pub fn network(&self) -> Network {
        self.network_info.network()
    }
}

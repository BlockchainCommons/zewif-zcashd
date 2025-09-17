use zewif::Network;

pub(crate) fn address_network_from_zewif(
    network: Network,
) -> zcash_address::Network {
    match network {
        Network::Main => zcash_address::Network::Main,
        Network::Test => zcash_address::Network::Test,
        Network::Regtest => zcash_address::Network::Regtest,
    }
}

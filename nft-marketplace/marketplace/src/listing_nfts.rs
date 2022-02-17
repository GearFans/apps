use crate::{MarketEvent, Market, GAS_RESERVE};
use gstd::{msg, debug, exec, prelude::*, ActorId};
use primitive_types::{U256};

impl Market {
    /// Lists user's NFTs on the market
    /// Called from NFT contract
    /// Arguments:
    /// * `owner`: the tokens owner
    /// * `token_ids`: the NFT ids that will be listed. If `None`: all the NFTs a user has will be listed on the market
    /// * `price`: the price for NFT items
    /// * `on_sale`: determines whether to put up for sale or not
    pub async fn call_from_nft_contract(&mut self, owner: &ActorId, tokens: Vec<U256>, price: u128, on_sale: bool) {
        if !self.approved_nft_contracts.contains(&msg::source()) {
            panic!("that nft contract is not approved");
        }
        for token in tokens.iter() {
            self.create_item(&msg::source(), *token, owner, price, on_sale).await;
        }
        msg::reply(
            MarketEvent::NFTsListed {
                nft_contract_id: msg::source(),
                owner: *owner,
                tokens,
                price,
            },
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }
}
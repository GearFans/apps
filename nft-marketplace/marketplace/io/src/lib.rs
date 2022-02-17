#![no_std]
use codec::{Decode, Encode};
use gstd::{exec, msg, prelude::*, ActorId};
use primitive_types::{H256, U256};
use scale_info::TypeInfo;

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct InitMarket {
    pub owner_id: ActorId,
    pub treasury_id: ActorId,
    pub treasury_fee: u128,
    pub approved_ft_token: ActorId,
    pub offer_history_length: Option<u8>,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum MarketAction {
    AddNftContract(ActorId),
    BuyItem {
        nft_contract_id: ActorId,
        token_id: U256,
    },
    AddBid {
        nft_contract_id: ActorId,
        token_id: U256,
        price: u128,
    },
    CreateAuction {
        nft_contract_id: ActorId,
        token_id: U256,
        price: u128,
        bid_period: u64,
    },
    SettleAuction{
        nft_contract_id: ActorId,
        token_id: U256,
    },
    NFTContractCall {
        owner: ActorId,
        tokens: Vec<U256>,
        price: u128,
        on_sale: bool,
    },
    Item {
        nft_contract_id: ActorId,
        token_id: U256,
    }
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum MarketEvent {
    ItemSold {
        owner: ActorId,
        nft_contract_id: ActorId,
        token_id: U256,
    },
    BidAdded {
        nft_contract_id: ActorId,
        token_id: U256,
        price: u128,
    },
    AuctionCreated {
        nft_contract_id: ActorId,
        token_id: U256,
        price: u128,
    },
    AuctionSettled {
        nft_contract_id: ActorId,
        token_id: U256,
        price: u128,
    },
    AuctionCancelled{
        nft_contract_id: ActorId,
        token_id: U256,
    },
    NFTsListed {
        nft_contract_id: ActorId,
        owner: ActorId,
        tokens: Vec<U256>,
        price: u128,
    },
    Item {
        owner_id: ActorId,
        nft_contract_id: ActorId,
        token_id: U256,
        price: u128,
        on_sale: bool,
    },
}


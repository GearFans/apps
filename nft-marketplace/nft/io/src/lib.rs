#![no_std]
use codec::{Decode, Encode};
use gstd::{String, BTreeMap,prelude::*, Vec, ActorId};
use primitive_types::{U256};
use scale_info::TypeInfo;
pub use royalties::*;

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct InitNFT {
    pub name: String,
    pub symbol: String,
    pub base_uri: String,
    pub price: u128,
    pub supply: U256,
    pub royalties: Option<Royalties>,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub struct MessageToMarket {
    pub sale: bool,
    pub price: u128,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum NFTAction {
    Mint {
        media: String,
        reference: String,
    },
    Burn(U256),
    Transfer {
        to: ActorId,
        token_id: U256,
    },
    Approve {
        to: ActorId,
        token_ids: Option<Vec<U256>>,
        message: Option<MessageToMarket>,
    },
    OwnerOf(U256),
    BalanceOf(ActorId),
    TokensForOwner(ActorId),
    NFTPayout {
        owner: ActorId,
        amount: u128,
    },
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum NFTEvent {
    Transfer {
        from: ActorId,
        to: ActorId,
        token_id: U256,
    },
    Approval {
        owner: ActorId,
        spender: ActorId,
        token_ids: Vec<U256>,
    },
    ApprovalOnCall {
        owner: ActorId,
        spender: ActorId,
        token_ids: Vec<U256>,
    },
    ApprovalForAll {
        owner: ActorId,
        operator: ActorId,
        approved: bool,
    },
    OwnerOf(ActorId),
    BalanceOf(U256),
    TokensForOwner(Vec<U256>),
    NFTPayout(BTreeMap<ActorId, u128>),
}
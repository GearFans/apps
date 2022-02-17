#![no_std]
#![feature(const_btree_new)]

use codec::{Encode};
use gstd::{debug, exec, msg, prelude::*, ActorId};
use primitive_types::{H256, U256};

pub use nft_io::*;
pub use royalties::*;

pub mod state;
pub use state::{State, StateReply};

pub mod market_messages;
pub use market_messages::list_nfts_on_market;

use non_fungible_token::base::NonFungibleTokenBase;
use non_fungible_token::token::TokenMetadata;
use non_fungible_token::{NonFungibleToken};

const GAS_RESERVE: u64 = 500_000_000;
const ZERO_ID: ActorId = ActorId::new([0u8; 32]);

#[derive(Debug)]
pub struct NFT {
    pub tokens: NonFungibleToken,
    pub owner: ActorId,
    pub owner_to_ids: BTreeMap<ActorId, Vec<U256>>,
    pub supply: U256,
    pub royalties: Option<Royalties>,
    pub price: u128,
    pub token_id: U256,
}

static mut CONTRACT: NFT = NFT {
    tokens: NonFungibleToken {
        name: String::new(),
        symbol: String::new(),
        base_uri: String::new(),
        owner_by_id: BTreeMap::new(),
        token_metadata_by_id: BTreeMap::new(),
        token_approvals: BTreeMap::new(),
        balances: BTreeMap::new(),
        operator_approval: BTreeMap::new(),
    },
    owner: ActorId::new(H256::zero().to_fixed_bytes()),
    owner_to_ids: BTreeMap::new(),
    supply: U256::zero(),
    royalties: None,
    price: 0,
    token_id: U256::zero(),
};

impl NFT {
    fn mint(&mut self, media: String, reference: String) {
        if self.token_id >= self.supply {
            panic!("No tokens left");
        }
        let token_id = self.token_id;
        self.token_id = self.token_id.saturating_add(U256::one());

        self.tokens.owner_by_id.insert(token_id, msg::source());
        self.owner_to_ids.entry(msg::source())
            .and_modify(|ids| ids.push(token_id))
            .or_insert(vec![token_id]);
        let metadata = TokenMetadata {
            title: None,
            description: None,
            media: Some(media),
            reference: Some(reference),
        };
        self.tokens.token_metadata_by_id.insert(self.token_id, metadata);
        let balance = *self
            .tokens
            .balances
            .get(&msg::source())
            .unwrap_or(&U256::zero());
        self.tokens
            .balances
            .insert(msg::source(), balance.saturating_add(U256::one()));

        msg::reply(
            NFTEvent::Transfer {
                from: ZERO_ID,
                to: msg::source(),
                token_id: self.token_id,
            },
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }

    fn burn(&mut self, token_id: U256) {
        if !self.tokens.exists(token_id) {
            panic!("NonFungibleToken: Token does not exist");
        }
        if !self.tokens.is_token_owner(token_id, &msg::source()) {
            panic!("NonFungibleToken: account is not owner");
        }
        self.tokens.token_approvals.remove(&token_id);
        self.tokens.owner_by_id.remove(&token_id);
        let balance = *self
            .tokens
            .balances
            .get(&msg::source())
            .unwrap_or(&U256::zero());
        self.tokens
            .balances
            .insert(msg::source(), balance.saturating_sub(U256::one()));

        msg::reply(
            NFTEvent::Transfer {
                from: msg::source(),
                to: ZERO_ID,
                token_id,
            },
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }

   async fn approve(
        &mut self,
        to: &ActorId,
        token_ids: Option<Vec<U256>>,
        msg: Option<MessageToMarket>,
    ) {
        let tokens: Vec<U256> = 
            if token_ids.is_some() {
                token_ids.unwrap()
            } else {
                self.owner_to_ids
                    .get(&msg::source())
                    .expect("there are no tokens")
                    .to_vec()
            };
        for token in tokens.clone().iter() {
            if self.tokens.owner_by_id.get(&token).unwrap_or(&ZERO_ID) != &msg::source() {
                panic!("Only owner can send token to market");
            }
            self.tokens.token_approvals.insert(*token, *to);
        }
        if msg.is_some() {
               list_nfts_on_market(
                    to, 
                    &msg::source(), 
                    tokens.clone(),
                    msg.as_ref().unwrap().price,
                    msg.unwrap().sale)
                    .await;
        }
        msg::reply(
            NFTEvent::Approval{
                owner: msg::source(),
                spender: *to,
                token_ids: tokens,
            },
            exec::gas_available() - GAS_RESERVE,
            0,
        );     
    }

    async fn nft_payout(&self, owner: &ActorId, amount: u128,) {
        let payouts: Payout = 
            if self.royalties.is_some() {
                self.royalties.as_ref().unwrap().payouts(owner, amount)
            } else {
                let mut single_payout = BTreeMap::new();
                single_payout.insert(*owner, amount);
                single_payout
            };
        msg::reply(
            NFTEvent::NFTPayout(payouts),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }
}

gstd::metadata! {
    title: "NFT",
        init:
            input: InitNFT,
        handle:
            input: NFTAction,
            output: NFTEvent,
        state:
            input: State,
            output: StateReply,
}

#[gstd::async_main]
async fn main() {
    let action: NFTAction = msg::load().expect("Could not load Action");
    match action {
        NFTAction::Mint {media, reference} => {
            CONTRACT.mint(media, reference);
        }
        NFTAction::Burn(token_id) => {
            CONTRACT.burn(token_id);
        }
        NFTAction::Transfer {to, token_id} => {
            CONTRACT.tokens.transfer(
                &msg::source(),
                &to,
                token_id,
            );
        }
        NFTAction::TokensForOwner(account) => {
            let tokens = CONTRACT.owner_to_ids
                    .get(&account)
                    .unwrap_or(&vec![])
                    .clone();
            debug!("tokens {:?}", tokens);
            msg::reply(
                NFTEvent::TokensForOwner(tokens),
                exec::gas_available() - GAS_RESERVE,
                0,
            );
        }
        NFTAction::NFTPayout {owner, amount} => {
            CONTRACT.nft_payout(&owner, amount).await;
        }
        NFTAction::Approve {to, token_ids, message} => {
            CONTRACT.approve(&to, token_ids, message).await;
        }
        NFTAction::OwnerOf(token_id) => {
            let owner = CONTRACT.tokens.owner_by_id.get(&token_id).unwrap_or(&ZERO_ID);
            msg::reply(
                NFTEvent::OwnerOf(*owner),
                exec::gas_available() - GAS_RESERVE,
                0,
            );
        }
        NFTAction::BalanceOf(account) => {
            let balance = *CONTRACT
                .tokens
                .balances
                .get(&account)
                .unwrap_or(&U256::zero());
            msg::reply(
                NFTEvent::BalanceOf(balance),
                exec::gas_available() - GAS_RESERVE,
                0,
            );
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitNFT = msg::load().expect("Unable to decode InitConfig");
    CONTRACT
        .tokens
        .init(config.name, config.symbol, config.base_uri);
    CONTRACT.owner = msg::source();
    CONTRACT.price = config.price;
    CONTRACT.supply = config.supply;
    CONTRACT.royalties = config.royalties;
}

#[no_mangle]
pub unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
    let query: State = msg::load().expect("failed to decode input argument");
    let encoded = match query {
        State::BalanceOfUser(input) => {
            let user = &ActorId::new(input.to_fixed_bytes());
            StateReply::BalanceOfUser(*CONTRACT.tokens.balances.get(user).unwrap_or(&U256::zero()))
                .encode()
        }
        State::TokenOwner(input) => {
            let user = CONTRACT.tokens.owner_by_id.get(&input).unwrap_or(&ZERO_ID);
            StateReply::TokenOwner(H256::from_slice(user.as_ref())).encode()
        }
        State::IsTokenOwner(input) => {
            let user = CONTRACT
                .tokens
                .owner_by_id
                .get(&input.token_id)
                .unwrap_or(&ZERO_ID);
            StateReply::IsTokenOwner(user == &ActorId::new(input.user.to_fixed_bytes())).encode()
        }
        State::GetApproved(input) => {
            let approved_address = CONTRACT
                .tokens
                .token_approvals
                .get(&input)
                .unwrap_or(&ZERO_ID);
            StateReply::GetApproved(H256::from_slice(approved_address.as_ref())).encode()
        }
    };
    let result = gstd::macros::util::to_wasm_ptr(&(encoded[..]));

    core::mem::forget(encoded);

    result
}

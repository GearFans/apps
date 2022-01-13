#![no_std]
#![feature(const_btree_new)]

use codec::{Decode, Encode};
use gstd::{debug, exec, msg, prelude::*, ActorId};
use primitive_types::{H256, U256};
use scale_info::TypeInfo;

pub mod payloads;
pub use payloads::{ToMarket, ApproveForAllInput, ApproveInput, InitConfig, TransferInput, MintInput};

pub mod state;
pub use state::{State, StateReply};

use non_fungible_token::base::NonFungibleTokenBase;
use non_fungible_token::{Approve, ApproveForAll, NonFungibleToken, Transfer};
use non_fungible_token::token::{TokenMetadata};

const GAS_RESERVE: u64 = 500_000_000;
const ZERO_ID: ActorId = ActorId::new(H256::zero().to_fixed_bytes());

#[derive(Debug, Decode, TypeInfo)]
pub enum Action {
    Mint(MintInput),
    Burn(U256),
    Transfer(TransferInput),
    Approve(ApproveInput),
    ApproveForAll(ApproveForAllInput),
    OwnerOf(U256),
    BalanceOf(H256),
    SendToMarket(ToMarket),
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum Event {
    Transfer(Transfer),
    Approval(Approve),
    ApprovalForAll(ApproveForAll),
    OwnerOf(H256),
    BalanceOf(U256),
}

#[derive(Debug)]
pub struct NFT {
    pub tokens: NonFungibleToken,
    pub owner: ActorId,
    pub supply: U256,
    pub minted_amount: U256,
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
    supply: U256::zero(),
    minted_amount: U256::zero(),

};

impl NFT {
    fn mint(
        &mut self,
        token_id: U256,
        media: String,
        reference: String,
        ) {
        if self.minted_amount >= self.supply {
            panic!("No tokens left");
        }
        self.tokens.owner_by_id.insert(token_id, msg::source());
        let metadata = TokenMetadata {
            title: None,
            description: None,
            media: Some(media),
            reference: Some(reference),
        };
        self.tokens.token_metadata_by_id.insert(token_id, metadata);
        self.minted_amount = self.minted_amount.saturating_add(U256::one());
        let balance = *self
            .tokens
            .balances
            .get(&msg::source())
            .unwrap_or(&U256::zero());
        self.tokens
            .balances
            .insert(msg::source(), balance.saturating_add(U256::one()));

        let transfer_token = Transfer {
            from: H256::zero(),
            to: H256::from_slice(msg::source().as_ref()),
            token_id,
        };

        msg::reply(
            Event::Transfer(transfer_token),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }

    async fn send_token_to_market(
        &self,
        token_id: U256,
        price: u128,
    ) {
        if msg::source() == *self.tokens.owner_by_id.get(&token_id).unwrap_or(&ZERO_ID) {
            panic!("Only owner can send token to market");
        };
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

        let transfer_token = Transfer {
            from: H256::from_slice(msg::source().as_ref()),
            to: H256::zero(),
            token_id,
        };
        msg::reply(
            Event::Transfer(transfer_token),
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }
}

gstd::metadata! {
    title: "NFT",
        init:
            input: InitConfig,
        handle:
            input: Action,
            output: Event,
        state:
            input: State,
            output: StateReply,
}

#[gstd::async_main]
async fn main() {
    let action: Action = msg::load().expect("Could not load Action");
    match action {
        Action::Mint(input) => {
            CONTRACT.mint(
                input.token_id,
                input.media,
                input.reference,
            );
        }
        Action::Burn(input) => {
            CONTRACT.burn(input);
        }
        Action::Transfer(input) => {
            CONTRACT.tokens.transfer(
                &msg::source(),
                &ActorId::new(input.to.to_fixed_bytes()),
                input.token_id,
            );
        }
        Action::SendToMarket(input) => {
            CONTRACT.send_token_to_market(
                input.token_id,
                input.price,
            ).await;
        }
        Action::Approve(input) => {
            CONTRACT.tokens.approve(
                &msg::source(),
                &ActorId::new(input.to.to_fixed_bytes()),
                input.token_id,
            );
        }
        Action::ApproveForAll(input) => {
            CONTRACT.tokens.approve_for_all(
                &msg::source(),
                &ActorId::new(input.to.to_fixed_bytes()),
                input.approve,
            );
        }
        Action::OwnerOf(input) => {
            let owner = CONTRACT.tokens.owner_by_id.get(&input).unwrap_or(&ZERO_ID);
            msg::reply(
                Event::OwnerOf(H256::from_slice(owner.as_ref())),
                exec::gas_available() - GAS_RESERVE,
                0,
            );
        }
        Action::BalanceOf(input) => {
            let balance = *CONTRACT
                .tokens
                .balances
                .get(&ActorId::new(input.to_fixed_bytes()))
                .unwrap_or(&U256::zero());
            msg::reply(
                Event::BalanceOf(balance),
                exec::gas_available() - GAS_RESERVE,
                0,
            );
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitConfig = msg::load().expect("Unable to decode InitConfig");
    debug!("NFT {:?}", config);
    CONTRACT
        .tokens
        .init(config.name, config.symbol, config.base_uri);
    CONTRACT.owner = msg::source();
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

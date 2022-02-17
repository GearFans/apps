#![no_std]
#![feature(const_btree_new)]

use codec::{Decode, Encode};
use gstd::{collections::btree_map::Entry, exec, msg, prelude::*, ActorId};
use primitive_types::{H256, U256};
use scale_info::TypeInfo;
pub use market_io::*;
pub mod nft_messages;
use nft_messages::{nft_transfer};

pub mod ft_messages;
use ft_messages::ft_transfer;

pub mod auction;
use auction::Auction;

pub mod offers;
use offers::Offer;

pub mod sale;
pub mod listing_nfts;

const GAS_RESERVE: u64 = 500_000_000;
const ZERO_ID: ActorId = ActorId::new(H256::zero().to_fixed_bytes());
const OFFER_HISTORY_LENGTH_DEFAULT: u8 = 10;
pub type ContractAndTokenId = String;

#[derive(Debug, Encode, Decode, TypeInfo, Clone, Default)]
pub struct Item {
    pub owner_id: ActorId,
    pub nft_contract_id: ActorId,
    pub token_id: U256,
    pub price: u128,
    pub auction: Option<Auction>,
    pub offers: Option<Vec<Offer>>,
    pub on_sale: bool,
}

#[derive(Debug, Default, Encode, Decode, TypeInfo)]
pub struct Market {
    pub owner_id: ActorId,
    pub treasury_id: ActorId,
    pub treasury_fee: u128,
    pub items: BTreeMap<ContractAndTokenId, Item>,
    pub approved_ft_token: ActorId,
    pub approved_nft_contracts: Vec<ActorId>,
    pub offer_history_length: u8,
}

static mut MARKET: Option<Market> = None;

impl Market {
    async fn add_nft_contract(&mut self, nft_contract_id: &ActorId) {
        self.approved_nft_contracts.push(*nft_contract_id);
    }

    // Creates new item in the market.
    // Arguments:
    // * `nft_contract_id`: an actor, who wishes to become a DAO member
    // * `token_id`: the number of tokens the applicant offered for shares in DAO
    // * `price`: the amount of shares the applicant is requesting for his token tribute
    async fn create_item(
        &mut self, 
        nft_contract_id: &ActorId, 
        token_id: U256, 
        owner_id: &ActorId,
        price: u128, 
        on_sale: bool
    ) {
    //    let owner_id = nft_owner_of(nft_contract_id, token_id).await;
       // nft_transfer(nft_contract_id, &exec::program_id(), token_id).await;
        let contract_and_token_id =
            format!("{}{}", H256::from_slice(nft_contract_id.as_ref()), token_id);
        let new_item = Item {
            owner_id: *owner_id,
            nft_contract_id: *nft_contract_id,
            token_id,
            price,
            auction: None,
            on_sale,
            offers: None,
        };
        self.items.insert(contract_and_token_id, new_item);
    }

    fn item_exists(&mut self, nft_contract_id: &ActorId, token_id: U256) -> bool {
        let contract_and_token_id =
            format!("{}{}", H256::from_slice(nft_contract_id.as_ref()), token_id);
        match self.items.entry(contract_and_token_id) {
            Entry::Occupied(_o) => true,
            Entry::Vacant(_v) => false,
        }
    }
}

gstd::metadata! {
    title: "NFTMarketplace",
        init:
            input: InitMarket,
        handle:
            input: MarketAction,
            output: MarketEvent,
        state:
            input: State,
            output: StateReply,
}

#[gstd::async_main]
async fn main() {
    let action: MarketAction = msg::load().expect("Could not load Action");
    let market: &mut Market = MARKET.get_or_insert(Market::default());
    match action {
        MarketAction::AddNftContract(nft_contract_id) => {
            market.add_nft_contract(&nft_contract_id).await;
        }
        MarketAction::NFTContractCall {owner, tokens, price, on_sale} => {
            market.call_from_nft_contract(
                    &owner,
                    tokens,
                    price,
                    on_sale,
                )
                .await;
        }
        MarketAction::BuyItem {nft_contract_id, token_id} => {
            market.buy_item(
                    &nft_contract_id,
                    token_id,
                )
                .await;
        }
        MarketAction::Item {nft_contract_id, token_id} => {
            let contract_and_token_id =
            format!("{}{}", H256::from_slice(nft_contract_id.as_ref()), token_id);
            let item = market.items.entry(contract_and_token_id).or_insert(Item::default());
            msg::reply(
                MarketEvent::Item{
                    owner_id: item.owner_id,
                    nft_contract_id: item.nft_contract_id,
                    token_id: item.token_id,
                    price: item.price,
                    on_sale: item.on_sale,
                },
                exec::gas_available() - GAS_RESERVE,
                0,
            );
        }
        MarketAction::CreateAuction {nft_contract_id, token_id, price, bid_period} => market.create_auction(
            &nft_contract_id,
            token_id,
            price,
            bid_period,
        ),
        MarketAction::AddBid {nft_contract_id, token_id, price} => market.add_bid(
            &nft_contract_id,
            token_id,
            price,
        ).await,
        MarketAction::SettleAuction {nft_contract_id, token_id} => {
            market.settle_auction(
                    &nft_contract_id,
                    token_id,
                )
                .await;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitMarket= msg::load().expect("Unable to decode InitConfig");
    let market = Market {
        owner_id: config.owner_id,
        treasury_id: config.treasury_id,
        treasury_fee: config.treasury_fee,
        approved_ft_token: config.approved_ft_token,
        offer_history_length: config.offer_history_length
                                    .unwrap_or(OFFER_HISTORY_LENGTH_DEFAULT),
        ..Market::default()
    };
    MARKET = Some(market);
}

// #[no_mangle]
// pub unsafe extern "C" fn meta_state() -> *mut [i32; 2] {
//     let query: State = msg::load().expect("failed to decode input argument");
//     let encoded = match query {
//         State::ItemInfo => {

//         }
//     };
//     let result = gstd::macros::util::to_wasm_ptr(&(encoded[..]));

//     core::mem::forget(encoded);

//     result
// }

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum State {
    ItemInfo,
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum StateReply {
    ItemInfo,
}


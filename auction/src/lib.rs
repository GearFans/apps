#![no_std]
#![feature(const_btree_new)]

#[cfg(test)]
use codec::{Decode, Encode};
use core::time::Duration;
use gstd::{debug, exec, msg, prelude::*, ActorId};
use primitive_types::U256;
use scale_info::TypeInfo;

const GAS_RESERVE: u64 = 500_000_000;
const ZERO_ID: ActorId = ActorId::new([0u8; 32]);
const DURATION: u64 = 60 * 60 * 24; // 1 day

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct InitConfig {
    pub StartAt: u64,   // timestamp
    pub ExpiresAt: u64, // timestamp
    pub Price: u64,
}

#[derive(Debug)]
struct Auction {
    start_at: u64,   // timestamp
    expires_at: u64, // timestamp
    price: u64,
}

impl Auction {
    pub const fn new() -> Self {
        Self {
            start_at: 0,
            expires_at: 0,
            price: 0,
        }
    }

    fn start(&self) {}
    fn end(&self) {}
    fn get_price(&self) {}
    fn buy(&self) {}
    fn withdraw(&self) {}
}

static mut AUCTION: Auction = Auction::new();

gstd::metadata! {
    title: "Auction",
    init:
        input: InitConfig,
    handle:
        input: Action,
        output: Event,
    state:
        input: State,
        output: StateReply,
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitConfig = msg::load().expect("Unable to decode InitConfig");

    let bt = Duration::from_millis(exec::block_timestamp());
    let _expiresAt = bt.saturating_add(Duration::from_secs(DURATION));

    AUCTION.start_at = bt.as_secs();
    AUCTION.expires_at = _expiresAt.as_secs();
    AUCTION.price = config.Price;
}

pub unsafe extern "C" fn handle() {
    let action: Action = msg::load().expect("Could not load Action");
    match action {
        Action::Start(newtime) => {
            AUCTION.start();
            // event
        }

        Action::End(newtime) => {
            AUCTION.end();
            // event
        }

        Action::GetPrice(tokenid) => {
            AUCTION.get_price();
        }

        Action::Buy(newtime) => {
            AUCTION.buy();
            // event
        }

        Action::Withdraw() => {
            AUCTION.withdraw();
            // event
        }
    }
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum Action {
    Buy(u128),
    GetPrice(ActorId),
    Start(u128),
    End(u128),
    Withdraw(),
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum Event {
    StartAction(),
    EndAction(),
    Buy(u128, ActorId),
    Withdraw(),
    ChangeDate(),
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum State {
    StartAt(),
    ExpiresAt(),
    Price(),
}

#[derive(Debug, Decode, Encode, TypeInfo)]
pub enum StateReply {
    StartAt(),
    ExpiresAt(),
    Price(),
}

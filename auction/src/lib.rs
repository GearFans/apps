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

#[derive(Debug, Decode, Encode, TypeInfo)]
pub struct InitConfig {
    pub StartAt: u64,   // timestamp
    pub ExpiresAt: u64, // timestamp
    pub Price: u64,
}

#[derive(Clone)]
pub struct Auction {
    pub StartAt: u64,   // timestamp
    pub ExpiresAt: u64, // timestamp
    pub Price: u64,
}

impl Auction {
    pub const fn new() -> Self {
        Self {
            StartAt: 0,
            ExpiresAt: 0,
            Price: 0,
        }
    }
}

static mut AUCTION: Auction = Auction::new();

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: InitConfig = msg::load().expect("Unable to decode InitConfig");

    let bt = Duration::from_millis(exec::block_timestamp());

    let _expiresAt = bt.saturating_add(Duration::from_secs(DURATION));
    AUCTION.StartAt = bt.as_secs();
    AUCTION.ExpiresAt = _expiresAt.as_secs();
    AUCTION.Price = config.Price;
}

pub unsafe extern "C" fn handle() {
    let new_msg = String::from_utf8(msg::load_bytes()).expect("Invalid message");

    if new_msg == "PING" {
        msg::reply_bytes("PONG", 12_000_000, 0);
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

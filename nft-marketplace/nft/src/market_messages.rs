use gstd::{msg, prelude::*, ActorId};
use primitive_types::{U256};
pub use market_io::*;
const GAS_RESERVE: u64 = 1000_000_000;

pub async fn list_nfts_on_market(market_id: &ActorId, owner: &ActorId, tokens: Vec<U256>, price: u128, on_sale: bool) {
    let _market_response: MarketEvent = msg::send_and_wait_for_reply(
        *market_id,
        MarketAction::NFTContractCall{
            owner: *owner,
            tokens,
            price,
            on_sale,
        },
        GAS_RESERVE,
        0,
    )
    .await
    .expect("error in sending message to marketplace");
}


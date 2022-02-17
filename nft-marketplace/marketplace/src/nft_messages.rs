use crate::{ZERO_ID};
use gstd::{msg, debug, prelude::*, ActorId};
use primitive_types::{U256};
pub type Payout = BTreeMap<ActorId, u128>;
use nft_io::*;
const GAS_RESERVE: u64 = 800_000_000;

pub async fn nft_transfer(nft_program_id: &ActorId, to: &ActorId, token_id: U256) {
    let _transfer_response: NFTEvent = msg::send_and_wait_for_reply(
        *nft_program_id,
        NFTAction::Transfer {
            to: *to,
            token_id,
        },
        GAS_RESERVE,
        0,
    )
    .await
    .expect("error in transfer");
}

pub async fn nft_owner_of(nft_program_id: &ActorId, token_id: U256) -> ActorId {
    let owner_of: NFTEvent = msg::send_and_wait_for_reply(
        *nft_program_id,
        NFTAction::OwnerOf(token_id),
        GAS_RESERVE,
        0,
    )
    .await
    .expect("Error in function 'nft_owner_of' call");
   match owner_of {
        NFTEvent::OwnerOf(owner) => owner,
        _ => ZERO_ID,
   }
}


pub async fn tokens_for_owner(nft_program_id: &ActorId, account: &ActorId) -> Vec<U256> {
    debug!("account {:?}", account);
    let tokens: NFTEvent = msg::send_and_wait_for_reply(
        *nft_program_id,
        NFTAction::TokensForOwner(*account),
        GAS_RESERVE,
        0,
    )
    .await
    .expect("Error in function 'nft_owner_of' call");
    match tokens {
        NFTEvent::TokensForOwner(tokens) => tokens,
        _ => vec![],
   }
}

pub async fn check_owner(
    nft_program_id: &ActorId, 
    account: &ActorId,
    token_ids: Vec<U256>,
) {
    for token_id in token_ids {
        let owner: NFTEvent = msg::send_and_wait_for_reply(
            *nft_program_id,
            NFTAction::OwnerOf(token_id),
            GAS_RESERVE,
            0,
        )
        .await
        .expect("Error in function 'nft_owner_of' call");  
        match owner {
            NFTEvent::OwnerOf(owner) => {
                if owner != *account {
                    panic!("Only owner can list NFTs");
                }
            }
            _ => panic!("Unexpected reply"),
        }
    }
}

pub async fn nft_payouts(nft_program_id: &ActorId, owner: &ActorId, amount: u128,) -> Payout {
    let payouts: NFTEvent = msg::send_and_wait_for_reply(
        *nft_program_id,
        NFTAction::NFTPayout {
            owner: *owner,
            amount
        },
       GAS_RESERVE,
        0,
    )
    .await
    .expect("Error in function 'nft_payout' call");
    match payouts {
        NFTEvent::NFTPayout(payouts) => payouts,
        _ => BTreeMap::new(),
   }
}

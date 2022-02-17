use gstd::{msg, ActorId};
use fungible_token_messages::*;
const GAS_RESERVE: u64 = 500_000_000;

pub async fn ft_transfer(token_id: &ActorId, from: &ActorId, to: &ActorId, amount: u128) {
    let transfer_data = TransferFromInput {
        owner: *from,
        to: *to,
        amount,
    };
    let _transfer_response: Event = msg::send_and_wait_for_reply(
        *token_id,
        Action::TransferFrom(transfer_data),
        GAS_RESERVE,
        0,
    )
    .await
    .expect("Error in transfer message");
}

pub async fn balance(token_id: &ActorId, account: &ActorId) -> u128 {
    let balance: Event = msg::send_and_wait_for_reply(
        *token_id,
        Action::BalanceOf(*account),
        GAS_RESERVE,
        0,
    )
    .await
    .expect("Error in balance message");

    match balance {
        Event::Balance(balance) => return balance,
        _ => panic!("Unexpected reply"),
    }
}


use codec::Encode;
use market_io::*;
use nft_io::*;
use fungible_token_messages::*;

use gtest::{Program, System};
use primitive_types::U256;

fn init_ft(sys: &System) {
    let ft = Program::from_file(
        &sys,
        "../../../apps/target/wasm32-unknown-unknown/release/fungible_token.wasm",
    );

    let res = ft.send(
        100001,
        InitConfig {
            name: String::from("MyToken"),
            symbol: String::from("MTK"),
        },
    );

    assert!(res.log().is_empty());
}

fn init_nft(sys: &System) {
    sys.init_logger();
    let nft = Program::from_file(
        &sys,
        "../../../apps/target/wasm32-unknown-unknown/release/nft.wasm",
    );

    let res = nft.send(
        100001,
        InitNFT {
            name: String::from("MyToken"),
            symbol: String::from("MTK"),
            base_uri: "".to_string(),
            price: 100,
            supply: 100.into(),
        },
    );

    assert!(res.log().is_empty());
}

fn init_market(sys: &System) {
    sys.init_logger();
    let market = Program::from_file(
        &sys,
        "../../../apps/target/wasm32-unknown-unknown/release/nft_marketplace.wasm",
    );

    let res = market.send(
        100001,
        InitMarket {
            owner_id: 3.into(),
            treasury_id: 3.into(),
            treasury_fee: 100,
            approved_ft_token: 1.into(),
            offer_history_length: None,
        },
    );
    assert!(res.log().is_empty());
}

fn mint_ft_tokens(ft: &Program, user: u64, amount: u128) {
    let res = ft.send(
        100001,
        Action::Mint(MintInput {
            account: user.into(),
            amount,
        }),
    );
    assert!(!res.main_failed());

    let res = ft.send(
        user,
        Action::Approve(ApproveInput {
            spender: 3.into(),
            amount,
        }),
    );
    assert!(!res.main_failed());
}
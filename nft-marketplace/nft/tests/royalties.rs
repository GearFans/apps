use codec::Encode;
use market_io::*;
use nft_io::*;
use fungible_token_messages::*;

use gtest::{Program, System};
use primitive_types::U256;

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


fn before_each_test(sys: &System) {
    init_ft(&sys);
    init_nft(&sys);
    let ft =  sys.get_program(1);
    let nft = sys.get_program(2);
    let market = sys.get_program(3);

    for user in USERS {
        mint_tokens(&ft, *user);
    }

    let res = nft.send (
        USERS[0],
        NFTAction::Mint {
            media: "".to_string(),
            reference: "".to_string()
        }
    );
    assert!(!res.main_failed());

    let res = market.send(USERS[0], MarketAction::AddNftContract(2.into()));
    assert!(res.log().is_empty());
    // lists nft on the market
    let res = nft.send(
        USERS[0],
        NFTAction::Approve {
            to: 3.into(),
            token_ids: Some(vec![0.into()]),
            message: Some(MessageToMarket {
                sale: true,
                price: 100,
            }),
        }
    );
    assert!(!res.main_failed());
}

#[test]
fn validate() {
    let sys = System::new();
    sys.init_logger();
    init_nft(&sys);
    init_market(&sys);
    let nft = sys.get_program(1);
    let market = sys.get_program(2);

    // User mints NFTs in the nft-contract
    let mut tokens: Vec<U256> = Vec::new();
    for i in 0..9 {
        tokens.push(i.into());
        mint_nft(&nft);
    };

    // User lists his NFTs on marketplace
    let res = nft.send(
        4,
        NFTAction::Approve {
            to: 2.into(),
            token_ids: None,
            message: Some(MessageToMarket {
                sale: true,
                price: 100,
            }),
        }
    );
    assert!(res.contains(&(
        4,
        NFTEvent::Approval {
            owner: 4.into(),
            spender: 2.into(),
            token_ids: tokens,
        }
        .encode()
    )));

    // Checks that items has appeared on the market
    for i in 0..9 {
        let res = market.send(
            4,
            MarketAction::Item {
                nft_contract_id: 1.into(),
                token_id: i.into(),
            }
        );
        assert!(res.contains(&(
            4,
            MarketEvent::Item {
                owner_id: 4.into(),
                nft_contract_id: 1.into(),
                token_id: i.into(),
                price: 100,
                on_sale: true,
            }
            .encode()
        )));
    };
}

#[test]
fn lists_selected_nfts() {
    let sys = System::new();
    sys.init_logger();
    init_nft(&sys);
    init_market(&sys);
    let nft = sys.get_program(1);
    let market = sys.get_program(2);

    // User mints NFTs in the nft-contract
    let tokens = vec![1.into(), 3.into(), 5.into()];
    for _i in 0..9 {
        mint_nft(&nft);
    };

    // User lists his NFTs on marketplace
    let res = nft.send(
        4,
        NFTAction::Approve {
            to: 2.into(),
            token_ids: Some(tokens.clone()),
            message: Some(MessageToMarket {
                sale: true,
                price: 100,
            }),
        }
    );
    assert!(res.contains(&(
        4,
        NFTEvent::Approval {
            owner: 4.into(),
            spender: 2.into(),
            token_ids: tokens.clone(),
        }
        .encode()
    )));

    //Checks that items has appeared on the market
    for token in tokens.iter() {
        let res = market.send(
            4,
            MarketAction::Item {
                nft_contract_id: 1.into(),
                token_id:  *token,
            }
        );
        assert!(res.contains(&(
            4,
            MarketEvent::Item {
                owner_id: 4.into(),
                nft_contract_id: 1.into(),
                token_id: *token,
                price: 100,
                on_sale: true,
            }
            .encode()
        )));
    };
}


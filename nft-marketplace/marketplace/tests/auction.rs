use codec::Encode;
use market_io::*;
use nft_io::*;
use fungible_token_messages::*;
use gtest::{Program, System};
use primitive_types::U256;
const USERS: &'static [u64] = &[4, 5, 6, 7];

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
            royalties: None,
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

fn mint_tokens(ft: &Program, user: u64) {
    let res = ft.send(
        100001,
        Action::Mint(MintInput {
            account: user.into(),
            amount: 1000000,
        }),
    );
    assert!(!res.main_failed());

    let res = ft.send(
        user,
        Action::Approve(ApproveInput {
            spender: 3.into(),
            amount: 1000000,
        }),
    );
    assert!(!res.main_failed());
}

fn before_each_test(sys: &System) {
    init_ft(&sys);
    init_nft(&sys);
    init_market(&sys);
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
fn create_auction() {
    let sys = System::new();
    sys.init_logger();
    before_each_test(&sys);
    let market = sys.get_program(3);

    let res = market.send(
        USERS[0],
        MarketAction::CreateAuction {
            nft_contract_id: 2.into(),
            token_id: 0.into(),
            price: 100,
            bid_period: 10000,
        }
    );
    assert!(res.contains(&(
        USERS[0],
        MarketEvent::AuctionCreated {
            nft_contract_id: 2.into(),
            token_id: 0.into(),
            price: 100,
        }
        .encode()
    )));
}

#[test]
fn create_auction_failures() {
    let sys = System::new();
    sys.init_logger();
    before_each_test(&sys);
    let market = sys.get_program(3);

    // must fail since a caller is not the item owner
    let res = market.send(
        USERS[1],
        MarketAction::CreateAuction {
            nft_contract_id: 2.into(),
            token_id: 0.into(),
            price: 100,
            bid_period: 10000,
        }
    );
    assert!(res.main_failed());

    // must fail since an item doesn't exist
    let res = market.send(
        USERS[0],
        MarketAction::CreateAuction {
            nft_contract_id: 2.into(),
            token_id: 1.into(),
            price: 100,
            bid_period: 10000,
        }
    );
    assert!(res.main_failed());

    // creates auction
    let res = market.send(
        USERS[0],
        MarketAction::CreateAuction {
            nft_contract_id: 2.into(),
            token_id: 0.into(),
            price: 100,
            bid_period: 10000,
        }
    );
    assert!(!res.main_failed());

    // must fail since the auction is alreay on
    let res = market.send(
        USERS[0],
        MarketAction::CreateAuction {
            nft_contract_id: 2.into(),
            token_id: 0.into(),
            price: 100,
            bid_period: 10000,
        }
    );
    assert!(res.main_failed());
}

#[test]
fn add_bid() {
    let sys = System::new();
    sys.init_logger();
    before_each_test(&sys);
    let market = sys.get_program(3);
    let res = market.send(
        USERS[0],
        MarketAction::CreateAuction {
            nft_contract_id: 2.into(),
            token_id: 0.into(),
            price: 100,
            bid_period: 10000,
        }
    );
    assert!(!res.main_failed());

    let res = market.send(
        USERS[1],
        MarketAction::AddBid {
            nft_contract_id: 2.into(),
            token_id: 0.into(),
            price: 101,
        }
    );

     assert!(res.contains(&(
        USERS[1],
        MarketEvent::BidAdded {
            nft_contract_id: 2.into(),
            token_id: 0.into(),
            price: 101,
        }
        .encode()
    )));

    // must fail since the price is equal to the current bid price
    let res = market.send(
        USERS[1],
        MarketAction::AddBid {
            nft_contract_id: 2.into(),
            token_id: 0.into(),
            price: 101,
        }
    );
    assert!(res.main_failed());

    // must fail since the user has no enough balance
    let res = market.send(
        USERS[1],
        MarketAction::AddBid {
            nft_contract_id: 2.into(),
            token_id: 0.into(),
            price: 10000000,
        }
    );
    assert!(res.main_failed());

    sys.spend_blocks(10001);

    // must fail since the auction has ended
    let res = market.send(
        USERS[1],
        MarketAction::AddBid {
            nft_contract_id: 2.into(),
            token_id: 0.into(),
            price: 101,
        }
    );
    assert!(res.main_failed());

}


#[test]
fn settle_auction() {
    let sys = System::new();
    sys.init_logger();
    before_each_test(&sys);
    let market = sys.get_program(3);
    let res = market.send(
        USERS[0],
        MarketAction::CreateAuction {
            nft_contract_id: 2.into(),
            token_id: 0.into(),
            price: 100,
            bid_period: 10000,
        }
    );
    assert!(!res.main_failed());

    // Users add bids
    USERS.iter().enumerate().for_each(|(i, user)| {
        let res = market.send(
            *user,
            MarketAction::AddBid {
                nft_contract_id: 2.into(),
                token_id: 0.into(),
                price: 101 + i as u128,
            }
        );
        assert!(!res.main_failed());
    });
    
    sys.spend_blocks(10001);

    let res = market.send(
        USERS[0],
        MarketAction::SettleAuction {
            nft_contract_id: 2.into(),
            token_id: 0.into(),
        }
    );

    assert!(res.contains(&(
        USERS[0],
        MarketEvent::AuctionSettled {
            nft_contract_id: 2.into(),
            token_id: 0.into(),
            price: 104,
        }
        .encode()
    )));

    let nft = sys.get_program(2);
    // Checks the NFT owner
    let res = nft.send(USERS[0], NFTAction::OwnerOf(0.into()));
    assert!(res.contains(&(
        USERS[0],
        NFTEvent::OwnerOf(7.into())
        .encode()
    )));
}

#[test]
fn auction_is_cancelled() {
    let sys = System::new();
    sys.init_logger();
    before_each_test(&sys);
    let market = sys.get_program(3);
    let res = market.send(
        USERS[0],
        MarketAction::CreateAuction {
            nft_contract_id: 2.into(),
            token_id: 0.into(),
            price: 100,
            bid_period: 10000,
        }
    );
    assert!(!res.main_failed());


    sys.spend_blocks(10001);

    let res = market.send(
        USERS[0],
        MarketAction::SettleAuction {
            nft_contract_id: 2.into(),
            token_id: 0.into(),
        }
    );

    assert!(res.contains(&(
        USERS[0],
        MarketEvent::AuctionCancelled {
            nft_contract_id: 2.into(),
            token_id: 0.into(),
        }
        .encode()
    )));
}

#[test]
fn settle_auction_failures() {
    let sys = System::new();
    sys.init_logger();
    before_each_test(&sys);
    let market = sys.get_program(3);
    let res = market.send(
        USERS[0],
        MarketAction::CreateAuction {
            nft_contract_id: 2.into(),
            token_id: 0.into(),
            price: 100,
            bid_period: 10000,
        }
    );
    assert!(!res.main_failed());

    // must fail since the auction is not over
    let res = market.send(
        USERS[0],
        MarketAction::SettleAuction {
            nft_contract_id: 2.into(),
            token_id: 0.into(),
        }
    );
    assert!(res.main_failed());
    
    let nft = sys.get_program(2);
    let res = nft.send (
        USERS[0],
        NFTAction::Mint {
            media: "".to_string(),
            reference: "".to_string()
        }
    );
    assert!(!res.main_failed());

    // lists nft on the market
    let res = nft.send(
        USERS[0],
        NFTAction::Approve {
            to: 3.into(),
            token_ids: Some(vec![1.into()]),
            message: Some(MessageToMarket {
                sale: true,
                price: 100,
            }),
        }
    );

    // must fail since the auction doesn't exist
    let res = market.send(
        USERS[0],
        MarketAction::SettleAuction {
            nft_contract_id: 2.into(),
            token_id: 1.into(),
        }
    );
    assert!(res.main_failed());
}

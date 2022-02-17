use crate::{
    ft_messages::{ft_transfer, balance}, nft_transfer, MarketEvent, Market, Item, GAS_RESERVE,
};
use codec::{Decode, Encode};
use gstd::{exec, debug, msg, prelude::*, ActorId};
use primitive_types::{H256, U256};
use scale_info::TypeInfo;

#[derive(Debug, Encode, Decode, TypeInfo, Clone)]
pub struct Bid {
    pub id: ActorId,
    pub price: u128,
}

#[derive(Debug, Default, Encode, Decode, TypeInfo, Clone)]
pub struct Auction {
    pub bid_period: u64,
    pub started_at: u64,
    pub ended_at: u64,
    pub current_price: u128,
    pub bids: Option<Vec<Bid>>,
}

impl Market {
    /// Creates an auction for selected item
    /// Requirements:
    /// * Only the item owner can start auction
    /// Arguments:
    /// * `nft_contract_id`: the NFT contract address
    /// * `token_id`: the NFT id
    /// * `min_price`: the starting price
    /// * `bid_period`: the time that the auction lasts until another bid occurs
    pub fn create_auction(
        &mut self,
        nft_contract_id: &ActorId,
        token_id: U256,
        min_price: u128,
        bid_period: u64,
    ) {       
        let contract_and_token_id =
            format!("{}{}", H256::from_slice(nft_contract_id.as_ref()), token_id);
        let item = self.items
            .get_mut(&contract_and_token_id)
            .expect("Item does not exist");
        if item.auction.is_some() {
            panic!("auction already exists");
        }
        if item.owner_id != msg::source() {
            panic!("not nft owner");
        }
        item.auction = Some(Auction {
            bid_period,
            started_at: exec::block_timestamp(),
            ended_at: exec::block_timestamp() + bid_period,
            current_price: min_price,
            bids: None,
        });
        msg::reply(
            MarketEvent::AuctionCreated {
                nft_contract_id: *nft_contract_id,
                token_id,
                price: min_price,
            },
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }

    /// Settles the auction
    /// Requirements:
    /// * The auction must be over
    /// Arguments:
    /// * `nft_contract_id`: the NFT contract address
    /// * `token_id`: the NFT id
    pub async fn settle_auction(&mut self, nft_contract_id: &ActorId, token_id: U256) {
        let contract_and_token_id =
            format!("{}{}", H256::from_slice(nft_contract_id.as_ref()), token_id);
        let item = self.items
            .get_mut(&contract_and_token_id)
            .expect("Item does not exist");

        let auction = item.auction.clone().expect("Auction doesn not exist");

        if auction.ended_at > exec::block_timestamp() {
            panic!("Auction is not over");
        }

        let bids = auction.bids.unwrap_or(Vec::new());

        if !bids.is_empty() {
            let highest_bid = &bids[bids.len() - 1];
            // transfer payment to owner
            ft_transfer(
                &self.approved_ft_token,
                &highest_bid.id,
                &item.owner_id,
                highest_bid.price,
            )
            .await;
            // transfer NFT
            nft_transfer(nft_contract_id, &highest_bid.id, token_id).await;
            msg::reply(
                MarketEvent::AuctionSettled {
                    nft_contract_id: *nft_contract_id,
                    token_id,
                    price: highest_bid.price,
                },
                exec::gas_available() - GAS_RESERVE,
                0,
            );
        } else {
            msg::reply(
                MarketEvent::AuctionCancelled {
                    nft_contract_id: *nft_contract_id,
                    token_id,
                },
                exec::gas_available() - GAS_RESERVE,
                0,
            );
        }
        item.auction = None;
    }

    /// Adds a bid to an ongoing auction
    /// Requirements:
    /// * The auction must be on
    /// * The caller must have enough balance for the offered price
    /// Arguments:
    /// * `nft_contract_id`: the NFT contract address
    /// * `token_id`: the NFT id
    /// * `price`: the offered price
    pub async fn add_bid(&mut self, nft_contract_id: &ActorId, token_id: U256, price: u128) {
        
        let contract_and_token_id =
            format!("{}{}", H256::from_slice(nft_contract_id.as_ref()), token_id);

        let item = self
            .items
            .get_mut(&contract_and_token_id)
            .expect("Item does not exist");

        let mut auction = item.auction.clone().expect("Auction doesn not exist");
        if auction.ended_at < exec::block_timestamp() {
            panic!("Auction has already ended");
        }
        
        let balance = balance(&self.approved_ft_token, &msg::source()).await;
        if balance < price {
            panic!("Not enough balance for the offered price");
        }
        
        let mut bids = auction.bids.unwrap_or(Vec::new());
        if !bids.is_empty() {
            let current_bid = &bids[bids.len() - 1];
            if price <= current_bid.price {
                panic!("Cant offer less or equal to the current bid price")
            }
        }

        bids.push(Bid {
            id: msg::source(),
            price,
        });

        auction.ended_at = exec::block_timestamp() + auction.bid_period;
        auction.bids = Some(bids);
        auction.current_price = price;
        item.auction = Some(auction);
        msg::reply(
            MarketEvent::BidAdded {
                nft_contract_id: *nft_contract_id,
                token_id,
                price,
            },
            exec::gas_available() - GAS_RESERVE,
            0,
        );
    }
}

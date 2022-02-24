## Auction

### 参考

https://github.com/ethereumbook/ethereumbook/tree/develop/code/auction_dapp

https://github.com/brynbellomy/solidity-auction

https://www.quicknode.com/guides/solidity/how-to-create-a-dutch-auction-smart-contract

https://solidity-by-example.org/app/dutch-auction/

https://solidity-by-example.org/app/english-auction/

[English auction](https://en.wikipedia.org/wiki/English_auction)

[Dutch auction](https://en.wikipedia.org/wiki/Dutch_auction)

### 拍卖要素

开始时间
结束世纪
出价人
定价
出价
物品

### 拍卖逻辑

正常：物品 => 定价 => 拍卖 => 开始时间 =>  出价 => 出价逻辑 => 时间结束 => 物归谁家

流拍：

### 图示

> ![](./img/auction_diagram.png)

### 代码

```rust
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

pub struct InitConfig {
  pub StartAt: time,
  pub ExpiresAt: time,
  pub Price: u64,
}

pub enum Action {
  Buy(u128),
  GetPrice(ActorId),
  Start(u128),
  End(u128),
  Withdraw(),
}

pub enum Event {
  StartAction(),
  EndAction(),
  Buy(u128, ActorId),
  Withdraw(),
  ChangeDate(),
}

pub enum State {
  StartAt(),
  ExpiresAt(),
  Price(),
}

pub enum StateReply {
  StartAt(),
  ExpiresAt(),
  Price(),
}
```

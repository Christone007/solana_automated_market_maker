# Constant Product AMM (Solana + Anchor)

A constant-product Automated Market Maker (AMM) built on Solana using the Anchor framework.

This program implements a Uniswap V2-style `x * y = k` liquidity pool with:
- Liquidity provision (deposit / withdraw)
- Token swaps with slippage protection
- Configurable swap fees
- Protocol fee + treasury
- Admin-controlled fee collection

---

## Features

- **Constant Product Curve** (`x * y = k`)
- **Initialize Pool** – create a new trading pair with custom seed and fees
- **Deposit Liquidity** – add tokens and receive LP tokens
- **Withdraw Liquidity** – burn LP tokens and receive underlying tokens
- **Swap** – exchange tokens with slippage protection
- **Protocol Fee + Treasury** – a portion of swap fees goes to a protocol treasury
- **Collect Fees** – authority can withdraw accumulated protocol fees
- **Pool Locking** – ability to pause the pool

---

## Fee Model

| Parameter       | Description                          | Example |
|-----------------|--------------------------------------|---------|
| `fee`           | Total fee charged to traders (bps)   | 30 (0.30%) |
| `protocol_fee`  | Portion of the fee sent to treasury  | 5 (0.05%) |

- Remaining fee stays in the pool and benefits LP holders.
- Protocol fees accumulate in `treasury_x` and `treasury_y` (owned by the config PDA).
- Only the `authority` can withdraw from the treasury using the `collect_fees` instruction.

---

## Instructions

| Instruction      | Description                                      |
|------------------|--------------------------------------------------|
| `initialize`     | Create a new AMM pool                            |
| `deposit`        | Add liquidity and receive LP tokens              |
| `withdraw`       | Burn LP tokens and withdraw tokens               |
| `swap`           | Swap one token for the other                     |
| `collect_fees`   | Authority withdraws protocol fees from treasury  |

---

## Tech Stack

- **Solana**
- **Anchor**
- **SPL Token**
- [constant-product-curve](https://github.com/deanmlittle/constant-product-curve) (educational curve math)

---

## Getting Started

### Prerequisites

- Rust
- Solana CLI
- Anchor

### Build

```bash
anchor build


### Test

```bash
anchor test

### Project Structure

programs/amm/
├── src/
│   ├── instructions/
│   │   ├── initialize.rs
│   │   ├── deposit.rs
│   │   ├── withdraw.rs
│   │   ├── swap.rs
│   │   └── collect_fees.rs
│   ├── state.rs
│   ├── error.rs
│   └── lib.rs
└── Cargo.toml


### Tests
All instructions are covered by integration tests.
 ![AMM Tests Passing](./vault_tests.png) 

### Contact
* **Nwaburu Emeka Christian** - [GitHub Profile](https://github.com/Christone007)
* **Email:** exellentemy@gmail.com
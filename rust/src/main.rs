
use std::env;

use anyhow::{anyhow, Context, Result};
use ethers::types::{Address, U256};
use ethers::utils::parse_units;
use nadfun_sdk::prelude::*;
use nadfun_sdk::trade::{BuyParams, GasEstimationParams, SellParams, Trade};
use tokio::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let cfg = AppConfig::from_env()?;
    let trade = Trade::new(cfg.rpc_url.clone(), cfg.private_key.clone())
        .await
        .context("failed to initialize Trade client")?;

    let recipient = cfg
        .recipient
        .unwrap_or_else(|| trade.wallet_address());
    let deadline = cfg.deadline_u256();

    println!(
        "Preparing buy for token {} with {} MON",
        cfg.token,
        format_units(cfg.amount_in)?
    );

    let (router, quoted_out) = trade
        .get_amount_out(cfg.token, cfg.amount_in, true)
        .await
        .context("failed to query quote")?;

    let amount_out_min = apply_slippage(quoted_out, cfg.slippage_bps);

    let buy_gas = trade
        .estimate_gas(
            &router,
            GasEstimationParams::Buy {
                token: cfg.token,
                amount_in: cfg.amount_in,
                amount_out_min,
                to: recipient,
                deadline,
            },
        )
        .await
        .context("failed to estimate buy gas")?;

    println!("Estimated buy gas: {}", buy_gas);

    let buy_receipt = trade
        .buy(
            &router,
            BuyParams {
                token: cfg.token,
                amount_in: cfg.amount_in,
                amount_out_min,
                recipient,
                deadline,
            },
        )
        .await
        .context("buy transaction failed")?;

    println!("Buy submitted: {:?}", buy_receipt.tx_hash);

    tokio::time::sleep(Duration::from_secs(cfg.settlement_wait_secs)).await;

    let token_helper =
        TokenHelper::new(cfg.rpc_url.clone(), cfg.private_key.clone()).await?;
    let balance = token_helper
        .balance_of(cfg.token, recipient)
        .await
        .context("failed to fetch wallet balance")?;

    if balance.is_zero() {
        return Err(anyhow!("no balance available to sell"));
    }

    println!(
        "Selling {} tokens from {}",
        format_units(balance)?,
        recipient
    );

    let sell_receipt = trade
        .sell(
            &router,
            SellParams {
                token: cfg.token,
                amount_in: balance,
                amount_out_min: U256::zero(),
                recipient,
                deadline,
            },
        )
        .await
        .context("sell transaction failed")?;

    println!("Sell submitted: {:?}", sell_receipt.tx_hash);

    Ok(())
}

struct AppConfig {
    rpc_url: String,
    private_key: String,
    token: Address,
    amount_in: U256,
    slippage_bps: u64,
    recipient: Option<Address>,
    deadline_secs_from_now: u64,
    settlement_wait_secs: u64,
}

impl AppConfig {
    fn from_env() -> Result<Self> {
        let rpc_url = env::var("RPC_URL").context("RPC_URL missing")?;
        let private_key =
            env::var("PRIVATE_KEY").context("PRIVATE_KEY missing")?;
        let token_str =
            env::var("TOKEN_ADDRESS").context("TOKEN_ADDRESS missing")?;
        let amount_in = parse_units(
            env::var("AMOUNT_IN_MON")
                .unwrap_or_else(|_| "0.1".into()),
            18,
        )
        .context("invalid AMOUNT_IN_MON")?;
        let token: Address = token_str.parse().context("invalid token")?;

        let recipient = env::var("RECIPIENT_ADDRESS")
            .ok()
            .and_then(|value| value.parse().ok());

        let slippage_bps = env::var("SLIPPAGE_BPS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(100); // 1%

        let deadline_secs_from_now = env::var("DEADLINE_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(600);

        let settlement_wait_secs = env::var("SETTLEMENT_WAIT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);

        Ok(Self {
            rpc_url,
            private_key,
            token,
            amount_in: amount_in.into(),
            slippage_bps,
            recipient,
            deadline_secs_from_now,
            settlement_wait_secs,
        })
    }

    fn deadline_u256(&self) -> U256 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        U256::from(now + self.deadline_secs_from_now)
    }
}

fn apply_slippage(amount: U256, slippage_bps: u64) -> U256 {
    let basis: U256 = U256::from(10_000u64);
    let slip: U256 = U256::from(slippage_bps);
    amount * (basis - slip) / basis
}

fn format_units(value: U256) -> Result<String> {
    Ok(ethers::utils::format_units(value, 18)?)
}


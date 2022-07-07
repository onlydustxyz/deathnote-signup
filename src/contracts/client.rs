use std::{str::FromStr, thread, time::Duration};

use anyhow::{anyhow, Result};
use starknet::{
    accounts::SingleOwnerAccount,
    core::{
        chain_id::{MAINNET, TESTNET},
        types::{AddTransactionResult, FieldElement, TransactionStatus},
    },
    providers::{Provider, SequencerGatewayProvider},
    signers::{LocalWallet, SigningKey},
};

pub struct StarkNetClient {
    pub provider: SequencerGatewayProvider,
    pub account: SingleOwnerAccount<SequencerGatewayProvider, LocalWallet>,
    pub badge_registry_address: FieldElement,
}

impl StarkNetClient {
    pub fn new(
        hex_account_address: &str,
        hex_private_key: &str,
        hex_badge_registry_address: &str,
        chain: StarkNetChain,
    ) -> Self {
        let provider = match chain {
            StarkNetChain::Testnet => SequencerGatewayProvider::starknet_alpha_goerli(),
            StarkNetChain::Mainnet => SequencerGatewayProvider::starknet_alpha_mainnet(),
        };
        let account_provider = match chain {
            StarkNetChain::Testnet => SequencerGatewayProvider::starknet_alpha_goerli(),
            StarkNetChain::Mainnet => SequencerGatewayProvider::starknet_alpha_mainnet(),
        };
        let chain_id = match chain {
            StarkNetChain::Testnet => TESTNET,
            StarkNetChain::Mainnet => MAINNET,
        };
        let signer = LocalWallet::from(SigningKey::from_secret_scalar(
            FieldElement::from_hex_be(hex_private_key).expect("Invalid private key"),
        ));
        let account_address =
            FieldElement::from_hex_be(hex_account_address).expect("Invalid account address");
        let badge_registry_address = FieldElement::from_hex_be(hex_badge_registry_address)
            .expect("Invalid address for badge_registry");

        StarkNetClient {
            provider,
            account: SingleOwnerAccount::new(account_provider, signer, account_address, chain_id),
            badge_registry_address,
        }
    }

    pub async fn wait_for_transaction_acceptance(
        &self,
        transaction_result: AddTransactionResult,
    ) -> Result<AddTransactionResult> {
        info!(
            "Waiting for transaction 0x{:x} to be accepted",
            transaction_result.transaction_hash
        );

        loop {
            let receipt = match self
                .provider
                .get_transaction_status(transaction_result.transaction_hash)
                .await
                .map_err(anyhow::Error::msg)
            {
                Ok(receipt) => receipt,
                Err(e) => {
                    warn!("{}", e);
                    thread::sleep(Duration::from_secs(3));
                    continue;
                }
            };

            info!("Transaction is {:?}", receipt.status);

            break match receipt.status {
                TransactionStatus::NotReceived
                | TransactionStatus::Received
                | TransactionStatus::Pending => {
                    thread::sleep(Duration::from_secs(3));
                    continue;
                }
                TransactionStatus::AcceptedOnL2 | TransactionStatus::AcceptedOnL1 => {
                    Ok(transaction_result)
                }
                TransactionStatus::Rejected => Err(anyhow!("Transaction rejected")),
            };
        }
    }
}

pub enum StarkNetChain {
    Testnet,
    Mainnet,
}

impl FromStr for StarkNetChain {
    type Err = ();

    fn from_str(input: &str) -> Result<StarkNetChain, Self::Err> {
        match input {
            "TESTNET" => Ok(StarkNetChain::Testnet),
            "MAINNET" => Ok(StarkNetChain::Mainnet),
            _ => Err(()),
        }
    }
}

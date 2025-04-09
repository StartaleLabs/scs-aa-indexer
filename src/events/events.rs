use alloy::sol;
use serde::Serialize;

use alloy::primitives::{aliases::U48, Address, B256, U256};

// ✅ Define Solidity Events
sol! {
    // Events for User Sponsored and Gas Refunded from Paymaster
    #[derive(Debug, Serialize)] 
    event UserOperationSponsored(bytes32 indexed userOpHash, address indexed user);

    #[derive(Debug, Serialize)] 
    event GasBalanceDeducted(address indexed user, uint256 amount, uint256 premium, uint8 mode);

    #[derive(Debug, Serialize)] 
    event RefundProcessed(address indexed user, uint256 amount);

    #[derive(Debug, Serialize)]
    event PaidGasInTokens(address indexed user, address indexed token, uint256 tokenCharge, uint48 appliedMarkup, uint256 exchangeRate);

    #[derive(Debug, Serialize)] 
    event UserOperationEvent(bytes32 indexed userOpHash, address indexed sender, address indexed paymaster, uint256 nonce, bool success, uint256 actualGasCost, uint256 actualGasUsed);
}

// custom events
#[derive(Debug, Clone, Serialize)]
pub struct CombinedUserOpEvent {
    // From UserOperationEvent
    pub user_op_hash: B256,
    pub sender: Address,
    pub paymaster: Address,
    pub paymaster_type: String,
    pub nonce: U256,
    pub success: bool,
    pub actual_gas_cost: U256,
    pub actual_gas_used: U256,

    // The user's account used to pay for the gas
    pub deducted_user: Option<Address>,

    // From GasBalanceDeducted (optional)
    pub deducted_amount: Option<U256>,
    pub deducted_premium: Option<U256>,

    // From PaidGasInTokens (optional)
    pub token: Option<Address>,
    pub token_charge: Option<U256>,
    pub applied_markup: Option<U48>,
    pub exchange_rate: Option<U256>,
}
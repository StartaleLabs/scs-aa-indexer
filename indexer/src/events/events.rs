use alloy::sol;
use serde::Serialize;

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
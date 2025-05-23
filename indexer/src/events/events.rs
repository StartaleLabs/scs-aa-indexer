use alloy::sol;
use serde::Serialize;

// ✅ Define Solidity Events
sol! {
    // Events for Prepaid Paymaster
    #[derive(Debug, Serialize)] 
    event UserOperationSponsored(bytes32 indexed userOpHash, address indexed user);

    #[derive(Debug, Serialize)] 
    event GasBalanceDeducted(address indexed user, uint256 amount, uint256 premium);

    #[derive(Debug, Serialize)] 
    event RefundProcessed(address indexed user, uint256 amount);

    // Events for Token Paymaster
    #[derive(Debug, Serialize)]
    event PaidGasInTokens(address indexed user, address indexed token, uint256 tokenCharge, uint48 appliedMarkup, uint256 exchangeRate);

    // Events for Postpaid Paymaster
    #[derive(Debug, Serialize)] 
    event UserOperationSponsoredForPostpaid(bytes32 indexed userOpHash, address indexed user);

    // Events for Entry Point
    #[derive(Debug, Serialize)] 
    event UserOperationEvent(bytes32 indexed userOpHash, address indexed sender, address indexed paymaster, uint256 nonce, bool success, uint256 actualGasCost, uint256 actualGasUsed);
}
use alloy::sol;

// ✅ Define Solidity Events
sol! {
    // Events for User Sponsored and Gas Refunded from Paymaster
    event UserOperationSponsored(bytes32 indexed userOpHash, address indexed user);
    event GasBalanceDeducted(address indexed user, uint256 amount, uint256 premium, uint8 mode);
    event RefundProcessed(address indexed user, uint256 amount);

    // Event for User operation from Entry point
    event UserOperationEvent(bytes32 indexed userOpHash, address indexed sender, address indexed paymaster, uint256 nonce, bool success, uint256 actualGasCost, uint256 actualGasUsed);
}
use alloy::{
    sol,
    primitives::{Address, U256},
};

sol! {
    // Sponsership paymaster events
    event UserOperationSponsored(bytes32 indexed userOpHash, address indexed user)
    event GasBalanceDeducted(address indexed user, uint256 amount, uint256 premium, uint8 mode);
    event RefundProcessed(address indexed user, uint256 amount)
    // Entry point events
    event UserOperationEvent(bytes32 indexed userOpHash, address indexed sender, address indexed paymaster, uint256 nonce, bool success, uint256 actualGasCost, uint256 actualGasUsed)
}


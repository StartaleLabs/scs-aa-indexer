use alloy::{
    sol,
    primitives::{Address, U256},
};

sol! {
    event DepositAdded(address indexed user, uint256 amount);
    event GasBalanceDeducted(address indexed user, uint256 amount, uint256 premium, uint256 mode);
    event WithdrawalRequested(address indexed sponsorAddress, address indexed withdrawAddress, uint256 amount);
    event WithdrawalExecuted(address indexed sponsorAddress, address indexed withdrawAddress, uint256 amount);
}

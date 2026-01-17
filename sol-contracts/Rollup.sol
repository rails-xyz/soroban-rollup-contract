// SPDX-License-Identifier: MIT
pragma solidity 0.8.25;

import '@openzeppelin/contracts/token/ERC20/IERC20.sol';
import '@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol';
import '@openzeppelin/contracts/utils/ReentrancyGuard.sol';
import '@openzeppelin/contracts/access/Ownable2Step.sol';

/**
 * @title Rollup
 * @dev The Rollup contract allows users to deposit and withdraw collateral tokens. Owner can call rollup() to update Merkle Tree root, and increment withdrawal allowances and fees.
 */
contract Rollup is Ownable2Step, ReentrancyGuard {
  using SafeERC20 for IERC20;

  // State variable to store the latest block hash.
  bytes32 public latestBlockHash;

  // State variable to store the collateral token's instance.
  IERC20 private immutable collateralToken;

  // Mapping to keep track of each user's withdrawal allowance.
  mapping(address => uint256) public withdrawalAllowances;

  // Variable to keep track of the fees to be collected by the owner.
  uint256 public fees = 0;

  // Variable to track the mininum balance needed in the contract to fulfil fees and withdraws, e.g. totalWithdrawable = fees + sum(withdrawalAllowances).
  uint256 public totalWithdrawable = 0;

  // Event to notify when a user deposits collateral tokens.
  event Deposit(address indexed user, uint256 amount);

  // Event to notify when a user withdraw collateral tokens.
  event Withdrawal(address indexed user, uint256 amount);

  // Event to notify when owner publishes a new block and updates withdrawal allowances and fees.
  event NewBlock(bytes32 indexed newBlockHash, uint256 newWithdrawalSum, uint256 newFees);

  // Event to notify when owner collects the fees from the contract.
  event FeesCollected(address indexed to, uint256 amount);

  /**
   * @dev Constructor to initialize with the collateral token contract.
   * @param _collateralTokenAddress The address of the collateral token contract.
   */
  constructor(
    address _collateralTokenAddress
  ) Ownable(msg.sender) {
    collateralToken = IERC20(_collateralTokenAddress);
  }

  /**
   * @dev Deposit collateral tokens into the contract. Callable by any user.
   * @param user The address of the user depositing the collateral tokens.
   * @param amount The amount of collateral tokens to deposit.
   */
  function deposit(
    address user,
    uint256 amount
  ) external {
    // Require that the deposit amount is greater than zero
    require(amount > 0, 'Deposit amount must be greater than 0');
    // Transfer collateral tokens from the user's account to the contract.
    collateralToken.safeTransferFrom(user, address(this), amount);
    // Emit the deposit event.
    emit Deposit(user, amount);
  }

  /**
   * @dev Updates withdrawal allowances and fees, then emit the NewBlock event with a block hash. Only callable by the owner.
   * @param newBlockHash The block hash of the new block.
   * @param newWithdrawalAddresses The addresses for which the withdrawal allowances are to be updated.
   * @param newWithdrawalAmounts The withdrawal allowances for the respective addresses.
   * @param newWithdrawalSum The sum of the new withdrawal allowances.
   * @param newFees The new fees to be added to the contract.
   */
  function rollup(
    bytes32 oldBlockHash,
    bytes32 newBlockHash,
    address[] memory newWithdrawalAddresses,
    uint256[] memory newWithdrawalAmounts,
    uint256 newWithdrawalSum,
    uint256 newFees
  ) external onlyOwner {
    // Check if the new block hash is not empty.
    require(newBlockHash != 0, "New block hash cannot be empty");
    // Check if the old block hash matches the latest block hash to ensure the rollup order.
    require(oldBlockHash == latestBlockHash, "Old block hash does not match the latest block hash");
    // Check if the newblockHash is not the same as the old block hash to prevent duplicate rollups.
    require(newBlockHash != oldBlockHash, "New block hash cannot be the same as the old block hash");
    // Check if the new withdrawal addresses and amounts arrays have the same length.
    require(newWithdrawalAddresses.length == newWithdrawalAmounts.length, "Arrays must have the same length");
    // Check if array length doesn't cause excessive gas usage
    require(newWithdrawalAddresses.length <= 100, "Array length exceeds gas limit safety bounds");
    uint256 calculatedWithdrawalSum = 0;
    for (uint256 i = 0; i < newWithdrawalAddresses.length; i++) {
        address user = newWithdrawalAddresses[i];
        uint256 allowance = newWithdrawalAmounts[i];
        withdrawalAllowances[user] += allowance;
        calculatedWithdrawalSum += allowance;
    }
    require(calculatedWithdrawalSum == newWithdrawalSum, "Calculated withdrawal sum does not match the provided sum");
    // Calculate net new withdrawable = sum of new withdrawal allowances + new fees.
    uint256 netNewWithdrawable = newWithdrawalSum + newFees;
    // Check if the contract has sufficient balance for these new allowances + fees.
    require(netNewWithdrawable <= collateralToken.balanceOf(address(this)) - totalWithdrawable, "Insufficient balance");
    // Update states.
    totalWithdrawable += netNewWithdrawable;
    fees += newFees;
    // Update the latest block hash.
    latestBlockHash = newBlockHash;
    // Emit the NewBlock event.
    emit NewBlock(newBlockHash, newWithdrawalSum, newFees);
  }

  /**
   * @dev Withdraw collateral tokens from the contract. Callable by any user. Requires the user to have a withdrawalAllowance.
   */
  function withdraw() external nonReentrant {
    uint256 amount = withdrawalAllowances[msg.sender];
    // Check if the user has allowance to withdraw.
    require(
      amount > 0,
      'No withdrawal allowance available'
    );
    // Set the withdrawal allowance to 0 for the withdrawal.
    withdrawalAllowances[msg.sender] = 0;
    // Since withdrawalAllowances changed, update totalWithdrawable so that "totalWithdrawable = fees + sum(withdrawalAllowances)" holds.
    totalWithdrawable -= amount;
    // Transfer collateral tokens from the contract to the specified address.
    collateralToken.safeTransfer(msg.sender, amount);
    // Emit the withdrawal event.
    emit Withdrawal(msg.sender, amount);
  }

  /**
   * @dev Collect the fees from the contract. Only callable by the owner.
   * @param to The address to which the fees should be transferred.
   */
  function collectFees(address to) external onlyOwner nonReentrant {
    // Check if there are fees to collect.
    require(fees > 0, 'No fees to collect');
    // Store the fees in a local variable.
    uint256 amount = fees;
    // Reset the fees to 0 after collection.
    fees = 0;
    // Since fees is now 0, update totalWithdrawable so that "totalWithdrawable = fees + sum(withdrawalAllowances)" holds.
    totalWithdrawable -= amount;
    // Transfer the fees to the specified address.
    collateralToken.safeTransfer(to, amount);
    // Emit the fees collected event.
    emit FeesCollected(to, amount);
  }

  /**
    * @dev Recover any ERC20 tokens (except for the collateral token) sent to the contract. Only callable by the owner.
    * @param tokenAddress The address of the ERC20 token to recover.
    * @param to The address to which the ERC20 tokens should be transferred.
    */
  function recover(address tokenAddress, address to, uint256 amount) external onlyOwner {
    // Check if the token address is not the collateral token address.
    require(tokenAddress != address(collateralToken), 'Cannot recover collateral token');
    // Check if the amount is greater than 0.
    require(amount > 0, 'Amount must be greater than 0');
    // Recover the ERC20 tokens and transfer them to the specified address.
    IERC20 token = IERC20(tokenAddress);
    // Transfer the ERC20 tokens to the specified address.
    token.safeTransfer(to, amount);
  }

  /**
   * @dev Disables renouncing ownership by overriding it with revert
   */
  function renounceOwnership() public virtual override onlyOwner {
    revert("Renouncing ownership is disabled");
  }
}


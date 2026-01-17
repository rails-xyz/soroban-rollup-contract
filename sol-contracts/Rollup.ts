import { loadFixture } from '@nomicfoundation/hardhat-toolbox/network-helpers'
import { expect } from 'chai'
import { Signer } from 'ethers'
import hre from 'hardhat'
import { createHash } from 'crypto'

describe('Rollup Contract', () => {
  async function deployFixture() {
    // Contracts are deployed using the first signer/account by default
    const [owner, otherAccount, feeAccount] = await hre.ethers.getSigners()

    const ERC20Token = await hre.ethers.getContractFactory('ERC20Token')
    const erc20TokenContract = await ERC20Token.deploy()
    const decimals = await erc20TokenContract.decimals()

    function parseTokenUnits(value: string) {
      return hre.ethers.parseUnits(value, decimals)
    }

    // Define some numbers
    const transferAmount = parseTokenUnits('20')
    const depositAmount = parseTokenUnits('10')
    const withdrawalAmount = parseTokenUnits('5')
    const feeAmount = parseTokenUnits('0.5')

    // Deploy the contracts
    const Rollup = await hre.ethers.getContractFactory('Rollup')
    const rollupContract = await Rollup.deploy(erc20TokenContract.getAddress())

    // Transfer some erc20Token token to `otherAccount`
    await erc20TokenContract
      .connect(owner)
      .transfer(otherAccount, transferAmount)

    const balance = await erc20TokenContract.balanceOf(otherAccount.address)
    expect(balance).to.equal(transferAmount)

    async function approveSpending(account: Signer, amount: bigint) {
      await erc20TokenContract.connect(account).approve(rollupContract, amount)
    }

    // Define a mock root for the rollup
    const newRoot = createHash('sha256').update('mockRoot', 'utf8').digest()

    return {
      parseTokenUnits,
      rollupContract,
      erc20TokenContract,
      owner,
      otherAccount,
      depositAmount,
      feeAccount,
      feeAmount,
      withdrawalAmount,
      approveSpending,
      newRoot,
    }
  }

  describe('Deployment', () => {
    it('Should deploy the contracts successfully', async () => {
      const { rollupContract } = await loadFixture(deployFixture)
      // .to.not.be.undefined syntax is valid for hardhat
      // eslint-disable-next-line @typescript-eslint/no-unused-expressions
      expect(await rollupContract.getAddress()).to.not.be.undefined
    })
  })

  describe('Deposit', () => {
    it('Should allow USDC deposit', async () => {
      const {
        rollupContract,
        erc20TokenContract,
        otherAccount,
        depositAmount,
        approveSpending,
      } = await loadFixture(deployFixture)
      await approveSpending(otherAccount, depositAmount)
      await rollupContract
        .connect(otherAccount)
        .deposit(otherAccount.address, depositAmount)
      const contractBalance = await erc20TokenContract.balanceOf(rollupContract)
      expect(contractBalance).to.equal(depositAmount)
    })

    it('Should reject deposit of zero amount', async () => {
      const { rollupContract, otherAccount, approveSpending, parseTokenUnits } =
        await loadFixture(deployFixture)
      await approveSpending(otherAccount, parseTokenUnits('0'))
      await expect(
        rollupContract
          .connect(otherAccount)
          .deposit(otherAccount.address, parseTokenUnits('0'))
      ).to.be.revertedWith('Deposit amount must be greater than 0')
    })

    it('Should emit a "Deposit" event on successful deposit', async () => {
      const { rollupContract, otherAccount, depositAmount, approveSpending } =
        await loadFixture(deployFixture)

      await approveSpending(otherAccount, depositAmount)
      await expect(
        await rollupContract
          .connect(otherAccount)
          .deposit(otherAccount.address, depositAmount)
      ).to.emit(rollupContract, 'Deposit')
    })

    it('Should handle usdcToken.transfer failure in deposit', async () => {
      const { otherAccount, owner, depositAmount, approveSpending } =
        await loadFixture(deployFixture)
      const ERC20TokenFail =
        await hre.ethers.getContractFactory('ERC20TokenFail')
      const mockUSDCFail = await ERC20TokenFail.deploy()
      const Rollup = await hre.ethers.getContractFactory('Rollup')
      const rollupContract = await Rollup.deploy(mockUSDCFail)
      await mockUSDCFail
        .connect(owner)
        .transfer(otherAccount.address, depositAmount)
      await approveSpending(otherAccount, depositAmount)
      await expect(
        rollupContract
          .connect(otherAccount)
          .deposit(otherAccount.address, depositAmount)
      ).to.be.reverted
    })
  })

  describe('Rollup', () => {
    it('Should allow owner to rollup', async () => {
      const {
        otherAccount,
        rollupContract,
        depositAmount,
        withdrawalAmount,
        feeAmount,
        approveSpending,
        newRoot,
      } = await loadFixture(deployFixture)
      await approveSpending(otherAccount, depositAmount)
      await rollupContract
        .connect(otherAccount)
        .deposit(otherAccount.address, depositAmount)
      const oldRoot = await rollupContract.latestBlockHash()
      await rollupContract.rollup(
        oldRoot,
        newRoot,
        [otherAccount.address],
        [withdrawalAmount],
        withdrawalAmount,
        feeAmount
      )
      const withdrawalAllowance = await rollupContract
        .connect(otherAccount)
        .withdrawalAllowances(otherAccount.address)
      expect(withdrawalAllowance).to.equal(withdrawalAmount)
      expect(await rollupContract.fees()).to.equal(feeAmount)
    })

    it('Should not alow non-owner to rollup', async () => {
      const {
        otherAccount,
        rollupContract,
        withdrawalAmount,
        feeAmount,
        newRoot,
      } = await loadFixture(deployFixture)
      const oldRoot = await rollupContract.latestBlockHash()
      await expect(
        rollupContract
          .connect(otherAccount)
          .rollup(
            oldRoot,
            newRoot,
            [otherAccount.address],
            [withdrawalAmount],
            withdrawalAmount,
            feeAmount
          )
      ).to.be.revertedWithCustomError(
        rollupContract,
        'OwnableUnauthorizedAccount'
      )
    })

    it('Should allow rollup of zero amount', async () => {
      const { rollupContract, newRoot } = await loadFixture(deployFixture)
      const oldRoot = await rollupContract.latestBlockHash()
      await expect(rollupContract.rollup(oldRoot, newRoot, [], [], 0, 0)).not.to
        .be.reverted
    })

    it('Should reject rollup with withdrawal beyond contract balance', async () => {
      const { rollupContract, owner, depositAmount, newRoot, parseTokenUnits } =
        await loadFixture(deployFixture)
      const moreThanTotalDepositAmount = depositAmount + parseTokenUnits('1')
      const oldRoot = await rollupContract.latestBlockHash()
      await expect(
        rollupContract.rollup(
          oldRoot,
          newRoot,
          [owner.address],
          [moreThanTotalDepositAmount],
          moreThanTotalDepositAmount,
          0
        )
      ).to.be.revertedWith('Insufficient balance')
    })

    it('Should reject rollup with fees beyond contract balance', async () => {
      const { rollupContract, depositAmount, newRoot, parseTokenUnits } =
        await loadFixture(deployFixture)
      const moreThanTotalDepositAmount = depositAmount + parseTokenUnits('1')
      const oldRoot = await rollupContract.latestBlockHash()
      await expect(
        rollupContract.rollup(
          oldRoot,
          newRoot,
          [],
          [],
          0,
          moreThanTotalDepositAmount
        )
      ).to.be.revertedWith('Insufficient balance')
    })

    it('Should reject rollup with no block hash', async () => {
      const { rollupContract, depositAmount, parseTokenUnits } =
        await loadFixture(deployFixture)
      const moreThanTotalDepositAmount = depositAmount + parseTokenUnits('1')
      const oldRoot = await rollupContract.latestBlockHash()
      await expect(
        rollupContract.rollup(
          oldRoot,
          hre.ethers.zeroPadBytes(new Uint8Array(), 32),
          [],
          [],
          0,
          moreThanTotalDepositAmount
        )
      ).to.be.revertedWith('New block hash cannot be empty')
    })

    it('Should reject rollup with same root', async () => {
      const { rollupContract, newRoot } = await loadFixture(deployFixture)
      const oldRoot = await rollupContract.latestBlockHash()
      await expect(rollupContract.rollup(oldRoot, newRoot, [], [], 0, 0)).not.to
        .be.reverted
      const wrongRoot = createHash('sha256')
        .update('wrongRoot', 'utf8')
        .digest()
      const anotherNewRoot = createHash('sha256')
        .update('anotherNewRoot', 'utf8')
        .digest()
      await expect(
        rollupContract.rollup(wrongRoot, anotherNewRoot, [], [], 0, 0)
      ).to.be.revertedWith(
        'Old block hash does not match the latest block hash'
      )
    })

    it('Should reject rollup with incorrect old root', async () => {
      const { rollupContract, newRoot } = await loadFixture(deployFixture)
      const oldRoot = await rollupContract.latestBlockHash()
      await expect(rollupContract.rollup(oldRoot, newRoot, [], [], 0, 0)).not.to
        .be.reverted
      await expect(
        rollupContract.rollup(newRoot, newRoot, [], [], 0, 0)
      ).to.be.revertedWith(
        'New block hash cannot be the same as the old block hash'
      )
    })

    it('Should reject rollup if address array and withdrawal array have different length', async () => {
      const {
        rollupContract,
        depositAmount,
        newRoot,
        otherAccount,
        withdrawalAmount,
      } = await loadFixture(deployFixture)
      const oldRoot = await rollupContract.latestBlockHash()
      await expect(
        rollupContract.rollup(
          oldRoot,
          newRoot,
          [otherAccount.address],
          [withdrawalAmount, withdrawalAmount],
          withdrawalAmount,
          depositAmount
        )
      ).to.be.revertedWith('Arrays must have the same length')
    })

    it('Should reject rollup if newWithdrawalAmounts does not addup newWithdrawalSum', async () => {
      const {
        rollupContract,
        depositAmount,
        newRoot,
        otherAccount,
        withdrawalAmount,
        parseTokenUnits,
      } = await loadFixture(deployFixture)
      const incorrectWithdrawalSum = withdrawalAmount + parseTokenUnits('1')
      const oldRoot = await rollupContract.latestBlockHash()
      await expect(
        rollupContract.rollup(
          oldRoot,
          newRoot,
          [otherAccount.address],
          [incorrectWithdrawalSum],
          withdrawalAmount,
          depositAmount
        )
      ).to.be.revertedWith(
        'Calculated withdrawal sum does not match the provided sum'
      )
    })

    it('Should reject rollup with too many entries', async () => {
      const { rollupContract, newRoot, parseTokenUnits } =
        await loadFixture(deployFixture)
      const oldRoot = await rollupContract.latestBlockHash()

      // Create arrays with 101 entries
      const addresses = Array(101).fill(hre.ethers.ZeroAddress)
      const amounts = Array(101).fill(parseTokenUnits('0.1'))

      await expect(
        rollupContract.rollup(
          oldRoot,
          newRoot,
          addresses,
          amounts,
          parseTokenUnits('10.1'),
          0
        )
      ).to.be.revertedWith('Array length exceeds gas limit safety bounds')
    })

    it('Should measure gas usage for different array sizes', async () => {
      const {
        rollupContract,
        newRoot,
        otherAccount,
        depositAmount,
        approveSpending,
        parseTokenUnits,
      } = await loadFixture(deployFixture)
      await approveSpending(otherAccount, depositAmount)
      await rollupContract
        .connect(otherAccount)
        .deposit(otherAccount.address, depositAmount)
      const oldRoot = await rollupContract.latestBlockHash()

      // Test with small array (5 entries)
      const smallAddresses = Array(5).fill(
        hre.ethers.Wallet.createRandom().address
      )
      const smallAmounts = Array(5).fill(parseTokenUnits('0.1'))
      const smallTx = await rollupContract.rollup.estimateGas(
        oldRoot,
        newRoot,
        smallAddresses,
        smallAmounts,
        parseTokenUnits('0.5'),
        0
      )

      // Test with medium array (50 entries)
      const mediumAddresses = Array(50).fill(
        hre.ethers.Wallet.createRandom().address
      )
      const mediumAmounts = Array(50).fill(parseTokenUnits('0.1'))
      const mediumTx = await rollupContract.rollup.estimateGas(
        oldRoot,
        newRoot,
        mediumAddresses,
        mediumAmounts,
        parseTokenUnits('5'),
        0
      )

      // Test with large array (100 entries)
      const largeAddresses = Array(100).fill(
        hre.ethers.Wallet.createRandom().address
      )
      const largeAmounts = Array(100).fill(parseTokenUnits('0.1'))
      const largeTx = await rollupContract.rollup.estimateGas(
        oldRoot,
        newRoot,
        largeAddresses,
        largeAmounts,
        parseTokenUnits('10'),
        0
      )

      // Verify gas usage scales reasonably
      expect(Number(mediumTx)).to.be.greaterThan(Number(smallTx))
      expect(Number(largeTx)).to.be.greaterThan(Number(mediumTx))

      // Verify all operations are within block gas limit
      const block = await hre.ethers.provider.getBlock('latest')
      expect(Number(largeTx)).to.be.lessThan(Number(block?.gasLimit))

      // Log gas usage for info
      console.log(`Gas used for 5 withdrawal entries: ${smallTx}`)
      console.log(`Gas used for 50 withdrawal entries: ${mediumTx}`)
      console.log(`Gas used for 100 withdrawal entries: ${largeTx}`)
    })

    it('Should emit a "NewBlock" event on successful rollup', async () => {
      const {
        otherAccount,
        rollupContract,
        depositAmount,
        withdrawalAmount,
        feeAmount,
        approveSpending,
        newRoot,
      } = await loadFixture(deployFixture)
      await approveSpending(otherAccount, depositAmount)
      await rollupContract
        .connect(otherAccount)
        .deposit(otherAccount.address, depositAmount)
      const oldRoot = await rollupContract.latestBlockHash()
      await expect(
        rollupContract.rollup(
          oldRoot,
          newRoot,
          [otherAccount.address],
          [withdrawalAmount],
          withdrawalAmount,
          feeAmount
        )
      ).to.emit(rollupContract, 'NewBlock')
    })

    it('Should handle specific rollup with multiple withdrawals', async () => {
      const { rollupContract, owner, parseTokenUnits, approveSpending } =
        await loadFixture(deployFixture)

      // Convert hex string to bytes
      const newBlockHash =
        '0x11b2937d24a2ef385fc62d575f5c6b7e39b52067a64606738f7e69cb94828af8'

      // Setup withdrawal addresses from the JSON data
      const withdrawalAddresses = [
        '0x247B13dB64571963DCC633D3b41B11BfBA0AAe49',
        '0xedD7cd76DBBD01Db7C43bC5576Ae8D5E34f5626c',
      ]

      // Setup withdrawal amounts from the JSON data
      const withdrawalAmounts = [
        parseTokenUnits('529.97'),
        parseTokenUnits('498.45'),
      ]

      // Calculate total withdrawal amount and fees
      const totalWithdrawalAmount = parseTokenUnits('1028.42')
      const feeAmount = parseTokenUnits('8.09411834')

      // Deposit enough tokens to the contract to cover withdrawals and fees
      const depositAmount = totalWithdrawalAmount + feeAmount
      await approveSpending(owner, depositAmount)
      await rollupContract.connect(owner).deposit(owner.address, depositAmount)

      // Get the current block hash
      const oldRoot = await rollupContract.latestBlockHash()

      // Execute the rollup
      await rollupContract.rollup(
        oldRoot,
        newBlockHash,
        withdrawalAddresses,
        withdrawalAmounts,
        totalWithdrawalAmount,
        feeAmount
      )

      // Verify withdrawal allowances were set correctly
      expect(
        await rollupContract.withdrawalAllowances(withdrawalAddresses[0])
      ).to.equal(withdrawalAmounts[0])
      expect(
        await rollupContract.withdrawalAllowances(withdrawalAddresses[1])
      ).to.equal(withdrawalAmounts[1])

      // Verify fees were collected
      expect(await rollupContract.fees()).to.equal(feeAmount)

      // Verify total withdrawals match the sum of individual withdrawals
      const sum = withdrawalAmounts[0] + withdrawalAmounts[1]
      expect(sum).to.equal(totalWithdrawalAmount)
    })
  })

  describe('Withdraw', () => {
    it('Should allow users to withdraw their withdrawalAllowance', async () => {
      const {
        rollupContract,
        newRoot,
        otherAccount,
        depositAmount,
        erc20TokenContract,
        withdrawalAmount,
        approveSpending,
      } = await loadFixture(deployFixture)
      await approveSpending(otherAccount, depositAmount)
      await rollupContract
        .connect(otherAccount)
        .deposit(otherAccount.address, depositAmount)
      const userStartingBalance = await erc20TokenContract.balanceOf(
        otherAccount.address
      )
      const contractStartingBalance =
        await erc20TokenContract.balanceOf(rollupContract)
      const startingWithdrawalAllowance =
        await rollupContract.withdrawalAllowances(otherAccount)
      expect(startingWithdrawalAllowance).to.equal(0)
      const oldRoot = await rollupContract.latestBlockHash()
      await rollupContract.rollup(
        oldRoot,
        newRoot,
        [otherAccount.address],
        [withdrawalAmount],
        withdrawalAmount,
        0
      )
      const withdrawalAllowance =
        await rollupContract.withdrawalAllowances(otherAccount)
      await rollupContract.connect(otherAccount).withdraw()
      const contractBalanceAfterWithdrawal =
        await erc20TokenContract.balanceOf(rollupContract)
      expect(contractBalanceAfterWithdrawal).to.equal(
        contractStartingBalance - withdrawalAmount
      )
      const userBalance = await erc20TokenContract.balanceOf(
        otherAccount.address
      )
      expect(userBalance).to.equal(userStartingBalance + withdrawalAmount)
      const withdrawalAllowanceAfterWithdrawal =
        await rollupContract.withdrawalAllowances(otherAccount)
      expect(withdrawalAllowanceAfterWithdrawal).to.equal(
        withdrawalAllowance - withdrawalAmount
      )
    })

    it('Should reject withdrawal without withdrawalAllowance', async () => {
      const { rollupContract, otherAccount, depositAmount, approveSpending } =
        await loadFixture(deployFixture)
      await approveSpending(otherAccount, depositAmount)
      await rollupContract
        .connect(otherAccount)
        .deposit(otherAccount.address, depositAmount)
      await expect(
        rollupContract.connect(otherAccount).withdraw()
      ).to.be.revertedWith('No withdrawal allowance available')
    })

    it('Should emit a "Withdrawal" event on successful withdrawal', async () => {
      const {
        rollupContract,
        otherAccount,
        depositAmount,
        withdrawalAmount,
        approveSpending,
        newRoot,
      } = await loadFixture(deployFixture)
      await approveSpending(otherAccount, depositAmount)
      await rollupContract
        .connect(otherAccount)
        .deposit(otherAccount.address, depositAmount)
      const oldRoot = await rollupContract.latestBlockHash()
      await rollupContract.rollup(
        oldRoot,
        newRoot,
        [otherAccount.address],
        [withdrawalAmount],
        withdrawalAmount,
        0
      )
      await expect(
        await rollupContract.connect(otherAccount).withdraw()
      ).to.emit(rollupContract, 'Withdrawal')
    })
  })

  describe('Fees', async () => {
    it('Should allow owner to collect fees to feeAccount', async () => {
      const {
        rollupContract,
        owner,
        otherAccount,
        feeAccount,
        depositAmount,
        withdrawalAmount,
        feeAmount,
        erc20TokenContract,
        approveSpending,
        newRoot,
      } = await loadFixture(deployFixture)
      await approveSpending(otherAccount, depositAmount)
      await rollupContract
        .connect(otherAccount)
        .deposit(otherAccount.address, depositAmount)
      const oldRoot = await rollupContract.latestBlockHash()
      await rollupContract.rollup(
        oldRoot,
        newRoot,
        [otherAccount.address],
        [withdrawalAmount],
        withdrawalAmount,
        feeAmount
      )
      const feeAccountStartingBalance =
        await erc20TokenContract.balanceOf(feeAccount)
      expect(feeAccountStartingBalance).to.equal(0)
      await rollupContract.connect(owner).collectFees(feeAccount)
      const feeAccountBalance = await erc20TokenContract.balanceOf(feeAccount)
      expect(feeAccountBalance).to.equal(feeAccountStartingBalance + feeAmount)
    })

    it('Should not allow non-owner to collect fees', async () => {
      const {
        rollupContract,
        otherAccount,
        depositAmount,
        withdrawalAmount,
        feeAmount,
        approveSpending,
        newRoot,
      } = await loadFixture(deployFixture)
      await approveSpending(otherAccount, depositAmount)
      await rollupContract
        .connect(otherAccount)
        .deposit(otherAccount.address, depositAmount)
      const oldRoot = await rollupContract.latestBlockHash()
      await rollupContract.rollup(
        oldRoot,
        newRoot,
        [otherAccount.address],
        [withdrawalAmount],
        withdrawalAmount,
        feeAmount
      )
      await expect(
        rollupContract.connect(otherAccount).collectFees(otherAccount)
      ).to.be.revertedWithCustomError(
        rollupContract,
        'OwnableUnauthorizedAccount'
      )
    })

    it('Should reject fee collection if there is no fees to collect', async () => {
      const { rollupContract, owner } = await loadFixture(deployFixture)
      await expect(
        rollupContract.connect(owner).collectFees(owner)
      ).to.be.revertedWith('No fees to collect')
    })

    it('Should emit a "FeesCollected" event on successful fee collection', async () => {
      const {
        rollupContract,
        owner,
        otherAccount,
        depositAmount,
        withdrawalAmount,
        feeAmount,
        approveSpending,
        newRoot,
      } = await loadFixture(deployFixture)
      await approveSpending(otherAccount, depositAmount)
      await rollupContract
        .connect(otherAccount)
        .deposit(otherAccount.address, depositAmount)
      const oldRoot = await rollupContract.latestBlockHash()
      await rollupContract.rollup(
        oldRoot,
        newRoot,
        [otherAccount.address],
        [withdrawalAmount],
        withdrawalAmount,
        feeAmount
      )
      await expect(rollupContract.connect(owner).collectFees(owner)).to.emit(
        rollupContract,
        'FeesCollected'
      )
    })
  })

  describe('Recover', () => {
    it('Should allow owner to recover non USDC tokens', async () => {
      const { rollupContract, owner, otherAccount, parseTokenUnits } =
        await loadFixture(deployFixture)
      const ERC20Token = await hre.ethers.getContractFactory('ERC20Token')
      const otherERC20 = await ERC20Token.connect(otherAccount).deploy()
      await otherERC20
        .connect(otherAccount)
        .approve(rollupContract, parseTokenUnits('100'))
      await otherERC20
        .connect(otherAccount)
        .transfer(rollupContract, parseTokenUnits('100'))
      await rollupContract
        .connect(owner)
        .recover(otherERC20, owner, parseTokenUnits('100'))
      const ownerBalance = await otherERC20.balanceOf(owner.address)
      expect(ownerBalance).to.equal(parseTokenUnits('100'))
    })

    it('Should reject recover with 0 amount', async () => {
      const { rollupContract, owner, parseTokenUnits } =
        await loadFixture(deployFixture)
      const ERC20Token = await hre.ethers.getContractFactory('ERC20Token')
      const mockERC20 = await ERC20Token.deploy()
      expect(
        rollupContract
          .connect(owner)
          .recover(mockERC20, owner, parseTokenUnits('0'))
      ).to.be.revertedWith('Amount must be greater than 0')
    })

    it('Should reject when recover amount exceed balance', async () => {
      const { rollupContract, owner, otherAccount, parseTokenUnits } =
        await loadFixture(deployFixture)
      const ERC20Token = await hre.ethers.getContractFactory('ERC20Token')
      const mockERC20 = await ERC20Token.deploy()
      await mockERC20
        .connect(owner)
        .transfer(otherAccount, parseTokenUnits('100'))
      await mockERC20
        .connect(otherAccount)
        .approve(rollupContract, parseTokenUnits('100'))
      await mockERC20
        .connect(otherAccount)
        .transfer(rollupContract, parseTokenUnits('100'))
      expect(
        rollupContract
          .connect(owner)
          .recover(mockERC20, owner, parseTokenUnits('120'))
      ).to.be.revertedWithCustomError(mockERC20, 'ERC20InsufficientBalance')
    })

    it('Should not allow non-owner to recover non USDC tokens', async () => {
      const { rollupContract, otherAccount, parseTokenUnits } =
        await loadFixture(deployFixture)
      const ERC20Token = await hre.ethers.getContractFactory('ERC20Token')
      const otherERC20 = await ERC20Token.connect(otherAccount).deploy()
      await otherERC20
        .connect(otherAccount)
        .approve(rollupContract, parseTokenUnits('100'))
      await otherERC20
        .connect(otherAccount)
        .transfer(rollupContract, parseTokenUnits('100'))
      await expect(
        rollupContract
          .connect(otherAccount)
          .recover(otherERC20, otherAccount, parseTokenUnits('100'))
      ).to.be.revertedWithCustomError(
        rollupContract,
        'OwnableUnauthorizedAccount'
      )
    })

    it('Should not allow owner to recover USDC tokens', async () => {
      const {
        rollupContract,
        owner,
        otherAccount,
        erc20TokenContract,
        parseTokenUnits,
      } = await loadFixture(deployFixture)
      await erc20TokenContract
        .connect(owner)
        .transfer(otherAccount, parseTokenUnits('100'))
      await erc20TokenContract
        .connect(otherAccount)
        .approve(rollupContract, parseTokenUnits('100'))
      await erc20TokenContract
        .connect(otherAccount)
        .transfer(rollupContract, parseTokenUnits('100'))
      expect(
        rollupContract
          .connect(owner)
          .recover(erc20TokenContract, owner, parseTokenUnits('100'))
      ).to.be.revertedWith('Cannot recover collateral token')
    })
  })

  describe('Ownership', () => {
    it('Should not allow owner to renounce ownership', async () => {
      const { rollupContract, owner } = await loadFixture(deployFixture)
      await expect(
        rollupContract.connect(owner).renounceOwnership()
      ).to.be.revertedWith('Renouncing ownership is disabled')
    })

    it('Should follow two-step ownership transfer process', async () => {
      const { rollupContract, owner, otherAccount } =
        await loadFixture(deployFixture)

      // Initial owner should be the deployer
      expect(await rollupContract.owner()).to.equal(owner.address)

      // Owner transfers ownership
      await rollupContract
        .connect(owner)
        .transferOwnership(otherAccount.address)

      // Owner should still be the original owner until acceptance
      expect(await rollupContract.owner()).to.equal(owner.address)

      // Only pending owner can accept ownership
      await expect(
        rollupContract.connect(owner).acceptOwnership()
      ).to.be.revertedWithCustomError(
        rollupContract,
        'OwnableUnauthorizedAccount'
      )

      // New owner accepts ownership
      await rollupContract.connect(otherAccount).acceptOwnership()

      // Owner should now be the new owner
      expect(await rollupContract.owner()).to.equal(otherAccount.address)
    })
  })
})

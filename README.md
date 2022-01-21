# Strange transaction failure with ethers-rs and geth

- Works with hardhat node
- Works with a similar example in typescript (below) and geth
- Works with ethers-rs if a contract state modification is done after the `transferFrom`.

To run

```shell
nix develop # or install hardhat, cargo in other way
hardhat compile
bin/build-abi
cargo run 
```

Equivalent typescript code
```typescript
import { expect } from "chai";
import { ethers, getNamedAccounts } from "hardhat";
import { Deposit, SimpleToken } from "../typechain-types";

describe("Deposit", function() {
  describe("deposit", async function() {
    let token: SimpleToken;
    let deposit: Deposit;

    beforeEach(async function() {
      token = await (await ethers.getContractFactory("SimpleToken")).deploy();
      deposit = await (await ethers.getContractFactory("Deposit")).deploy();
    });

    it("can deposit", async function() {

      const provider = ethers.provider;
      let alice = ethers.Wallet.createRandom();
      alice = alice.connect(provider);
      const deployer = (await getNamedAccounts()).deployer;
      const signer = await ethers.getSigner(deployer);

      let tx = await signer.sendTransaction({
        to: alice.address,
        value: ethers.utils.parseEther("1.0")
      });
      await tx.wait();
      console.log('funded');

      tx = await token.transfer(alice.address, 1000);
      await tx.wait();
      console.log('token funded');


      const aliceToken = token.connect(alice);
      tx = await aliceToken.approve(deposit.address, 1000);
      await tx.wait();
      console.log('approved');

      const aliceDeposit = deposit.connect(alice);
      tx = await aliceDeposit.deposit(token.address, 1000);
      await tx.wait();
      const receipt = await provider.getTransactionReceipt(tx.hash);
      expect(receipt.status).to.eq(1);

      expect(await token.balanceOf(deposit.address)).to.eq(1000);
    });

  });
});
```

`
This project demonstrates a basic Hardhat use case. It comes with a sample contract, a test for that contract, a sample script that deploys that contract, and an example of a task implementation, which simply lists the available accounts.

Try running some of the following tasks:

```shell
npx hardhat accounts
npx hardhat compile
npx hardhat clean
npx hardhat test
npx hardhat node
node scripts/sample-script.js
npx hardhat help
```

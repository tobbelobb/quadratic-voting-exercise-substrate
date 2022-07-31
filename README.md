# Quadratic Voting Exercise Substrate
A learning project to get me into Substrate development. A simple quadratic voting system pallet for Substrate.

## User Stories & Development Status

 - [ ] Use FRAME Identity pallet
 - [ ] Users (IDs) can vote by reserving tokens
 - [ ] Votes = sqrt(reserved tokens)
 - [ ] Proposals: on-chain hashes
 - [ ] Simple case voting alternatives: Aye or Nay
 - [ ] Bonus: Proposing and voting on multiple things at once

## Done
 - [x] Make reservable currency available to the qv pallet
 - [x] Make identity pallet available to the qv pallet
 - [x] Reserve tokens function exists
   - [x] Test it
 - [ ] Represent referendums
 - [ ] Weigh reserved tokens on quadratic scale

## Reading List

 - [Wikipedia: Quadratic Voting](https://en.wikipedia.org/wiki/Quadratic_voting)
 - [Quadratic Voting, Lalley, Weyl (2014)](https://www.aeaweb.org/conference/2015/retrieve.php?pdfid=3009&tk=BHDG8H2E)
 - [Substrate DID Pallet (Demo Project)](https://github.com/substrate-developer-hub/pallet-did)
 - [Polkadot Wiki: Identity](https://wiki.polkadot.network/docs/learn-identity)
 - [FRAME Pallet Identity](https://paritytech.github.io/substrate/master/pallet_identity/index.html)
 - [FRAME Pallet Democracy](https://paritytech.github.io/substrate/master/pallet_democracy/index.html), [on
   Github](https://github.com/paritytech/substrate/tree/master/frame/democracy)
 - [Substrate Quadratic Democracy (2020)](https://github.com/MVPWorkshop/substrate-quadratic-democracy),
   [discussion](https://github.com/substrate-developer-hub/hacktoberfest/issues/22)
 - [Pallet-Quadratic-Funding](https://github.com/jakehemmerle/uc-zk-voting)
 - [How to Build Custom Pallets with Substrate](https://learn.figment.io/tutorials/how-to-build-custom-pallets-with-substrate)
 - [Alternative Identity Pallet](https://github.com/sunshine-protocol/sunshine-keybase)

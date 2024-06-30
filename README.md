# Simple Voting pallet
The `pallet-voting` is a Substrate pallet that implements a decentralized voting system. It allows users to create proposals, vote on them, and manage the voting process on the blockchain.

### Features

- **Create Proposals**: Users can create new proposals by submitting a description and a duration for the voting period.
- **Vote on Proposals**: Users can cast votes on active proposals. Each user can only vote once per proposal. Votes can be either "Yes" or "No".
- **Finalize Proposals**: Once the voting period for a proposal ends, the results are finalized automatically. The proposal is marked as approved if the majority of votes are "Yes" or true otherwise, it is rejected.

## Storage

- `ProposalCount`: Stores the number of proposals.
- `Proposals`: Stores the details of each proposal.
- `Votes`: Stores the votes for each proposal.
- `NextProposalId`: Provides for the next proposal ID.
- `ProposalsToFinalize`: Stores the proposals to be finalized at a specific block.

## Events

- `ProposalCreated`: Emitted when a new proposal is created.
- `VoteCast`: Emitted when a vote is cast on a proposal.
- `ProposalFinalized`: Emitted when a proposal is finalized.

## Errors

- `ProposalNotFound`: The specified proposal does not exist.
- `VotingPeriodEnded`: The voting period for the proposal has ended.
- `AlreadyVoted`: The user has already voted on the proposal.
- `VotingPeriodNotEnded`: The voting period for the proposal has not ended.
- `DescriptionTooLong`: The description of the proposal is too long.
- `TooManyProposalsInBlock`: There are too many proposals in a block.


## Benchmarking

The pallet has been properly benchmarked according to the standard procedures.

## Tests

```sh
cargo build --release
```

## Build

Use the following command to build the node without launching it:

```sh
cargo build --release
```

## Single-Node Development Chain

The following command starts a single-node development chain that doesn't
persist state:

```sh
./target/release/node-template --dev --alice -d data/alice
```

To purge the development chain's state, just remove the /data folder in the project root directory.

## Frontend

To start the frontend emplate for the pallet interactor, do the following:

```sh
cd frontend
npm install
npm start
```

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;


#[cfg(test)]
#[path = "mock.rs"]
mod mock;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, BoundedVec};
    use frame_system::pallet_prelude::*;
    use scale_info::prelude::vec::Vec;
    pub use frame_system::pallet_prelude::BlockNumberFor;
    use scale_info::TypeInfo;
	use crate::weights::WeightInfo;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        #[pallet::constant]
        type MaxDescriptionLength: Get<u32>;
		#[pallet::constant]
        type MaxProposalsPerBlock: Get<u32>;

		/// A type representing the weights required by the dispatchables of this pallet.
		type WeightInfo: WeightInfo;
    }

    #[pallet::storage]
    #[pallet::getter(fn proposal_count)]
    pub type ProposalCount<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn proposals)]
    pub type Proposals<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32,
        Proposal<T::AccountId, BlockNumberFor<T>, BoundedVec<u8, T::MaxDescriptionLength>>,
        OptionQuery
    >;

    #[pallet::storage]
    #[pallet::getter(fn votes)]
    pub type Votes<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        (u32, T::AccountId),
        bool,
        OptionQuery
    >;

    #[pallet::type_value]
    pub fn DefaultForNextProposalId() -> u32 { 1 }

    #[pallet::storage]
    #[pallet::getter(fn next_proposal_id)]
    pub type NextProposalId<T> = StorageValue<_, u32, ValueQuery, DefaultForNextProposalId>;

	#[pallet::storage] // block => Vec<proposal ids>
    #[pallet::getter(fn proposals_to_finalize)]
    pub type ProposalsToFinalize<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BlockNumberFor<T>,
		BoundedVec<u32, T::MaxProposalsPerBlock>,
        ValueQuery
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ProposalCreated{
			proposal_id: u32,
			creator: T::AccountId,
			description: Vec<u8>,
			begin_block: BlockNumberFor<T>,
			end_block: BlockNumberFor<T>,
			yes_votes: u32,
			no_votes: u32,
			voting_ended: bool,
		},
        VoteCast(u32, T::AccountId, bool),
        ProposalFinalized(u32, bool),
    }

    #[pallet::error]
    pub enum Error<T> {
        ProposalNotFound,
        VotingPeriodEnded,
        AlreadyVoted,
        VotingPeriodNotEnded,
        DescriptionTooLong,
		TooManyProposalsInBlock
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::create_proposal(description.len() as u32))]
        pub fn create_proposal(
            origin: OriginFor<T>,
            description: Vec<u8>,
            duration: BlockNumberFor<T> // Use BlockNumberFor<T>
        ) -> DispatchResult {
            let creator = ensure_signed(origin)?;
            let bounded_description: BoundedVec<u8, T::MaxDescriptionLength> = description.try_into().map_err(|_| Error::<T>::DescriptionTooLong)?;
            let proposal_id = Self::next_proposal_id();
            let current_block = <frame_system::Pallet<T>>::block_number();
			let end_block = current_block + duration;

            let proposal = Proposal {
                id: proposal_id,
                creator: creator.clone(),
                description: bounded_description,
                begin_block: current_block,
                end_block,
                yes_votes: 0,
                no_votes: 0,
                voting_ended: false,
            };

			// Storing
            <Proposals<T>>::insert(proposal_id, proposal.clone());
            <NextProposalId<T>>::put(proposal_id + 1);
			<ProposalsToFinalize<T>>::mutate(proposal.end_block, |proposal_ids| {
				if proposal_ids.try_push(proposal_id).is_err() {
					return Err(Error::<T>::TooManyProposalsInBlock);
				}
				Ok(())
			})?;

            Self::deposit_event(Event::ProposalCreated{
				proposal_id: proposal.id,
				creator,
				description: proposal.description.into(),
                begin_block: current_block,
                end_block,
                yes_votes: 0,
                no_votes: 0,
                voting_ended: false,
			});
            Ok(())
        }

        #[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::vote(*proposal_id))]
        pub fn vote(
            origin: OriginFor<T>,
            proposal_id: u32,
            vote: bool
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(<Proposals<T>>::contains_key(proposal_id), Error::<T>::ProposalNotFound);

            let proposal = <Proposals<T>>::get(proposal_id).unwrap();
            let current_block = <frame_system::Pallet<T>>::block_number();
            ensure!(current_block <= proposal.end_block, Error::<T>::VotingPeriodEnded);
            ensure!(!<Votes<T>>::contains_key((proposal_id, who.clone())), Error::<T>::AlreadyVoted);

            if vote {
                <Proposals<T>>::mutate(proposal_id, |p| {
                    if let Some(p) = p {
                        p.yes_votes += 1;
                    }
                });
            } else {
                <Proposals<T>>::mutate(proposal_id, |p| {
                    if let Some(p) = p {
                        p.no_votes += 1;
                    }
                });
            }

            <Votes<T>>::insert((proposal_id, who.clone()), vote);
            Self::deposit_event(Event::VoteCast(proposal_id, who, vote));
            Ok(())
        }
    }

	#[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(now: BlockNumberFor<T>) -> Weight {
			let block_actions = Self::block_actions(now);
			match block_actions {
				Ok(_) => {
					T::WeightInfo::on_initialize()
				}
				Err(e) => {
					log::error!("Error while executing block action: {:?}", e);
					T::WeightInfo::on_initialize()
				}
			}
		}
    }

    #[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen)]
    pub struct Proposal<AccountId, BlockNumber, Description> {
        pub id: u32,
        pub creator: AccountId,
        pub description: Description,
        pub begin_block: BlockNumber,
        pub end_block: BlockNumber,
        pub yes_votes: u32,
        pub no_votes: u32,
        pub voting_ended: bool,
    }
}

use frame_support::dispatch::DispatchResult;
impl <T: Config> Pallet<T> {
	pub fn block_actions(now: BlockNumberFor<T>) -> DispatchResult {
        // Get the proposals to finalize for the current block
		let mut proposal_ids = <ProposalsToFinalize<T>>::get(now);
		// Retain only the unfinalized proposals
		proposal_ids.retain(|&proposal_id| {
			if let Some(mut proposal) = <Proposals<T>>::get(proposal_id) {
				if !proposal.voting_ended {
					let approved = proposal.yes_votes > proposal.no_votes;
					proposal.voting_ended = true;
					<Proposals<T>>::insert(proposal_id, proposal);
					Self::deposit_event(Event::ProposalFinalized(proposal_id, approved));
					return false; // Remove finalized proposal
				}
			}
			true // Retain unfinalized proposal
		});

		// Update the storage
		if proposal_ids.is_empty() {
			<ProposalsToFinalize<T>>::remove(now);
		} else {
			<ProposalsToFinalize<T>>::insert(now, proposal_ids);
		}

        Ok(())
    }
}

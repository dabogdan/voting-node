#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
pub use frame_system::pallet_prelude::*;
use frame_support::traits::Get;
use scale_info::prelude::vec;

benchmarks! {
    create_proposal {
        let d in 0 .. T::MaxDescriptionLength::get();
        let description = vec![0; d as usize];
        let caller: T::AccountId = whitelisted_caller();
    }: _(RawOrigin::Signed(caller), description, 100u32.into())
    verify {
        assert_eq!(NextProposalId::<T>::get(), 2);
    }

    vote {
        let d in 0 .. T::MaxDescriptionLength::get();
        let description = vec![0; d as usize];
        let caller: T::AccountId = whitelisted_caller();
        Pallet::<T>::create_proposal(RawOrigin::Signed(caller.clone()).into(), description, 100u32.into())?;
    }: _(RawOrigin::Signed(caller), 1, true)
    verify {
        let proposal = Proposals::<T>::get(1).unwrap();
        assert_eq!(proposal.yes_votes, 1);
    }

	on_initialize {
        let max_length: u32 = T::MaxDescriptionLength::get();
        let description = vec![0u8; max_length as usize];
        let caller: T::AccountId = whitelisted_caller();
        for _ in 0..100 {
            Pallet::<T>::create_proposal(RawOrigin::Signed(caller.clone()).into(), description.clone(), 1u32.into())?;
        }

        frame_system::Pallet::<T>::set_block_number(1u32.into());
    }: {
        <Pallet<T> as frame_support::traits::Hooks<BlockNumberFor<T>>>::on_initialize(1u32.into());
    }
    verify {
        for i in 1..=100 {
            let proposal = Proposals::<T>::get(i).unwrap();
            assert!(proposal.voting_ended);
        }
    }

	impl_benchmark_test_suite!(VotingModule, crate::mock::new_test_ext(), crate::mock::Test);
}

#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::assert_ok;

    #[test]
    fn test_benchmarks() {
        new_test_ext().execute_with(|| {
            assert_ok!(test_benchmark_create_proposal::<Test>());
            assert_ok!(test_benchmark_vote::<Test>());
            assert_ok!(test_benchmark_on_initialize::<Test>());
        });
    }
}

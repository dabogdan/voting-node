use crate::{mock::*, Error, Event};
use frame_support::{
	assert_ok,
	assert_err
};

#[test]
fn create_proposal_should_work() {
    new_test_ext().execute_with(|| {
        let description = b"Proposal description".to_vec();
        let duration = 10;

        // Create a proposal
        assert_ok!(VotingModule::create_proposal(RuntimeOrigin::signed(1), description.clone(), duration));

        // Verify proposal storage
        let proposal = VotingModule::proposals(1).unwrap();
        assert_eq!(proposal.creator, 1);
        assert_eq!(proposal.description, description);
        assert_eq!(proposal.begin_block, 0);
        assert_eq!(proposal.end_block, 10);
        assert_eq!(proposal.yes_votes, 0);
        assert_eq!(proposal.no_votes, 0);
        assert_eq!(proposal.voting_ended, false);
    });
}

#[test]
fn create_proposal_with_long_description_should_fail() {
    new_test_ext().execute_with(|| {
        let description = vec![0; 300]; // 300 bytes, exceeds MaxDescriptionLength, which is 256
        let duration = 10;

        // Attempt to create a proposal with too long description
        assert_err!(VotingModule::create_proposal(RuntimeOrigin::signed(1), description, duration), Error::<Test>::DescriptionTooLong);
    });
}

#[test]
fn vote_on_proposal_should_work() {
    new_test_ext().execute_with(|| {
        let description = b"Proposal description".to_vec();
        let duration = 10;

        // Create a proposal
        assert_ok!(VotingModule::create_proposal(RuntimeOrigin::signed(1), description.clone(), duration));

        // Vote on the proposal
        assert_ok!(VotingModule::vote(RuntimeOrigin::signed(2), 1, true));

        // Verify votes
        let proposal = VotingModule::proposals(1).unwrap();
        assert_eq!(proposal.yes_votes, 1);
        assert_eq!(proposal.no_votes, 0);
    });
}

#[test]
fn double_voting_should_fail() {
    new_test_ext().execute_with(|| {
        let description = b"Proposal description".to_vec();
        let duration = 10;

        // Create a proposal
        assert_ok!(VotingModule::create_proposal(RuntimeOrigin::signed(1), description.clone(), duration));

        // Vote on the proposal
        assert_ok!(VotingModule::vote(RuntimeOrigin::signed(2), 1, true));

        // Attempt to vote again
        assert_err!(VotingModule::vote(RuntimeOrigin::signed(2), 1, false), Error::<Test>::AlreadyVoted);
    });
}

#[test]
fn voting_after_end_should_fail() {
    new_test_ext().execute_with(|| {
        let description = b"Proposal description".to_vec();
        let duration = 1;

        // Create a proposal
        assert_ok!(VotingModule::create_proposal(RuntimeOrigin::signed(1), description.clone(), duration));

        // Move to block 2
        System::set_block_number(2);

        // Attempt to vote after the voting period has ended
        assert_err!(VotingModule::vote(RuntimeOrigin::signed(2), 1, true), Error::<Test>::VotingPeriodEnded);
    });
}

#[test]
fn finalize_proposal_should_work() {
    new_test_ext().execute_with(|| {
        let description = b"Proposal description".to_vec();
        let duration = 1;

        // Create a proposal
        assert_ok!(VotingModule::create_proposal(RuntimeOrigin::signed(1), description.clone(), duration));

        // Vote on the proposal
        assert_ok!(VotingModule::vote(RuntimeOrigin::signed(2), 1, true));
        assert_ok!(VotingModule::vote(RuntimeOrigin::signed(3), 1, false));
		assert_ok!(VotingModule::vote(RuntimeOrigin::signed(4), 1, true));

        // Move to block 2
        System::set_block_number(1);
		let now = System::block_number();

        // Finalize proposal
        let _ = VotingModule::block_actions(now).is_ok();

        // Verify proposal finalization
        let proposal = VotingModule::proposals(1).unwrap();
        assert_eq!(proposal.voting_ended, true);

        // Verify event
        System::assert_last_event(Event::ProposalFinalized(1, true).into());
    });
}

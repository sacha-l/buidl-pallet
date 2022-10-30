#![cfg_attr(not(feature = "std"), no_std)]

//! # BUIDL Pallet
//!
//! The BUIDL pallet encourages team collaboration and participation at hackathons.
//!
//! User groups include:
//! - Organizers: any organization can post prizes to incentivize buidl teams.
//! - Teams: teams must register their members and can create bounties for other buidlers to contribute to.
//! - Individual buidlers: any hackathon participant hunting bounties.
//! - Judges: elected members to vote on submitted challenges. Note: These have power to execute decisions for the organizers.
//!
//! ## BUIDL pallet
//!
//! The general idea of this pallet is that teams must register in order to be elegible to submit a solution once
//! the buidl period is over. After this period, judges elect which solution win what prize. During the buidl period,
//! team members can issue bounties for other buidlers participating in the event in order to help them achieve their
//! submission idea. If said team wins, the prize is distributed according to the percentages of ownership issued by the
//! team.
//!
//! ### Buidl Bounties
//!
//! Bounties can only be created by a team member.
//! Bounties will expire after some period of time. Once a bounty is claimed and submitted, team members
//! must either approve or reject the bounty. If the bounty is rejected by one member, a majority of members
//! must also reject it, after which the bounty's expiry date is reset.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! Sponsor actions:
//! 
//! - `create_challenge` - Admin just check that the funds are available.
//! - `add_judges` - Admin can add the addresses of initial judges. Note: this should be available but it's the 
//!                  challenge submitter's responsibility to tag judges in the challenge description.
//! - `edit_challenge` - Challenge authors may need to update challenges.
//! 
//! Admin actions:
//! 
//! - `register` - Anyone can register their event by depositing a bond. This should contain admin accounts. 
//! - `update_challenge_list` - Admins can approve new challenges. 
//! - `update_period` - Can update the start and end periods for all event periods (submissions and vote)
//! 
//! Team creation actions:
//!
//! - `create_team` - A participant can create a team of maximum 5 people.
//! - `add_member` - Team members can add up to n members, according to the admin rules.
//!
//! Bounty protocol (for Team):
//!
//! - `post_bounty` - Team members can post a new bounty. Bounty has expiry. Can only be claimed once at time and has a percentage attached.
//! - `extend_bounty_expiry` - Extend the expiry block number of the bounty and stay active.
//! - `approve_bounty` - Close and pay out the specified amount for the completed work.
//!
//! Bounty protocol (for Individual buidl):
//! 
//! - `claim_bounty` - Individual buidlers who worked on a bounty can claim it with their solution.
//! 
//! Judge actions
//! 
//! - `vote` - Judges submit their votes on challenges submission. Once the voting period ends the prizes are
//! 		   automatically distributed according to the challenge description.


// Notes:
// Should be able to handle multiple hackathons at once
// Timing wise (block times for start and end for submissions and judging period)
// Challenges should be able to be added once the hackathon is registered
// Hackathon:

// Organizer
// Challenges
// Judges
// Submission period
// We don't need to be managing funds. We need to find a way to use XCM so that we remove the trust of putting in funds into a pallet. TBD.

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use frame_support::{
		traits::{Currency, LockIdentifier, LockableCurrency, WithdrawReasons},
	};
	use sp_core::H256;

	const DEPOSIT_FOR_CHALLENGE: LockIdentifier = *b" deposit";

	// Handler for balances
	type BalanceOf<T> =
		<<T as Config>::Deposit as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	// Challenge struct
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct Challenge<T: Config> {
		/// Description (ipfs hash)
		pub description: H256,
		/// Reward
		pub reward: BalanceOf<T>,
		/// Eligible judges
		pub judges: Optifon<BoundedVec<T::AccountId, T::MaxMembers>>,
		/// Number of times a challenge has had a solution submitted to it
		pub submissions: u32,
	}
	#[derive(Encode, Decode, TypeInfo, MaxEncodedLen)]
	pub struct ChallengeSolution<T:Config> {
		/// pointer to solution
		pub solution: Vec<u8>,
		/// participants
		pub members: BoundedVec<T::AccountId, T::MaxMembers>,
	}

	/// Struct for holding team information
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Team<T: Config> {
		/// The founding member of this team.
		/// Note: this could be the prime member from membership pallet.
		team_founder: T::AccountId,
		/// The team ID.
		team_id: u32,
		/// The members of this team.
		members: BoundedVec<T::AccountId, T::MaxMembers>,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The abstraction over currency and balances for this pallet.
		type Deposit: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
		/// The maximum amount of people in a team.
		type MaxMembers: Get<u32>;
	}

	/// The next `ChallengeId` to assign.
	#[pallet::storage]
	pub type NextChallengeId<T> = StorageValue<_, u16, ValueQuery>;

	/// ChallengeId -> Challenge
	#[pallet::storage]
	pub type Challenges<T: Config> =
		StorageMap<_, Twox64Concat, u16, Challenge<T>, OptionQuery>;

	/// List of ChallegeIds that are ready to be voted on
	#[pallet::storage]
	pub type ChallengeSolutions<T> = StorageMapStorageMap<_, Twox64Concat, u16, ChallengeSolution<T>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A challenge has been created with [id, creator]
		ChallengeCreated {id: u16, creator: T::AccountId },
		/// Solution has been submitted for a certain challenge [challengeId, sender]
		SolutionSubmitted {id: u16, member: T::AccountId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
	
		// A way for anyone to post their challenge and lock their reward.
		// TODO: This should return with PostInfo 
		#[pallet::weight(0)]
		pub fn create_challenge(
			origin: OriginFor<T>, 
			description: H256,
			id: u32,
			reward: BalanceOf<T>,
			judges: Option<BoundedVec<T::AccountId, T::MaxMembers>>
		) -> DispatchResult
		{
			let who = ensure_signed(origin)?;

			// check has sufficient funds and lock
			// the trait isn't great for multi assets 
			// custom custom impl better over multi-assets
			T::Deposit::set_lock(
				DEPOSIT_FOR_CHALLENGE,
				&who,
				reward,
				WithdrawReasons::all(),
			);

			// create new challenge object
			let new_challenge = Challenge::<T> {
				id,
				description,
				reward,
				judges,
				amount_submitted: 0
			};

			// write to storage 
			Challenges::<T>::insert(who.clone(), new_challenge);

			Self::deposit_event(Event::ChallengeCreated { id, creator: who.clone() });

			Ok(()).into()

		}

		#[pallet::weight(0)]
		pub fn submit_solution(
			origin: OriginFor<T>,
			challengeId: u16,
			solution: Vec<u8>,
			members: Vec<AccountId>
		) -> DispatchResult {
			
			let who = ensure_signed(origin)?;
			Challenges::<T>::contains_key(challengeId)?;
			
			let new_solution: ChallengeSolution = ChallengeSolution::<T> {
				solution = soltion.clone(),
				members = members.clone(),
			};

			ChallengeSolutions::<T>::insert(new_solution);

			Self::deposit_event(Event::SolutionSubmitted{ challengeId, creator: who.clone() });

			Ok(()).into();
		}
	}
}

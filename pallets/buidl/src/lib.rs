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
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;


	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
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
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => return Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}
	}
}

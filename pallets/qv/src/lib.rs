//! # Quadratic Voting Pallet ( pallet-qv )
//!
//! > NOTE: This pallet is tightly coupled with pallet-identity and pallet-referenda.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		traits::{schedule::DispatchTime, Currency, ReservableCurrency},
	};
	use frame_system::{pallet_prelude::*, RawOrigin};

	// From pallet_identity we use functions like has_identity() and set_identity()
	use pallet_identity::IdentityField;
	const IDENTITY_FIELD_DISPLAY: u64 = IdentityField::Display as u64;

	use pallet_referenda::ReferendumIndex;

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Uses tight coupling of pallet_identity and pallet_referenda
	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_identity::Config + pallet_referenda::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// We represent votes by reserving currency
		type Currency: ReservableCurrency<Self::AccountId>;
		type LaunchDeposit: Get<u64>;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn public_props)]
	pub type Depositors<T: Config> =
		StorageMap<_, Blake2_128Concat, u32, Vec<(T::AccountId, BalanceOf<T>)>, OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An amount from the specified accound was reserved
		/// Exposing account id here would go against voting anonymity.
		AmountReserved(BalanceOf<T>),

		/// An amount from the specified accound was unreserved
		AmountUnreserved(BalanceOf<T>),

		/// Somebody voted to launch a referendum
		LaunchVotesCast { number_of_votes: BalanceOf<T>, index: ReferendumIndex },

		/// Referendum launch phase was successfull and will transition to voting phase
		LaunchPhaseSuccess { index: ReferendumIndex },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Can not cast zero votes
		ZeroVote,
		/// User has not set an identity
		NoIdentity,
		/// The proposal already exists
		DuplicateProposal,
		/// The proposal has no valid track
		NoTrack,
		/// The user is considered to already have voted
		AlreadyVoted,
		/// The referendum is still ongoing
		StillOngoing,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Initiate a referendum, which means putting the proposal on-chain and reserving
		/// the initiator's deposit
		///
		/// - `origin`: must be `Signed` and the account must have funds available for the
		///   referendum's track's Decision Deposit.
		/// - `proposal`: A simple hash for now.
		///
		/// This function emits an event Submitted that contains the
		///  1. index,
		///  2. the track,
		///  3. and the hash
		/// of the referendum.
		/// We therefore make the assumption in the rest of the code that a mapping
		///
		/// f(proposal) -> index,
		///
		/// exists off-chain, and all other functions later in the referendum flow
		/// uses the index to refer to the referendum concering the proposal
		///
		/// Reserving the submission deposit is handled entirely inside pallet_referenda.
		/// Refunding it should also be handled inside pallet_referenda.
		///
		/// Emits `pallet_referenda::Event::Submitted`.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn initiate_referendum(origin: OriginFor<T>, proposal: T::Hash) -> DispatchResult {
			const REFERENDUM_BLOCKS_TOTAL: u32 = 892800; // =  2*31*24*60*60/6 = "Two months" / "block time"
			let now = <frame_system::Pallet<T>>::block_number();

			let res = <pallet_referenda::Pallet<T>>::submit(
				origin.clone(),
				Box::new(RawOrigin::Root.into()),
				proposal,
				DispatchTime::At(now + REFERENDUM_BLOCKS_TOTAL.into()),
			);

			if res == Ok(()) {
				// Getting index like this is really fragile in case pallet-referenda changes its
				// indexing scheme The problem is pallet-referenda keeps the members of the
				// ReferendumStatus struct private. Some workaround for that must be found
				let index: ReferendumIndex =
					pallet_referenda::pallet::ReferendumCount::<T>::get() - 1;
				let who = ensure_signed(origin)?;
				let backer_element: (T::AccountId, BalanceOf<T>) = (who, 0u32.into());
				<Depositors<T>>::append(index, backer_element);
			}
			res
		}

		/// Cast launch votes for a referendum that is in the launch phase.
		///
		/// - `origin`: must be `Signed` and the account must have funds equal to or larger than
		///   number_of_votes^2
		/// - `number_of_votes`: The origin wants to cast this number of quadratically priced votes
		/// - `index`: The index of the submitted referendum whose Decision Deposit is yet to be
		///   posted.
		///
		/// This splitting of the deposits across several origins, and the quadratic pricing,
		/// are not implemented inside pallet-referenda.
		/// Therefore we implement this book-keeping ourselves.s
		/// We build on top of
		/// Referenda::place_decision_deposit()
		/// and want to stay as close to its behaviour as we can.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn cast_launch_votes(
			origin: OriginFor<T>,
			number_of_votes: u32,
			index: ReferendumIndex,
		) -> DispatchResult {
			if number_of_votes == 0u32 {
				// This zero-check could probably have been done with a trait
				return Err(Error::<T>::ZeroVote.into())
			}

			let who = ensure_signed(origin.clone())?;
			<pallet_referenda::Pallet<T>>::ensure_ongoing(index)?;

			let depositors_vec = <Depositors<T>>::get(index).unwrap_or_default();
			let disallowed_voters: Vec<T::AccountId> =
				depositors_vec.iter().map(|x| x.0.clone()).collect();

			let disallowed = disallowed_voters.iter().any(|x| *x == who);
			if disallowed {
				return Err(Error::<T>::AlreadyVoted.into())
			}

			let number_of_votes_already =
				depositors_vec.iter().fold(0u32.into(), |acc: BalanceOf<T>, x| acc + x.1);

			// Is the aggregated deposit large enough yet?
			if number_of_votes_already + number_of_votes.into() >=
				(T::LaunchDeposit::get() as u32).into()
			{
				// Last depositor should get refunded through pallet_referenda
				// This must simply be remembered later, when writing
				// the refund_launch_deposit function
				<pallet_referenda::Pallet<T>>::place_triggering_decision_deposit(
					origin.clone(),
					index,
					(number_of_votes * number_of_votes).into(),
				)?;
				let backer_element: (<T as frame_system::Config>::AccountId, BalanceOf<T>) =
					(who.clone(), number_of_votes.into());
				<Depositors<T>>::append(index, backer_element);
				Ok(())
			} else {
				// Register the deposit
				Self::reserve_an_amount_of_token(
					origin.clone(),
					(number_of_votes * number_of_votes).into(),
				)?;
				let backer_element: (<T as frame_system::Config>::AccountId, BalanceOf<T>) =
					(who.clone(), number_of_votes.into());
				<Depositors<T>>::append(index, backer_element);
				Self::deposit_event(Event::LaunchVotesCast {
					number_of_votes: number_of_votes.into(),
					index,
				});
				Ok(())
			}
		}
	}

	/// Helper functions
	impl<T: Config> Pallet<T> {
		/// Reserves an amount of token for a user.
		pub fn reserve_an_amount_of_token(
			origin: OriginFor<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			if pallet_identity::Pallet::<T>::has_identity(&who, IDENTITY_FIELD_DISPLAY) {
				// If funds are too low and Err will be returned
				let res = <T as Config>::Currency::reserve(&who, amount);
				if res == Ok(()) {
					Self::deposit_event(Event::AmountReserved(amount));
				}
				res
			} else {
				Err(Error::<T>::NoIdentity.into())
			}
		}

		/// Unreserves an amount of token for a user.
		pub fn unreserve_an_amount_of_token(
			origin: OriginFor<T>,
			who: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			ensure_root(origin)?;
			if pallet_identity::Pallet::<T>::has_identity(&who, IDENTITY_FIELD_DISPLAY) {
				<T as Config>::Currency::unreserve(&who, amount);
				Self::deposit_event(Event::AmountUnreserved(amount));
				Ok(())
			} else {
				Err(Error::<T>::NoIdentity.into())
			}
		}

		/// Refund all Decision Deposits (launch votes) for a referendum.
		///
		/// - `origin`: must be `Root`
		/// - `index`: The index of the submitted referendum whose Decision Deposit is yet to be
		///   posted.
		///
		/// This function should be triggered by the state machine inside pallet_referendum
		/// (the service_referendum() function).
		///
		/// The splitting of the deposits across several origins, and the quadratic pricing,
		/// are not implemented inside pallet-referenda.
		/// Therefore we implement this book-keeping ourselves inside pallet-qv.
		/// We build on top of
		/// Referenda::refund_decision_deposit()
		/// and want to stay as close to its behaviour as we can.
		pub fn refund_launch_votes(origin: OriginFor<T>, index: ReferendumIndex) -> DispatchResult {
			ensure_root(origin.clone())?;
			if <pallet_referenda::Pallet<T>>::is_ongoing(index) {
				return Err(Error::<T>::StillOngoing.into())
			}

			let mut depositors_vec = <Depositors<T>>::get(index).unwrap_or_default();
			if !depositors_vec.is_empty() {
				depositors_vec.pop(); // Last voter is refunded by pallet_referenda
				depositors_vec
					.iter()
					.next() // first backer is the initiator, which is handled by pallet_referenda
					.map(|(id, amount)| {
						Self::unreserve_an_amount_of_token(origin.clone(), (*id).clone(), *amount)
					});

				<pallet_referenda::Pallet<T>>::refund_decision_deposit(origin.clone(), index)?;
			}

			Ok(())
		}
	}
}

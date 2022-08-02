use crate as pallet_qv;

use crate::mock::system::{EnsureRoot, EnsureSignedBy};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	ord_parameter_types,
	pallet_prelude::TypeInfo,
	parameter_types,
	traits::{
		ConstU16, ConstU32, ConstU64, EitherOfDiverse, EqualPrivilegeOnly, OriginTrait, VoteTally,
	},
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};

use pallet_referenda::{TrackInfo, TracksInfo};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
	pub const MaxLocks: u32 = 10;
}
impl pallet_balances::Config for Test {
	type Balance = u64;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = MaxLocks;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
}

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const BasicDeposit: u64 = 10;
	pub const FieldDeposit: u64 = 10;
	pub const SubAccountDeposit: u64 = 10;
	pub const MaxSubAccounts: u32 = 2;
	pub const MaxAdditionalFields: u32 = 2;
	pub const MaxRegistrars: u32 = 20;
}
ord_parameter_types! {
	pub const One: u64 = 1;
	pub const Two: u64 = 2;
	pub const Three: u64 = 3;
	pub const Four: u64 = 4;
	pub const Five: u64 = 5;
}
type EnsureOneOrRoot = EitherOfDiverse<EnsureRoot<u64>, EnsureSignedBy<One, u64>>;
type EnsureTwoOrRoot = EitherOfDiverse<EnsureRoot<u64>, EnsureSignedBy<Two, u64>>;

impl pallet_identity::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type BasicDeposit = BasicDeposit;
	type FieldDeposit = FieldDeposit;
	type SubAccountDeposit = SubAccountDeposit;
	type MaxSubAccounts = MaxSubAccounts;
	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxRegistrars = MaxRegistrars;
	type Slashed = ();
	type RegistrarOrigin = EnsureOneOrRoot;
	type ForceOrigin = EnsureTwoOrRoot;
	type WeightInfo = ();
}

const ONE_MONTH: u64 = 446400; // 31*24*60*60/6 = "One month" / "block time"

pub struct TestTracksInfo;
impl TracksInfo<u64, u64> for TestTracksInfo {
	type Id = u8;
	type Origin = <Origin as OriginTrait>::PalletsOrigin;
	fn tracks() -> &'static [(Self::Id, TrackInfo<u64, u64>)] {
		static DATA: [(u8, TrackInfo<u64, u64>); 1] = [(
			// IMPORTANT STUFF, this is where we design our referenda.
			// We try to match Votion's visions by configuring this right
			0u8,
			TrackInfo {
				name: "votion",
				max_deciding: 100_000, // This is how many referenda we can have at once
				/// Amount that must be placed on deposit before a decision can be made.
				decision_deposit: 1000, // Need 1000 PWR to go from launch to voting
				/// Amount of time this must be submitted for before a decision can be made.
				prepare_period: ONE_MONTH, // Don't think we will use the prepare period feture
				/// Amount of time that a decision may take to be approved prior to
				/// cancellation.
				decision_period: ONE_MONTH,
				/// Amount of time that the approval criteria must hold before it can be
				/// approved.
				confirm_period: 1,
				/// Minimum amount of time that an approved proposal must be in the dispatch
				/// queue.
				min_enactment_period: 0,
				min_approval: pallet_referenda::Curve::LinearDecreasing {
					length: Perbill::one(), // Go flat at 0% almost from the start
					floor: Perbill::zero(), // We want all referendums to "pass"
					ceil: Perbill::from_percent(100),
				},
				min_support: pallet_referenda::Curve::LinearDecreasing {
					length: Perbill::one(), // Go flat at 0% almost from the start
					floor: Perbill::zero(), // We want all referendums to "pass"
					ceil: Perbill::from_percent(100),
				},
			},
		)];
		&DATA[..]
	}
	fn track_for(id: &Self::Origin) -> Result<Self::Id, ()> {
		if let Ok(system_origin) = frame_system::RawOrigin::try_from(id.clone()) {
			match system_origin {
				frame_system::RawOrigin::Root => Ok(0),
				frame_system::RawOrigin::None => Ok(1),
				_ => Err(()),
			}
		} else {
			Err(())
		}
	}
}

impl pallet_preimage::Config for Test {
	type Event = Event;
	type WeightInfo = ();
	type Currency = Balances;
	type ManagerOrigin = EnsureRoot<u64>;
	type MaxSize = ConstU32<4096>;
	type BaseDeposit = ();
	type ByteDeposit = ();
}

parameter_types! {
	pub static AlarmInterval: u64 = 1;
}

impl pallet_scheduler::Config for Test {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = ConstU64<2_000_000_000_000>;
	type ScheduleOrigin = EnsureRoot<u64>;
	type MaxScheduledPerBlock = ConstU32<100>;
	type WeightInfo = ();
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type PreimageProvider = Preimage;
	type NoPreimagePostponement = ConstU64<10>;
}

#[derive(Encode, Debug, Decode, TypeInfo, Eq, PartialEq, Clone, MaxEncodedLen)]
pub struct Tally {
	pub ayes: u32,
	pub nays: u32,
}

impl<Class> VoteTally<u32, Class> for Tally {
	fn new(_: Class) -> Self {
		Self { ayes: 0, nays: 0 }
	}

	fn ayes(&self, _: Class) -> u32 {
		self.ayes
	}

	fn support(&self, _: Class) -> Perbill {
		Perbill::from_percent(self.ayes)
	}

	fn approval(&self, _: Class) -> Perbill {
		if self.ayes + self.nays > 0 {
			Perbill::from_rational(self.ayes, self.ayes + self.nays)
		} else {
			Perbill::zero()
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn unanimity(_: Class) -> Self {
		Self { ayes: 100, nays: 0 }
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn rejection(_: Class) -> Self {
		Self { ayes: 0, nays: 100 }
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn from_requirements(support: Perbill, approval: Perbill, _: Class) -> Self {
		let ayes = support.mul_ceil(100u32);
		let nays = ((ayes as u64) * 1_000_000_000u64 / approval.deconstruct() as u64) as u32 - ayes;
		Self { ayes, nays }
	}
}

impl pallet_referenda::Config for Test {
	type WeightInfo = ();
	type Call = Call;
	type Event = Event;
	type Scheduler = Scheduler;
	type Currency = pallet_balances::Pallet<Self>;
	type SubmitOrigin = frame_system::EnsureSigned<u64>;
	type CancelOrigin = EnsureSignedBy<Four, u64>;
	type KillOrigin = EnsureRoot<u64>;
	type Slash = ();
	type Votes = u32;
	type Tally = Tally;
	type SubmissionDeposit = ConstU64<1000>;
	type MaxQueued = ConstU32<3>;
	type UndecidingTimeout = ConstU64<ONE_MONTH>; // "one month in sec" / "6s block time"
	type AlarmInterval = AlarmInterval;
	type Tracks = TestTracksInfo;
}

impl pallet_qv::Config for Test {
	type Event = Event;
	type Currency = Balances;
}

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances,
		Qv: pallet_qv::{Pallet, Call, Storage, Event<T>},
		Identity: pallet_identity,
		Preimage: pallet_preimage,
		Referenda: pallet_referenda,
		Scheduler: pallet_scheduler,
	}
);

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
	// Give different origins some start-mints
	// the set_identity function costs 10 tokens...
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(1, 10),
			(2, 10),
			(3, 10),
			(10, 100),
			(20, 110),
			(30, 1010),
			(31, 250_010), // 31 and 32 have many tokens just to allow shorter tests
			(32, 250_010),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();
	t.into()
}

#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;



#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::{DispatchResult, DispatchResultWithPostInfo},
		pallet_prelude::{*, StorageValue},
		sp_runtime::{traits::{Hash, Zero}},
		traits::{Currency, ExistenceRequirement, Randomness},
		transactional, Twox64Concat,
	};
	use frame_system::pallet_prelude::{*, OriginFor};

	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> = 
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct File<T: Config> {
		//pub file_cid: [u8; 16],
		pub file_link: String,
		pub allow_download: bool,
		pub file_type: FileType,
		pub cost: Option<BalanceOf<T>>,
		pub file_size: u64,
		pub owner: AccountOf<T> 
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum FileType {
		Normal, 
		Priviledged
	}

	//impl Default for FileType {
	//	fn default() -> Self {
	//		FileType::Normal
	//	}
	//}

	#[pallet::pallet]
	#[pallet::generate_store(trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: pallet_balances::Config + frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: Currency<Self::AccountId>;

		#[pallet::constant]
		type DefaultFreeFileSize: Get<u32>;

		#[pallet::constant]
		type CostPerByte: Get<u32>;

		#[pallet::constant]
		type MaxFilesUploaded: Get<u32>;

		//#[pallet::constant]
		//type
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	
	#[pallet::storage]
	#[pallet::getter(fn all_files_count)]
	/// Keeps track of the number of files available.
	pub(super) type AllFilesCount<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn file_details)]
	/// File details
	pub(super) type Files<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::Hash,
		File<T>
		>;  
	
	#[pallet::storage]
	#[pallet::getter(fn get_user_file_details)]
	pub(super) type FilesPerUser<T: Config> = StorageMap<
		_, 
		Twox64Concat, 
		T::AccountId, 
		BoundedVec<T::Hash, 
		T::MaxFilesUploaded>, 
		ValueQuery
		>;
	
	
	#[pallet::storage]
	#[pallet::getter(fn downloaded_files)]
	pub(super) type DownloadedFiles<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		BoundedVec<T::Hash, T::MaxFilesUploaded>,
		ValueQuery,
	    >;

	//#[pallet::storage]
	//#[pallet::getter(fn users_who_download)]
	//pub (super) type UsersWhoDownload<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, BoundedVec<T::Hash, T::MaxFilesUploaded>, ValueQuery>;



	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// User is not allowed to download file.
		FileNotAllowedToDownload,
		/// The file does not exist
		FileNotExist,
		/// Ensures that the user who wants to dowload privileged files has enought funds to do so.
		FilePricetooLow,
		FileCountOverflow,
		ExceedMaxFileUploaded,
		/// Ensures that an account has enough funds to download file
		NotEnoughBalance 
	}

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A file was successfully uploaded.
		Uploaded(T::AccountId, T::Hash),
		/// A file was successfully downloaded.
		Downloaded(T::AccountId, T::Hash,  BalanceOf<T>),
		/// A file was successfully transfered.
		Transfered(T::AccountId, T::AccountId, T::Hash)
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(100)]
		pub fn upload_file(
			origin: OriginFor<T>, 
			file_cid: T::Hash, 
			file_link: String,
		  	allow_download: bool,
			file_type: FileType,
			cost: BalanceOf<T>,
			file_size: u64
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let file = File::<T> {
				file_link,
				allow_download,
				file_type,
				cost,
				file_size,
				owner: sender.clone()
			};

			let file_cid = T::Hashing::hash_of(&file);
			let new_count = Self::all_files_count().checked_add(1).ok_or(<Error<T>>::FileCountOverflow)?;

			<FilesPerUser<T>>::try_mutate(&sender, |file_vec| file_vec.try_push(file_cid))
			.map_err(|_| <Error<T>>::ExceedMaxFileUploaded)?;

			<Files<T>>::insert(file_cid, file);
			<AllFilesCount<T>>::put(new_count);

			Self::deposit_event(Event::Uploaded(sender, file_cid));

			Ok(())

		}

		#[pallet::weight(100)]
		pub fn download_file(origin: OriginFor<T>, file_cid: T::Hash) -> DispatchResult {

		}

		#[pallet::weight(100)]
		pub fn transfer_file(origin: OriginFor<T>, file_cid: T::Hash, new_owner: T::AccountId) -> DispatchResult {

		}
	}
}

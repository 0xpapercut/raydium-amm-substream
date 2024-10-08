pub(crate) mod asset_data;
pub(crate) mod collection;
pub(crate) mod creator;
pub(crate) mod data;
pub(crate) mod delegate;
pub(crate) mod edition;
pub(crate) mod edition_marker;
pub(crate) mod edition_marker_v2;
pub(crate) mod escrow;
pub mod fee;
pub(crate) mod master_edition;
pub(crate) mod metadata;
pub(crate) mod migrate;
// pub(crate) mod programmable;
// pub(crate) mod reservation;
pub(crate) mod token_auth_payload;
pub(crate) mod uses;

pub use asset_data::*;
use borsh::{BorshDeserialize, BorshSerialize};
pub use collection::*;
pub use creator::*;
pub use data::*;
pub use delegate::*;
pub use edition::*;
pub use edition_marker::*;
pub use edition_marker_v2::*;
pub use escrow::*;
pub use fee::*;
pub use master_edition::*;
pub use metadata::*;
pub use migrate::*;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
// pub use programmable::*;
// pub use reservation::*;
pub use uses::*;
use super::error::{ProgramError, MetadataError};

pub(crate) use substreams_solana_utils::pubkey::Pubkey;

// Re-export constants to maintain compatibility.
// pub use crate::pda::{BURN, COLLECTION_AUTHORITY, EDITION, PREFIX, USER};
// use crate::{
//     assertions::assert_owned_by,
//     error::MetadataError,
//     utils::{meta_deser_unchecked, try_from_slice_checked},
//     ID,
// };

/// Index of the discriminator on the account data.
pub const DISCRIMINATOR_INDEX: usize = 0;

#[repr(C)]
#[cfg_attr(feature = "serde-feature", derive(Serialize, Deserialize))]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum TokenStandard {
    NonFungible,                    // This is a master edition
    FungibleAsset,                  // A token with metadata that can also have attributes
    Fungible,                       // A token with simple metadata
    NonFungibleEdition,             // This is a limited edition
    ProgrammableNonFungible,        // NonFungible with programmable configuration
    ProgrammableNonFungibleEdition, // NonFungible with programmable configuration
}

pub trait TokenMetadataAccount: BorshDeserialize {
    fn key() -> Key;

    fn size() -> usize;

    fn is_correct_account_type(data: &[u8], data_type: Key, data_size: usize) -> bool {
        if data.is_empty() {
            return false;
        }

        let key: Option<Key> = Key::from_u8(data[0]);
        match key {
            Some(key) => {
                (key == data_type || key == Key::Uninitialized)
                    && (data.len() == data_size || data_size == 0)
            }
            None => false,
        }
    }

    fn pad_length(buf: &mut Vec<u8>) -> Result<(), MetadataError> {
        let padding_length = Self::size()
            .checked_sub(buf.len())
            .ok_or(MetadataError::NumericalOverflowError)?;
        buf.extend(vec![0; padding_length]);
        Ok(())
    }

    // fn safe_deserialize(mut data: &[u8]) -> Result<Self, BorshError> {
    //     if !Self::is_correct_account_type(data, Self::key(), Self::size()) {
    //         return Err(BorshError::new(ErrorKind::Other, "DataTypeMismatch"));
    //     }

    //     let result = Self::deserialize(&mut data)?;

    //     Ok(result)
    // }

//     fn from_account_info(a: &AccountInfo) -> Result<Self, ProgramError>
// where {
//         let data = &a.data.borrow_mut();

//         let ua = Self::safe_deserialize(data).map_err(|_| MetadataError::DataTypeMismatch)?;

//         // Check that this is a `token-metadata` owned account.
//         assert_owned_by(a, &ID)?;

//         Ok(ua)
//     }
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Copy, FromPrimitive)]
pub enum Key {
    Uninitialized,
    EditionV1,
    MasterEditionV1,
    ReservationListV1,
    MetadataV1,
    ReservationListV2,
    MasterEditionV2,
    EditionMarker,
    UseAuthorityRecord,
    CollectionAuthorityRecord,
    TokenOwnedEscrow,
    TokenRecord,
    MetadataDelegate,
    EditionMarkerV2,
    HolderDelegate,
}

#[cfg(feature = "serde-feature")]
fn deser_option_pubkey<'de, D>(deserializer: D) -> Result<Option<Pubkey>, D::Error>
where
    D: Deserializer<'de>,
{
    <Option<String> as serde::de::Deserialize>::deserialize(deserializer)?
        .map(|s| Pubkey::from_str(&s))
        .transpose()
        .map_err(serde::de::Error::custom)
}

#[cfg(feature = "serde-feature")]
fn ser_option_pubkey<S>(pubkey: &Option<Pubkey>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let pubkey_string = pubkey.as_ref().map(|p| p.to_string());
    serde::ser::Serialize::serialize(&pubkey_string, serializer)
}

/// Trait for resizable accounts.
///
/// Implementing this trait for a type will automatically allow the use of the `save` method,
/// which can modify the size of an account.
///
/// A type implementing this trait must specify the `from_bytes` method, since an account can
/// have variable size.
pub trait Resizable: TokenMetadataAccount + BorshSerialize {
    /// Deserializes the struct data from the specified byte array.
    ///
    /// In most cases this will perform a custom deserialization since the size of the
    /// stored byte array (account) can change.
    fn from_bytes(data: &[u8]) -> Result<Self, ProgramError>;
}

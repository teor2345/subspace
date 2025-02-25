//! SCALE codec encoding and decoding primitives.

use codec::{Decode, Encode, EncodeLike, Error, Input, Output};
use scale_info::prelude::marker::PhantomData;
use scale_info::prelude::ops::{Deref, DerefMut};
use scale_info::prelude::vec::Vec;
use scale_info::TypeInfo;

/// A type that decodes from either `Current` or `Fallback`, but always encodes to `Current`.
/// Useful for types that are decoded before or during storage migrations.
///
/// If decoding both types fails, returns the error from decoding `Current`.
#[derive(Clone, Debug, PartialEq, Eq, TypeInfo)]
pub struct DecodeFallback<Current, Fallback> {
    /// The decoded value, possibly converted from `Fallback`.
    pub current: Current,
    /// A marker to bind the `Fallback` generic to this struct.
    _phantom: PhantomData<Fallback>,
}

impl<Current, Fallback> EncodeLike<DecodeFallback<Current, Fallback>>
    for DecodeFallback<Current, Fallback>
where
    Self: Encode,
{
}

impl<Current, Fallback> Deref for DecodeFallback<Current, Fallback> {
    type Target = Current;

    fn deref(&self) -> &Self::Target {
        &self.current
    }
}

impl<Current, Fallback> DerefMut for DecodeFallback<Current, Fallback> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.current
    }
}

// Unfortunately we can't implement the comparison the other way generically due to orphan rules.
impl<Current, Fallback> PartialEq<Current> for DecodeFallback<Current, Fallback>
where
    Current: PartialEq,
{
    fn eq(&self, other: &Current) -> bool {
        &self.current == other
    }
}

impl<Current, Fallback> From<Current> for DecodeFallback<Current, Fallback> {
    fn from(current: Current) -> Self {
        Self {
            current,
            _phantom: PhantomData,
        }
    }
}

// Always encodes to Current
impl<Current, Fallback> Encode for DecodeFallback<Current, Fallback>
where
    Current: Encode,
{
    fn size_hint(&self) -> usize {
        self.current().size_hint()
    }

    fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
        self.current().encode_to(dest)
    }

    fn encode(&self) -> Vec<u8> {
        self.current().encode()
    }

    fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
        self.current().using_encoded(f)
    }

    fn encoded_size(&self) -> usize {
        self.current().encoded_size()
    }
}

// Try decoding as Current, then if that fails, decode as Fallback
impl<Current, Fallback> Decode for DecodeFallback<Current, Fallback>
where
    Current: Decode,
    Fallback: Decode + Into<Current>,
{
    fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
        match Current::decode(input) {
            Ok(current) => Ok(Self {
                current,
                _phantom: PhantomData,
            }),
            Err(current_error) => {
                if let Ok(fallback) = Fallback::decode(input) {
                    Ok(Self {
                        current: fallback.into(),
                        _phantom: PhantomData,
                    })
                } else {
                    Err(current_error)
                }
            }
        }
    }

    fn encoded_fixed_size() -> Option<usize> {
        // All values only have a fixed size if the fixed sizes of the current and fallback types
        // are the same.
        if Current::encoded_fixed_size() == Fallback::encoded_fixed_size() {
            Current::encoded_fixed_size()
        } else {
            None
        }
    }
}

impl<Current, Fallback> DecodeFallback<Current, Fallback> {
    /// Returns the `Current` value.
    #[inline]
    pub fn current(&self) -> &Current {
        &self.current
    }

    /// Returns a mutable reference to the `Current` value.
    #[inline]
    pub fn current_mut(&mut self) -> &mut Current {
        &mut self.current
    }

    /// Extracts the `Current` value.
    #[inline]
    pub fn into_current(self) -> Current {
        self.current
    }
}

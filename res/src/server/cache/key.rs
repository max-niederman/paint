use canvas::Id;
use miette::{IntoDiagnostic, Result, WrapErr, miette};
use std::ffi::CString;

/// A structured key in the cache [`Tree`].
pub trait Key: Sized {
    type Bytes: AsRef<[u8]>;
    fn as_bytes(&self) -> Self::Bytes;
    fn parse_bytes<B: Iterator<Item = u8>>(bytes: &mut B) -> Result<Self>;
}

pub trait KeyPrefix<K: Key>: Key {}
impl<K: Key> KeyPrefix<K> for K {}

impl Key for () {
    type Bytes = [u8; 0];
    fn as_bytes(&self) -> Self::Bytes {
        []
    }
    fn parse_bytes<B: Iterator<Item = u8>>(bytes: &mut B) -> Result<Self> {
        Ok(())
    }
}

pub struct CanvasKey<P> {
    pub prefix: P,
    // the instance must not contain a null byte, so we use [`CString`]
    pub instance: CString,
}
impl<P: Key> Key for CanvasKey<P> {
    type Bytes = Vec<u8>;
    fn as_bytes(&self) -> Self::Bytes {
        [
            self.prefix.as_bytes().as_ref(),
            &self.instance.as_bytes_with_nul(),
        ]
        .concat()
    }
    fn parse_bytes<B: Iterator<Item = u8>>(bytes: &mut B) -> Result<Self> {
        Ok(Self {
            prefix: P::parse_bytes(bytes)?,
            instance: CString::new(
                bytes
                    .take_while(|&b| b != b'\0')
                    .chain(std::iter::once(b'\0'))
                    .collect::<Vec<u8>>(),
            )
            .into_diagnostic()
            .wrap_err("while parsing Canvas instance key segment")?,
        })
    }
}
impl<P: Key> KeyPrefix<CanvasKey<P>> for P {}

pub struct IdKey<P> {
    pub prefix: P,
    pub id: Id,
}
impl<'p, P: Key> Key for IdKey<P> {
    type Bytes = Vec<u8>;
    fn as_bytes(&self) -> Self::Bytes {
        [self.prefix.as_bytes().as_ref(), &self.id.to_be_bytes()].concat()
    }
    fn parse_bytes<B: Iterator<Item = u8>>(bytes: &mut B) -> Result<Self> {
        Ok(Self {
            prefix: P::parse_bytes(bytes)?,
            id: Id::from_be_bytes(
                bytes
                    .take(std::mem::size_of::<Id>())
                    .collect::<Vec<u8>>()
                    .try_into()
                    .map_err(|_| miette!("unexpected end of input"))
                    .wrap_err("while parsing Canvas ID key segment")?
            ),
        })
    }
}
impl<P: Key> KeyPrefix<IdKey<P>> for P {}

// we alias these to make the semantics clearer
pub type CourseKey<P> = IdKey<P>;
pub type UserKey<P> = IdKey<P>;

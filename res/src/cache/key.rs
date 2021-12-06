use super::Error;
use crate::{view, Result};

pub trait Key: Sized {
    /// The length in bytes of the serialized key.
    const LEN: usize;

    /// Serialize the key into a byte vector.
    ///
    /// Unfortunately, because Rust does not have full-blown dependent typing (yet),
    /// we cannot just return a [`[u8; Self::LEN]`] here because we would then not be
    /// able to implement [`Key`] for some composite keys. Nevertheless, the returned
    /// [`heapless::Vec`] must always be at capacity.
    fn serialize(&self) -> Result<Vec<u8>>;
    /// Deserialize the key from a byte iterator.
    fn deserialize<I: Iterator<Item = u8>>(bytes: &mut I) -> Result<Self>;
}

/// The maximum length of a Canvas instance.
/// Instances shorter than this will be padded and instances longer will be fail.
/// This is necessary to prevent accidental prefix overlaps.
pub const MAX_CANVAS_LENGTH: usize = 64;

impl Key for view::Canvas {
    const LEN: usize = MAX_CANVAS_LENGTH;

    fn serialize(&self) -> Result<Vec<u8>> {
        if !self.base_url.as_bytes().iter().all(|&b| b != b'\0') {
            return Err(Error::UnexpectedStreamYield {
                expected: "non-null byte",
                actual: "null byte",
            }
            .into());
        }

        let mut bytes = heapless::Vec::<u8, { Self::LEN }>::from_slice(self.base_url.as_bytes())
            .map_err(|_| Error::IllegalCanvasBaseUrl {
                base_url: self.base_url.clone(),
                location: Some((MAX_CANVAS_LENGTH, self.base_url.len() - 1)),
                problem: "exceeds maximum length",
            })?;
        bytes.resize(bytes.capacity(), b'\0').unwrap();
        Ok(bytes.to_vec())
    }

    fn deserialize<I: Iterator<Item = u8>>(bytes: &mut I) -> Result<Self> {
        let mut base_url_bytes = Vec::with_capacity(Self::LEN);
        base_url_bytes.extend(bytes.take(Self::LEN).take_while(|&b| b != b'\0'));

        Ok(Self {
            base_url: String::from_utf8(base_url_bytes).map_err(|e| {
                let valid_up_to = e.utf8_error().valid_up_to();
                Error::IllegalCanvasBaseUrl {
                    base_url: format!(
                        "{:#?} followed by {} invalid bytes",
                        std::str::from_utf8(&base_url_bytes[..valid_up_to]),
                        valid_up_to
                    ),
                    location: None,
                    problem: "invalid UTF-8",
                }
            })?,
        })
    }
}

impl Key for view::Viewer {
    // one byte for the discriminant and eight for the union
    const LEN: usize = 1 + 8;

    fn serialize(&self) -> Result<Vec<u8>> {
        let mut bytes = heapless::Vec::<u8, { Self::LEN }>::new();

        match self {
            Self::User(id) => {
                bytes.push(0).unwrap();
                bytes.extend(id.to_be_bytes());
            }
        }

        debug_assert!(bytes.is_full());
        Ok(bytes.to_vec())
    }

    fn deserialize<I: Iterator<Item = u8>>(bytes: &mut I) -> Result<Self> {
        let discriminant = bytes.next().ok_or(Error::UnexpectedStreamYield {
            expected: "discriminant",
            actual: "end of stream",
        })?;
        match discriminant {
            0 => Ok(Self::User(canvas::Id::from_be_bytes(
                bytes
                    .take(8)
                    .collect::<heapless::Vec<u8, 8>>()
                    .into_array()
                    .map_err(|_| Error::UnexpectedStreamYield {
                        expected: "eight-byte user id",
                        actual: "end of stream",
                    })?,
            ))),
            _ => Err(Error::IllegalViewerDiscriminant { discriminant }.into()),
        }
    }
}

impl Key for view::View {
    const LEN: usize = view::Canvas::LEN + view::Viewer::LEN;

    fn serialize(&self) -> Result<Vec<u8>> {
        let mut bytes = heapless::Vec::<u8, { Self::LEN }>::new();

        bytes.extend(self.truth.serialize()?);
        bytes.extend(self.viewer.serialize()?);

        debug_assert!(bytes.is_full());
        Ok(bytes.to_vec())
    }

    fn deserialize<I: Iterator<Item = u8>>(bytes: &mut I) -> Result<Self> {
        Ok(Self {
            truth: view::Canvas::deserialize(bytes)?,
            viewer: view::Viewer::deserialize(bytes)?,
        })
    }
}

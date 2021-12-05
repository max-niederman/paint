use canvas::Id;
use std::mem::{self, MaybeUninit};

/// A structured key in the cache [`Tree`].
pub trait Key: Sized + Eq {
    const LEN: usize;
    fn into_bytes(&self) -> [u8; Self::LEN];
    fn from_bytes(bytes: &[u8; Self::LEN]) -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitKey;
impl Key for UnitKey {
    const LEN: usize = 0;
    fn into_bytes(&self) -> [u8; Self::LEN] {
        []
    }
    fn from_bytes(bytes: &[u8; Self::LEN]) -> Self {
        UnitKey
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Join<L: Key, R: Key>(pub L, pub R);
impl<L: Key, R: Key> Key for Join<L, R>
where
    [(); L::LEN + R::LEN]: Sized,
{
    const LEN: usize = L::LEN + R::LEN;
    fn into_bytes(&self) -> [u8; Self::LEN] {
        let joined = [0; L::LEN + R::LEN];

        let (lb, rb) = joined.split_at_mut(L::LEN);
        lb.copy_from_slice(self.0.into_bytes().as_ref());
        rb.copy_from_slice(self.1.into_bytes().as_ref());

        joined
    }
    fn from_bytes(bytes: &[u8; Self::LEN]) -> Self {
        let (lb, rb) = bytes.split_array_mut();
        Self(L::from_bytes(lb), R::from_bytes(rb))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdKey(pub Id);
impl Key for IdKey {
    const LEN: usize = mem::size_of::<Id>();
    fn into_bytes(&self) -> [u8; Self::LEN] {
        self.0.to_be_bytes()
    }
    fn from_bytes(bytes: &[u8; Self::LEN]) -> Self {
        Self(Id::from_be_bytes(bytes))
    }
}

// we store the canvas key as a sequence of non null bytes followed by a variable number of null padding bytes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanvasKey<const LEN: usize>([u8; LEN]);
impl<const LEN: usize> Key for CanvasKey<LEN> {
    const LEN: usize = LEN;
    fn into_bytes(&self) -> [u8; Self::LEN] {
        self.0
    }
    fn from_bytes(bytes: &[u8; Self::LEN]) -> Self {
        Self(*bytes)
    }
}
impl<const LEN: usize> CanvasKey<LEN> {
    /// Create a [`CanvasKey`] from a slice of bytes.
    /// This checks to make sure there are no null bytes.
    ///
    /// # Panics
    /// Panics if `cnv`'s length exceeds the maximum.
    pub fn new<S: AsRef<[u8]>>(cnv: &S) -> Option<Self> {
        assert!(cnv.as_ref().len() > LEN);
        cnv.as_ref()
            .iter()
            .all(|&b| b != b'\0')
            .then(|| unsafe { Self::new_unchecked(cnv) })
    }

    /// Create a [`CanvasKey`] without checking to make sure it is valid.
    pub const unsafe fn new_unchecked<S: AsRef<[u8]>>(cnv: &S) -> Self {
        let arr = [b'\0'; LEN];
        arr[..cnv.as_ref().len()].copy_from_slice(cnv.as_ref());
        Self(arr)
    }
}

#[marker]
pub trait KeyPrefix<K: Key> {}

// keys are prefixes of themselves
impl<K: Key> KeyPrefix<K> for K {}
// the unit key is a prefix of every key
impl<K: Key> KeyPrefix<K> for UnitKey {}
// [`Join`] composes prefixes with suffixes
impl<L: Key, R: Key> KeyPrefix<Join<L, R>> for L {}
// the prefix property is transitive
// impl<K: Key + KeyPrefix<A>, A: Key + KeyPrefix<B>, B: Key> KeyPrefix<B> for K {}

#[test]
fn parse_bytes_inverts_as_bytes() {
    use std::fmt::Debug;

    fn test<K: Key + Debug>(key: K) {
        let bytes = key.as_bytes();
        let mut bytes = bytes.as_ref().iter().copied();
        let parsed = K::parse_bytes(&mut bytes).unwrap();
        assert_eq!(parsed, key);
    }

    test(IdKey(0));
    test(IdKey(1));

    test(CanvasKey::new("foo.bar"));
}

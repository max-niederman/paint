use super::prelude::*;
use resource::submission::*;

impl Cache for Submission {
    type Key = SubmissionKey;

    #[inline]
    fn key(&self) -> Self::Key {
        self.into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SubmissionKey {
    assignment: canvas::Id,
    user: canvas::Id,
    attempt: u32,
}

impl Key for SubmissionKey {
    const SER_LEN: usize = canvas::Id::SER_LEN + canvas::Id::SER_LEN + 4;

    fn serialize(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(Self::SER_LEN);
        buf.extend_from_slice(&self.assignment.serialize()?);
        buf.extend_from_slice(&self.user.serialize()?);
        buf.extend_from_slice(&self.attempt.to_be_bytes());
        Ok(buf)
    }

    fn deserialize<I: Iterator<Item = u8>>(bytes: &mut I) -> Result<Self> {
        Ok(Self {
            assignment: canvas::Id::deserialize(bytes)?,
            user: canvas::Id::deserialize(bytes)?,
            attempt: u32::from_be_bytes(
                bytes
                    .by_ref()
                    .take(4)
                    .collect::<heapless::Vec<u8, 4>>()
                    .into_array()
                    .map_err(|_| Error::UnexpectedStreamYield {
                        expected: "four-byte attempt id",
                        actual: "end of stream",
                    })?,
            ),
        })
    }
}

impl From<&Submission> for SubmissionKey {
    fn from(submission: &Submission) -> Self {
        Self {
            assignment: submission.assignment_id,
            user: submission.user_id,
            attempt: submission.attempt.unwrap_or(0),
        }
    }
}

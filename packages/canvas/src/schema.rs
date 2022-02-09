use canvas_lms::resource::*;
use gen_schema::collection_of;
use std::{env, io, path::PathBuf};

fn main() -> io::Result<()> {
    let schemas = collection_of![
        Assignment,
        Course,
        Enrollment,
        Grade,
        GradingPeriod,
        Submission,
        User,
    ];

    let out = PathBuf::from(
        env::args()
            .skip(1)
            .next()
            .expect("must specify output directory"),
    );

    schemas.write_to(&out)
}

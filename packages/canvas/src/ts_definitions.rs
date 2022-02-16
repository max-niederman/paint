use canvas_lms::resource::*;
use std::{env, fs, io, path::PathBuf};

fn main() -> io::Result<()> {
    let definitions = all_types::definitions();
    let out_path = PathBuf::from(
        env::args()
            .skip(1)
            .next()
            .as_ref()
            .map(String::as_str)
            .unwrap_or("/dev/stdout"),
    );

    fs::write(&out_path, definitions.as_bytes())
}

mod all_types {
    use canvas_lms::{id, resource::*};

    macro_rules! type_definitions_for {
        ($( $type:ty ),* $(,)?) => {
            {
                let mut definitions = String::new();

                definitions.push_str("declare namespace Canvas {");
                $(
                    definitions.push_str(
                        &<$type as typescript_definitions::TypeScriptifyTrait>::type_script_ify()
                    );
                    definitions.push_str("\n\n");
                )*
                definitions.push_str("}\n");

                definitions
            }
        };
    }

    pub fn definitions() -> String {
        type_definitions_for!(
            id::Id,
            assignment::Assignment,
            assignment::AssignmentOverride,
            assignment::GradingType,
            assignment::ScoreStatistics,
            assignment::LockInfo,
            course::Course,
            course::CourseWorkflowState,
            course::CourseView,
            course::Term,
            course::CourseProgress,
            course::CourseFormat,
            course::Permissions,
            enrollment::Enrollment,
            enrollment::InlineEnrollment,
            enrollment::EnrollmentState,
            enrollment::EnrollmentType,
            enrollment::EnrollmentRole,
            enrollment::Grade,
            grading_period::GradingPeriod,
            submission::Submission,
            submission::SubmissionType,
            submission::SubmissionWorkflowState,
            submission::LatePolicyStatus,
            user::User,
        )
    }
}

use std::{env, fs, io, path::PathBuf};

fn main() -> io::Result<()> {
    let definitions = definitions();
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

macro_rules! type_definitions_for {
    ($( $type:ty ),* $(,)?) => {
        {
            let mut definitions = String::new();

            definitions.push_str("declare namespace Oil {");
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
    use oil::{routes, view};

    type_definitions_for!(view::View, routes::view::NewView)
}

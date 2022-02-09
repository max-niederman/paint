use schemars::{schema_for, JsonSchema};
use std::{fs, io, path::Path};

#[derive(Debug, Clone, Default)]
pub struct Collection {
    schemas: Vec<(String, schemars::schema::RootSchema)>,
}

impl Collection {
    pub const fn new() -> Self {
        Self {
            schemas: Vec::new(),
        }
    }

    pub fn insert<T: JsonSchema>(&mut self) -> &mut Self {
        self.schemas.push((T::schema_name(), schema_for!(T)));
        self
    }

    pub fn write_to(&self, path: &impl AsRef<Path>) -> io::Result<()> {
        for (name, schema) in &self.schemas {
            let mut path = path.as_ref().join(name);
            path.set_extension("json");

            let file = fs::File::create(path)?;

            serde_json::to_writer_pretty(file, schema)?;
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! collection_of {
    ($( $type:ty ),* $(,)?) => {{
        let mut collection = ::gen_schema::Collection::new();
        $(
            collection.insert::<$type>();
        )*
        collection
    }};
}

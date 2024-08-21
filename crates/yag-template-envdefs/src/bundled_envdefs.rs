use super::{EnvDefSource, EnvDefs, ParseError};

macro_rules! sources {
    ($($filename:literal),*) => {
        &[$(
            EnvDefSource::new_static($filename, include_str!(concat!("../../../bundled-defs/", $filename))),
        )*]
    }
}

pub fn load() -> Result<EnvDefs, ParseError> {
    static BUNDLED_SOURCES: &[EnvDefSource] = sources![
        "builtin_funcs.ydef",
        "context_funcs.ydef",
        "ext_plugin_funcs.ydef",
        "general_funcs.ydef",
        "interaction_funcs.ydef"
    ];

    super::parse(BUNDLED_SOURCES)
}

#[test]
fn bundled_sources_are_valid() {
    load().expect("bundled sources should be valid");
}

use super::{EnvDefSource, EnvDefs, ParseError};

pub fn load() -> Result<EnvDefs, ParseError> {
    static BUNDLED_SOURCES: &[EnvDefSource<'static>] = &[
        EnvDefSource {
            name: "builtin_funcs.ydef",
            data: include_str!("../../../bundled-defs/builtin_funcs.ydef"),
        },
        EnvDefSource {
            name: "context_funcs.ydef",
            data: include_str!("../../../bundled-defs/builtin_funcs.ydef"),
        },
        EnvDefSource {
            name: "ext_plugin_funcs.ydef",
            data: include_str!("../../../bundled-defs/ext_plugin_funcs.ydef"),
        },
        EnvDefSource {
            name: "general_funcs.ydef",
            data: include_str!("../../../bundled-defs/general_funcs.ydef"),
        },
    ];

    super::parse(BUNDLED_SOURCES)
}

#[test]
fn bundled_sources_are_valid() {
    load().expect("bundled sources should be valid");
}

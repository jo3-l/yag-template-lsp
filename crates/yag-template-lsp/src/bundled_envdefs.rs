use yag_template_envdefs::{EnvDefSource, EnvDefs, ParseError};

pub(crate) fn load() -> Result<EnvDefs, ParseError> {
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

    yag_template_envdefs::parse(BUNDLED_SOURCES)
}

#[test]
fn bundled_sources_are_valid() {
    load().expect("bundled sources should be valid");
}

use crate::util::toml_ext::{TomlBuilder, TomlExt};

#[test]
fn test_override() {
    let val1 = toml::toml! {
        debug = true
        log = false

        [map1]
        val11 = 42
        val12 = false
    };

    let val2 = toml::toml! {
        [map1.map2]
        val22 = 5.2
    };

    let val3 = toml::toml! {
        log = true

        [map1]
        val11 = 40
    };

    let result = toml::toml! {
        debug = true
        log = true

        [map1]
        val11 = 40
        val12 = false

        [map1.map2]
        val22 = 5.2
    };

    assert_eq!(
        val1.with_overrides(val2).with_overrides(val3),
        result.into()
    );
}

#[test]
fn test_builder() {
    assert_eq!(
        TomlBuilder::default()
            .with_opt(["debug"], Some(true))
            .with_opt(["log"], None::<bool>)
            .with_opt(["map1", "map2", "val"], Some(5.2))
            .build(),
        toml::toml! {
            debug = true

            [map1.map2]
            val = 5.2
        }
        .into()
    );
}

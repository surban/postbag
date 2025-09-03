#[cfg(test)]
mod tests {
    use postcard::{Config, DefaultCfg, to_io, to_io_with_cfg, to_vec, to_vec_with_cfg};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    enum TestEnum {
        Variant1,
        Variant2(u32),
    }

    // Type aliases for common configurations
    type WithIdentifiers = Config<true>;
    type WithoutIdentifiers = Config<false>;

    #[test]
    fn test_configurable_serialization() {
        let data = TestEnum::Variant2(42);

        // Test with DefaultCfg (includes identifiers)
        let with_identifiers = to_vec_with_cfg::<_, DefaultCfg>(&data).unwrap();

        // Test with Config<false> (no identifiers)
        let without_identifiers = to_vec_with_cfg::<_, Config<false>>(&data).unwrap();

        // Verify that serialization produces different output when configuration changes
        assert_ne!(with_identifiers, without_identifiers);
    }

    #[test]
    fn test_configurable_io_serialization() {
        let data = TestEnum::Variant1;

        // Test to_io_with_cfg with different configurations
        let mut buffer1 = Vec::new();
        let mut buffer2 = Vec::new();

        to_io_with_cfg::<_, _, DefaultCfg>(&data, &mut buffer1).unwrap();
        to_io_with_cfg::<_, _, Config<false>>(&data, &mut buffer2).unwrap();

        // Different configurations should produce different output
        assert_ne!(buffer1, buffer2);
    }

    #[test]
    fn test_backward_compatibility() {
        let data = 42u32;

        // The original functions should work the same as the configurable ones with DefaultCfg
        let original_vec = to_vec(&data).unwrap();
        let configurable_vec = to_vec_with_cfg::<_, DefaultCfg>(&data).unwrap();

        assert_eq!(original_vec, configurable_vec);

        let mut buffer1 = Vec::new();
        let mut buffer2 = Vec::new();

        to_io(&data, &mut buffer1).unwrap();
        to_io_with_cfg::<_, _, DefaultCfg>(&data, &mut buffer2).unwrap();

        assert_eq!(buffer1, buffer2);
    }

    #[test]
    fn test_custom_config_types() {
        let data = TestEnum::Variant2(100);

        // Test with custom type aliases
        let with_ids = to_vec_with_cfg::<_, WithIdentifiers>(&data).unwrap();
        let without_ids = to_vec_with_cfg::<_, WithoutIdentifiers>(&data).unwrap();

        // Should produce different outputs
        assert_ne!(with_ids, without_ids);

        // WithIdentifiers should be same as DefaultCfg
        let default_cfg = to_vec_with_cfg::<_, DefaultCfg>(&data).unwrap();
        assert_eq!(with_ids, default_cfg);
    }
}

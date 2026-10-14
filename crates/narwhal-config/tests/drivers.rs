use narwhal_config::{DriverSettings, Settings};

#[test]
fn save_drivers_keeps_comments_and_other_keys() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(
        &path,
        "schema_version = 2\n\n# my theme\n[settings]\ntheme = \"light\"\n",
    )
    .unwrap();

    let mut settings = Settings::load(&path).unwrap();
    settings.drivers.enabled = Some(vec!["postgres".into(), "clickhouse".into()]);
    settings.save_drivers(&path).unwrap();

    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("# my theme"));
    assert!(text.contains("theme = \"light\""));
    let back = Settings::load(&path).unwrap();
    assert_eq!(
        back.drivers.allow_list(),
        Some(&["postgres".to_owned(), "clickhouse".to_owned()][..])
    );
}

#[test]
fn wildcard_means_every_driver() {
    let mut d = DriverSettings::default();
    assert_eq!(d.allow_list(), None);
    d.enabled = Some(vec![DriverSettings::ALL.into()]);
    assert_eq!(d.allow_list(), None);
}

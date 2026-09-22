mod cli {
    use assert_cmd::cargo_bin_cmd;
    use predicates::str::contains;

    #[tokio::test]
    async fn test_cli_aborts_without_keyfile() {
        let mut cmd = cargo_bin_cmd!();
        cmd.args(["list-inactive-identities"])
            .assert()
            .failure()
            .stderr(contains(
                "error: the following required arguments were not provided:\n  -k",
            ));
    }

    #[tokio::test]
    async fn test_cli_fails_with_dummy_keyfile() {
        let mut cmd = cargo_bin_cmd!();
        cmd.args(["list-inactive-identities", "-k", "key-dummy.json"])
            .assert()
            .failure()
            .stderr(contains("client error (Connect)"));
    }

    #[tokio::test]
    async fn test_cli_emmits_delete_warning_on_flag() {
        let mut cmd = cargo_bin_cmd!();
        cmd.args(["purge-inactive-identities", "-k", "key-dummy.json"])
            .assert()
            .failure()
            .stdout(contains("Are you sure you want to do this?"));
    }

    #[tokio::test]
    async fn all_subcommands_are_accepted() {
        let mut cmd = cargo_bin_cmd!();
        cmd.args(["list-inactive-identities"])
            .assert()
            .failure()
            .stderr(contains(
                "the following required arguments were not provided:",
            ));
        cmd = cargo_bin_cmd!();
        cmd.args(["purge-inactive-identities"])
            .assert()
            .failure()
            .stderr(contains(
                "the following required arguments were not provided:",
            ));
        cmd = cargo_bin_cmd!();
        cmd.args(["purge-pictures"])
            .assert()
            .failure()
            .stderr(contains(
                "the following required arguments were not provided:",
            ));
    }
}

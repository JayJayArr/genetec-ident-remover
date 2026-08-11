mod cli {
    use assert_cmd::cargo_bin_cmd;
    use predicates::str::contains;

    #[tokio::test]
    async fn test_cli_aborts_without_keyfile() {
        let mut cmd = cargo_bin_cmd!();
        cmd.assert().failure().stderr(contains(
            "error: the following required arguments were not provided:\n  -k",
        ));
    }

    #[tokio::test]
    async fn test_cli_fails_with_dummy_keyfile() {
        let mut cmd = cargo_bin_cmd!();
        cmd.args(&["-k", "key-dummy.json"])
            .assert()
            .failure()
            .stderr(contains("client error (Connect)"));
    }

    #[tokio::test]
    async fn test_cli_emmits_delete_warning_on_flag() {
        let mut cmd = cargo_bin_cmd!();
        cmd.args(&["-k","key-dummy.json","--delete"]).assert().failure().stdout(contains(" This runs destructive action, please run without --delete before running in destructive mode",));
    }
}

mod cli {
    use assert_cmd::cargo_bin_cmd;

    #[tokio::test]
    async fn test_cli_fails_without_keyfile() {
        let mut cmd = cargo_bin_cmd!();
        cmd.assert().failure();
    }
}

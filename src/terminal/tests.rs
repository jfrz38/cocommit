use super::ensure_interactive;

#[test]
fn accepts_interactive_standard_streams() {
    assert!(ensure_interactive(true, true).is_ok());
}

#[test]
fn identifies_each_non_interactive_stream() {
    assert_eq!(
        ensure_interactive(false, true)
            .expect_err("redirected standard input should fail")
            .to_string(),
        "standard input must be an interactive terminal"
    );
    assert_eq!(
        ensure_interactive(true, false)
            .expect_err("redirected standard output should fail")
            .to_string(),
        "standard output must be an interactive terminal"
    );
    assert_eq!(
        ensure_interactive(false, false)
            .expect_err("redirected streams should fail")
            .to_string(),
        "standard input and standard output must be interactive terminals"
    );
}

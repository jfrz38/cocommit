use std::sync::mpsc;

use super::{ensure_interactive, settle_updates};
use crate::{app::App, update_check::UpdateNotice};

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

#[test]
fn applies_a_finished_update_check_to_the_application() {
    let (sender, receiver) = mpsc::channel();
    let mut app = App::new(false);
    let mut updates = Some(receiver);

    settle_updates(&mut app, &mut updates);
    assert!(
        updates.is_some(),
        "an in-flight check should not settle the loop"
    );
    assert!(app.update_version().is_none());

    sender
        .send(UpdateNotice {
            latest: "0.3.0".to_owned(),
        })
        .expect("notice should be sent");
    settle_updates(&mut app, &mut updates);

    assert_eq!(app.update_version(), Some("0.3.0"));
    assert!(
        updates.is_none(),
        "a delivered notice should settle the loop"
    );

    settle_updates(&mut app, &mut updates);
    assert_eq!(
        app.update_version(),
        Some("0.3.0"),
        "a settled check should not disturb the notice"
    );
}

#[test]
fn settles_an_update_check_that_finished_without_a_notice() {
    let (sender, receiver) = mpsc::channel();
    let mut app = App::new(false);
    let mut updates = Some(receiver);
    drop(sender);

    settle_updates(&mut app, &mut updates);

    assert!(updates.is_none(), "a closed channel should settle the loop");
    assert!(app.update_version().is_none());
}

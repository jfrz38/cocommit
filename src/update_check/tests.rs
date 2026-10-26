use std::{
    cell::Cell,
    ffi::OsStr,
    fs,
    process::Command,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use super::*;

fn unix(seconds: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(seconds)
}

fn cache_file(directory: &tempfile::TempDir) -> std::path::PathBuf {
    directory.path().join("update-check.toml")
}

#[test]
fn parses_release_versions_with_an_optional_v_prefix() {
    assert_eq!(parse_version("0.2.3"), Some((0, 2, 3)));
    assert_eq!(parse_version("v1.20.300"), Some((1, 20, 300)));
    assert_eq!(parse_version(" 1.2.3 "), Some((1, 2, 3)));
}

#[test]
fn rejects_versions_that_are_not_plain_release_triplets() {
    for value in [
        "",
        "v",
        "0.2",
        "0.2.3.4",
        "0.2.3-beta",
        "0.2.3+build",
        "2.3.x",
        "1..3",
        "-1.2.3",
        "18446744073709551616.1.1",
    ] {
        assert_eq!(parse_version(value), None, "{value:?} should be rejected");
    }
}

#[test]
fn newest_tag_returns_the_highest_release_tag() {
    let output = "\
2f0a1b0c\trefs/tags/v0.2.3
9c8d7e6f\trefs/tags/v0.10.0
aa11bb22\trefs/tags/v0.9.9
";

    assert_eq!(newest_tag(output).as_deref(), Some("0.10.0"));
}

#[test]
fn newest_tag_ignores_unrelated_and_malformed_refs() {
    let output = "\
2f0a1b0c\trefs/tags/v0.3.0-beta
9c8d7e6f\trefs/tags/legacy
aa11bb22\trefs/heads/main
cc44dd55\trefs/tags/v0.2
";

    assert_eq!(newest_tag(output), None);
}

#[test]
fn notice_requires_a_strictly_newer_release() {
    let current = (0, 2, 3);

    assert_eq!(
        newer_notice("0.2.4", &current),
        Some(UpdateNotice {
            latest: "0.2.4".to_owned()
        })
    );
    assert_eq!(
        newer_notice("v1.0.0", &current),
        Some(UpdateNotice {
            latest: "1.0.0".to_owned()
        })
    );
    assert_eq!(newer_notice("0.2.3", &current), None);
    assert_eq!(newer_notice("0.1.9", &current), None);
    assert_eq!(newer_notice("nonsense", &current), None);
}

#[test]
fn fresh_success_cache_avoids_fetching() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let path = cache_file(&directory);
    write_cache(&path, Some("0.3.0"), unix(1_000));
    let fetched = Cell::new(false);

    let notice = check_with("0.2.3", Some(&path), unix(1_060), || {
        fetched.set(true);
        None
    });

    assert_eq!(
        notice,
        Some(UpdateNotice {
            latest: "0.3.0".to_owned()
        })
    );
    assert!(!fetched.get(), "a fresh cache should skip the network");
}

#[test]
fn expired_success_cache_fetches_and_replaces_the_outcome() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let path = cache_file(&directory);
    write_cache(&path, Some("0.3.0"), unix(1_000));
    let later = unix(1_000) + SUCCESS_TTL;

    let notice = check_with("0.2.3", Some(&path), later, || Some("0.4.0".to_owned()));

    assert_eq!(
        notice,
        Some(UpdateNotice {
            latest: "0.4.0".to_owned()
        })
    );
    assert_eq!(
        fresh_cache(&path, later).and_then(|cached| cached.latest),
        Some("0.4.0".to_owned())
    );
}

#[test]
fn successful_checks_stay_cached_for_a_day() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let path = cache_file(&directory);
    write_cache(&path, Some("0.3.0"), unix(0));

    assert!(
        fresh_cache(&path, unix(0) + SUCCESS_TTL - Duration::from_secs(1)).is_some(),
        "a successful check should stay fresh for the whole window"
    );
    assert!(
        fresh_cache(&path, unix(0) + SUCCESS_TTL).is_none(),
        "a successful check should expire after its window"
    );
}

#[test]
fn failed_checks_expire_sooner_than_successful_ones() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let path = cache_file(&directory);
    write_cache(&path, None, unix(0));

    assert!(FAILURE_TTL < SUCCESS_TTL);
    assert!(
        fresh_cache(&path, unix(0) + FAILURE_TTL - Duration::from_secs(1)).is_some(),
        "a failed check should stay fresh for its shorter window"
    );
    assert!(fresh_cache(&path, unix(0) + FAILURE_TTL).is_none());
}

#[test]
fn failed_checks_are_remembered_without_another_fetch() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let path = cache_file(&directory);
    write_cache(&path, None, unix(1_000));
    let fetched = Cell::new(false);

    let notice = check_with("0.2.3", Some(&path), unix(1_060), || {
        fetched.set(true);
        Some("0.3.0".to_owned())
    });

    assert_eq!(notice, None);
    assert!(
        !fetched.get(),
        "a remembered failure should skip the network"
    );
}

#[test]
fn missing_or_unreadable_cache_falls_back_to_the_fetch_result() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let path = cache_file(&directory);

    let notice = check_with("0.2.3", Some(&path), unix(1_000), || {
        Some("0.3.0".to_owned())
    });

    assert_eq!(
        notice,
        Some(UpdateNotice {
            latest: "0.3.0".to_owned()
        })
    );
    assert!(
        path.exists(),
        "the outcome should be persisted for the next start"
    );

    fs::write(&path, "not valid toml {").expect("cache should be corrupted");
    let notice = check_with("0.2.3", Some(&path), unix(1_000), || {
        Some("0.3.1".to_owned())
    });
    assert_eq!(
        notice,
        Some(UpdateNotice {
            latest: "0.3.1".to_owned()
        })
    );
}

#[test]
fn without_a_cache_location_the_check_still_reports_newer_releases() {
    let notice = check_with("0.2.3", None, unix(1_000), || Some("0.3.0".to_owned()));

    assert_eq!(
        notice,
        Some(UpdateNotice {
            latest: "0.3.0".to_owned()
        })
    );
}

#[test]
fn an_unparseable_running_version_skips_the_check_entirely() {
    let fetched = Cell::new(false);

    let notice = check_with("not-a-version", None, unix(1_000), || {
        fetched.set(true);
        Some("0.3.0".to_owned())
    });

    assert_eq!(notice, None);
    assert!(!fetched.get());
}

#[test]
fn background_checks_deliver_only_a_newer_release() {
    let receiver = spawn_check(|| {
        Some(UpdateNotice {
            latest: "0.3.0".to_owned(),
        })
    })
    .expect("background thread should start");

    let notice = receiver
        .recv_timeout(Duration::from_secs(5))
        .expect("the notice should arrive");

    assert_eq!(
        notice,
        UpdateNotice {
            latest: "0.3.0".to_owned()
        }
    );
    assert!(
        receiver.recv().is_err(),
        "the channel should close after the notice"
    );
}

#[test]
fn background_checks_close_the_channel_without_a_notice() {
    let receiver = spawn_check(|| None).expect("background thread should start");

    assert!(
        receiver.recv_timeout(Duration::from_secs(5)).is_err(),
        "a check without a newer release should only close the channel"
    );
}

#[test]
fn recognized_opt_out_values_disable_the_check() {
    assert!(disables(Some(OsStr::new("0"))));
    assert!(disables(Some(OsStr::new("false"))));
    assert!(disables(Some(OsStr::new(" OFF "))));
    assert!(disables(Some(OsStr::new("No"))));
    assert!(!disables(Some(OsStr::new("1"))));
    assert!(!disables(Some(OsStr::new("always"))));
    assert!(!disables(None));
    assert!(!enabled(false), "configuration should always win");
}

#[test]
fn reads_release_tags_from_a_repository() {
    let Ok(version) = Command::new("git").arg("--version").status() else {
        eprintln!("skipping update-check test because git --version could not run");
        return;
    };
    if !version.success() {
        eprintln!("skipping update-check test because git --version failed with {version}");
        return;
    }

    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let root = directory.path();
    for arguments in [
        ["init"].as_slice(),
        ["config", "user.email", "test@example.com"].as_slice(),
        ["config", "user.name", "Test User"].as_slice(),
        ["config", "commit.gpgsign", "false"].as_slice(),
    ] {
        assert!(
            Command::new("git")
                .args(arguments)
                .current_dir(root)
                .status()
                .expect("git should start")
                .success()
        );
    }
    fs::write(root.join("staged.txt"), "staged\n").expect("file should be written");
    for arguments in [
        ["add", "staged.txt"].as_slice(),
        ["commit", "-m", "initial"].as_slice(),
        ["tag", "v1.2.3"].as_slice(),
        ["tag", "v1.10.0"].as_slice(),
        ["tag", "not-a-version"].as_slice(),
    ] {
        assert!(
            Command::new("git")
                .args(arguments)
                .current_dir(root)
                .status()
                .expect("git should start")
                .success()
        );
    }
    let repository = root.to_str().expect("path should be UTF-8");

    let tags = list_tags(repository).expect("tags should be listed");

    assert_eq!(newest_tag(&tags).as_deref(), Some("1.10.0"));
}

#[test]
fn reports_a_subprocess_failure_for_an_unavailable_repository() {
    let Ok(version) = Command::new("git").arg("--version").status() else {
        eprintln!("skipping update-check test because git --version could not run");
        return;
    };
    if !version.success() {
        eprintln!("skipping update-check test because git --version failed with {version}");
        return;
    }

    let error = list_tags("this-repository-does-not-exist-cocommit-test")
        .expect_err("an unavailable repository should fail");

    assert!(format!("{error:#}").contains("git ls-remote exited"));
}

#[test]
fn output_wait_respects_an_expired_deadline() {
    let (_sender, receiver) = mpsc::channel();

    let error = receive_output(receiver, Instant::now())
        .expect_err("output that has not arrived by the deadline should fail");

    assert!(format!("{error:#}").contains("output arrived too late"));
}

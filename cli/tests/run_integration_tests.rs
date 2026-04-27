use std::io::Write;
use std::process::Command;

use tempfile::NamedTempFile;

fn gapline_bin() -> String {
    env!("CARGO_BIN_EXE_gapline").to_string()
}

fn create_valid_feed() -> NamedTempFile {
    let tmp = tempfile::Builder::new().suffix(".zip").tempfile().unwrap();
    let file = std::fs::File::create(tmp.path()).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let opts = zip::write::SimpleFileOptions::default();

    zip.start_file("agency.txt", opts).unwrap();
    zip.write_all(b"agency_id,agency_name,agency_url,agency_timezone\nA1,Agency,http://a.com,America/New_York\n").unwrap();

    zip.start_file("routes.txt", opts).unwrap();
    zip.write_all(
        b"route_id,agency_id,route_short_name,route_long_name,route_type\nR1,A1,1,Route One,3\n",
    )
    .unwrap();

    zip.start_file("trips.txt", opts).unwrap();
    zip.write_all(b"route_id,service_id,trip_id\nR1,S1,T1\n")
        .unwrap();

    zip.start_file("stops.txt", opts).unwrap();
    zip.write_all(b"stop_id,stop_name,stop_lat,stop_lon\nST1,Stop One,40.0,-74.0\nST2,Stop Two,40.01,-74.01\n")
        .unwrap();

    zip.start_file("stop_times.txt", opts).unwrap();
    zip.write_all(
        b"trip_id,arrival_time,departure_time,stop_id,stop_sequence\nT1,08:00:00,08:00:00,ST1,1\nT1,08:05:00,08:05:00,ST2,2\n",
    )
    .unwrap();

    zip.start_file("calendar.txt", opts).unwrap();
    zip.write_all(b"service_id,monday,tuesday,wednesday,thursday,friday,saturday,sunday,start_date,end_date\nS1,1,1,1,1,1,0,0,20240101,20241231\n").unwrap();

    zip.finish().unwrap();
    tmp
}

fn write_hw_file(content: &str) -> NamedTempFile {
    let mut tmp = tempfile::Builder::new().suffix(".gl").tempfile().unwrap();
    tmp.write_all(content.as_bytes()).unwrap();
    tmp.flush().unwrap();
    tmp
}

#[test]
fn run_feed_and_read() {
    let feed = create_valid_feed();
    let hw = write_hw_file(&format!("feed {}\nread stops\n", feed.path().display()));

    let output = Command::new(gapline_bin())
        .args(["run", hw.path().to_str().unwrap()])
        .output()
        .expect("failed to run gapline");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ST1"), "Output must contain stop ST1");
}

#[test]
fn run_comments_and_blank_lines_ignored() {
    let feed = create_valid_feed();
    let hw = write_hw_file(&format!(
        "# Load the feed\nfeed {}\n\n# Read stops\nread stops\n\n",
        feed.path().display()
    ));

    let output = Command::new(gapline_bin())
        .args(["run", hw.path().to_str().unwrap()])
        .output()
        .expect("failed to run gapline");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn run_echo_commands() {
    let feed = create_valid_feed();
    let hw = write_hw_file(&format!("feed {}\nread stops\n", feed.path().display()));

    let output = Command::new(gapline_bin())
        .args(["run", hw.path().to_str().unwrap()])
        .output()
        .expect("failed to run gapline");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("[1] feed"), "First command must be echoed");
    assert!(
        stderr.contains("[2] read stops"),
        "Second command must be echoed"
    );
}

#[test]
fn run_no_feed_before_command() {
    let hw = write_hw_file("read stops\n");

    let output = Command::new(gapline_bin())
        .args(["run", hw.path().to_str().unwrap()])
        .output()
        .expect("failed to run gapline");

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No feed loaded"),
        "Must mention no feed loaded. Got: {stderr}"
    );
}

#[test]
fn run_delete_without_confirm() {
    let feed = create_valid_feed();
    let hw = write_hw_file(&format!(
        "feed {}\ndelete stops --where stop_id=ST1\n",
        feed.path().display()
    ));

    let output = Command::new(gapline_bin())
        .args(["run", hw.path().to_str().unwrap()])
        .output()
        .expect("failed to run gapline");

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--confirm"),
        "Must mention --confirm. Got: {stderr}"
    );
}

#[test]
fn run_create_without_confirm() {
    let feed = create_valid_feed();
    let hw = write_hw_file(&format!(
        "feed {}\ncreate stops --set stop_id=NEW stop_name=Test stop_lat=40.0 stop_lon=-74.0\n",
        feed.path().display()
    ));

    let output = Command::new(gapline_bin())
        .args(["run", hw.path().to_str().unwrap()])
        .output()
        .expect("failed to run gapline");

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--confirm"), "Got: {stderr}");
}

#[test]
fn run_nonexistent_hw_file() {
    let output = Command::new(gapline_bin())
        .args(["run", "/tmp/nonexistent_test_file.gl"])
        .output()
        .expect("failed to run gapline");

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("File not found"),
        "Must report file not found. Got: {stderr}"
    );
}

#[test]
fn run_unknown_command() {
    let hw = write_hw_file("foobar some args\n");

    let output = Command::new(gapline_bin())
        .args(["run", hw.path().to_str().unwrap()])
        .output()
        .expect("failed to run gapline");

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Parse error at line 1") && stderr.contains("unknown command 'foobar'"),
        "Must report parse error with line number. Got: {stderr}"
    );
}

#[test]
fn run_save_without_path_overwrites_original() {
    let feed = create_valid_feed();
    let tmp_feed = tempfile::Builder::new().suffix(".zip").tempfile().unwrap();
    std::fs::copy(feed.path(), tmp_feed.path()).unwrap();

    let hw = write_hw_file(&format!(
        "feed {}\ncreate stops --set stop_id=NEW stop_name=NewStop stop_lat=41.0 stop_lon=-73.0 --confirm\nsave\n",
        tmp_feed.path().display()
    ));

    let output = Command::new(gapline_bin())
        .args(["run", hw.path().to_str().unwrap()])
        .output()
        .expect("failed to run gapline");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let verify_hw = write_hw_file(&format!(
        "feed {}\nread stops --where stop_id=NEW\n",
        tmp_feed.path().display()
    ));

    let verify = Command::new(gapline_bin())
        .args(["run", verify_hw.path().to_str().unwrap()])
        .output()
        .expect("failed to run gapline");

    let stdout = String::from_utf8_lossy(&verify.stdout);
    assert!(
        stdout.contains("NEW"),
        "Saved feed must contain the new stop. Got: {stdout}"
    );
}

#[test]
fn run_save_with_path() {
    let feed = create_valid_feed();
    let output_feed = tempfile::Builder::new().suffix(".zip").tempfile().unwrap();

    let hw = write_hw_file(&format!(
        "feed {}\ncreate stops --set stop_id=S99 stop_name=Saved stop_lat=42.0 stop_lon=-72.0 --confirm\nsave {}\n",
        feed.path().display(),
        output_feed.path().display()
    ));

    let output = Command::new(gapline_bin())
        .args(["run", hw.path().to_str().unwrap()])
        .output()
        .expect("failed to run gapline");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let verify_hw = write_hw_file(&format!(
        "feed {}\nread stops --where stop_id=S99\n",
        output_feed.path().display()
    ));

    let verify = Command::new(gapline_bin())
        .args(["run", verify_hw.path().to_str().unwrap()])
        .output()
        .expect("failed to run gapline");

    let stdout = String::from_utf8_lossy(&verify.stdout);
    assert!(
        stdout.contains("S99"),
        "Output feed must contain S99. Got: {stdout}"
    );
}

#[test]
fn run_stop_on_first_error() {
    let feed = create_valid_feed();
    let hw = write_hw_file(&format!(
        "feed {}\ncreate stops --set bad_field=x --confirm\nread stops\n",
        feed.path().display()
    ));

    let output = Command::new(gapline_bin())
        .args(["run", hw.path().to_str().unwrap()])
        .output()
        .expect("failed to run gapline");

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("[1]"));
    assert!(stderr.contains("[2]"), "Failing command must be echoed");
    assert!(
        !stderr.contains("[3]"),
        "Commands after failure must not run"
    );
}

#[test]
fn run_multiple_feed_directives() {
    let feed_a = create_valid_feed();
    let feed_b = create_valid_feed();

    let hw = write_hw_file(&format!(
        "feed {}\nread stops\nfeed {}\nread stops\n",
        feed_a.path().display(),
        feed_b.path().display()
    ));

    let output = Command::new(gapline_bin())
        .args(["run", hw.path().to_str().unwrap()])
        .output()
        .expect("failed to run gapline");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("[3] feed"),
        "Second feed directive must be echoed"
    );
}

#[test]
fn run_delete_with_confirm_and_save() {
    let feed = create_valid_feed();
    let output_feed = tempfile::Builder::new().suffix(".zip").tempfile().unwrap();

    let hw = write_hw_file(&format!(
        "feed {}\ndelete stops --where stop_id=ST2 --confirm\nsave {}\n",
        feed.path().display(),
        output_feed.path().display()
    ));

    let output = Command::new(gapline_bin())
        .args(["run", hw.path().to_str().unwrap()])
        .output()
        .expect("failed to run gapline");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let verify_hw = write_hw_file(&format!(
        "feed {}\nread stops\n",
        output_feed.path().display()
    ));

    let verify = Command::new(gapline_bin())
        .args(["run", verify_hw.path().to_str().unwrap()])
        .output()
        .expect("failed to run gapline");

    let stdout = String::from_utf8_lossy(&verify.stdout);
    assert!(stdout.contains("ST1"), "ST1 must still exist");
    assert!(!stdout.contains("ST2"), "ST2 must be gone");
}

#[test]
fn run_update_with_confirm_and_save() {
    let feed = create_valid_feed();
    let output_feed = tempfile::Builder::new().suffix(".zip").tempfile().unwrap();

    let hw = write_hw_file(&format!(
        "feed {}\nupdate stops --where stop_id=ST1 --set stop_name=\"Corrected Name\" --confirm\nsave {}\n",
        feed.path().display(),
        output_feed.path().display()
    ));

    let output = Command::new(gapline_bin())
        .args(["run", hw.path().to_str().unwrap()])
        .output()
        .expect("failed to run gapline");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let verify_hw = write_hw_file(&format!(
        "feed {}\nread stops --where stop_id=ST1\n",
        output_feed.path().display()
    ));

    let verify = Command::new(gapline_bin())
        .args(["run", verify_hw.path().to_str().unwrap()])
        .output()
        .expect("failed to run gapline");

    let stdout = String::from_utf8_lossy(&verify.stdout);
    assert!(
        stdout.contains("Corrected Name"),
        "Stop must have updated name. Got: {stdout}"
    );
    assert!(
        !stdout.contains("Stop One"),
        "Old name must be gone. Got: {stdout}"
    );
}

#[test]
fn run_full_workflow_validate_create_validate_save() {
    let feed = create_valid_feed();
    let output_feed = tempfile::Builder::new().suffix(".zip").tempfile().unwrap();

    let hw = write_hw_file(&format!(
        "feed {}\nvalidate\ncreate stops --set stop_id=ST3 stop_name=\"New Stop\" stop_lat=40.02 stop_lon=-74.02 --confirm\nvalidate\nsave {}\n",
        feed.path().display(),
        output_feed.path().display()
    ));

    let output = Command::new(gapline_bin())
        .args(["run", hw.path().to_str().unwrap()])
        .output()
        .expect("failed to run gapline");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("[1]"));
    assert!(stderr.contains("[5]"), "All 5 commands must execute");
}

#[test]
fn run_validate_in_batch() {
    let feed = create_valid_feed();

    let hw = write_hw_file(&format!("feed {}\nvalidate\n", feed.path().display()));

    let output = Command::new(gapline_bin())
        .args(["run", hw.path().to_str().unwrap()])
        .output()
        .expect("failed to run gapline");

    assert!(
        output.status.success(),
        "Valid feed must pass validation.\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

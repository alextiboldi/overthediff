//! Background tasks: cancel at a safe point, per-item errors and how a task ends. Each test steps
//! the task's thread by hand through channels, never by time.

use std::sync::mpsc;

use otd_core::task::{End, Task};

#[test]
fn a_cancelled_task_stops_at_its_next_safe_point() {
    let (reached, progress) = mpsc::channel();
    let (go, wait_for_go) = mpsc::channel::<()>();
    let task = Task::spawn("Count", true, move |worker| {
        for done in 1..=10 {
            if worker.cancelled() {
                return;
            }
            worker.progress(done, Some(10));
            let _ = reached.send(done);
            let _ = wait_for_go.recv();
        }
    });
    assert_eq!(progress.recv(), Ok(1));
    task.cancel();
    go.send(()).unwrap();

    let outcome = task.wait();
    assert_eq!(outcome.end, End::Cancelled);
    let total = outcome.progress.total.unwrap();
    assert!(outcome.progress.done < total, "it stopped midway: {:?}", outcome.progress);
}

#[test]
fn a_task_that_never_saw_the_cancel_request_is_not_reported_cancelled() {
    let (go, wait_for_go) = mpsc::channel::<()>();
    let task = Task::spawn("Uninterruptible step", true, move |worker| {
        let _ = wait_for_go.recv();
        worker.progress(1, Some(1));
    });
    task.cancel();
    go.send(()).unwrap();
    assert_eq!(task.wait().end, End::Completed);
}

#[test]
fn a_task_with_three_failing_items_lists_all_three_in_order() {
    let task = Task::spawn("Copy", true, |worker| {
        for item in ["a", "b", "c", "d", "e"] {
            if matches!(item, "a" | "c" | "e") {
                worker.fail(item, "Access denied");
            }
        }
    });
    let outcome = task.wait();
    assert_eq!(outcome.end, End::Failed);
    let items: Vec<_> = outcome.errors.iter().map(|error| error.item.as_str()).collect();
    assert_eq!(items, ["a", "c", "e"]);
}

#[test]
fn a_task_that_panics_ends_failed_instead_of_running_forever() {
    let task = Task::spawn("Broken", false, |_| panic!("test panic"));
    let outcome = task.wait();
    assert_eq!(outcome.end, End::Failed);
    assert!(!outcome.errors.is_empty(), "the summary says why it failed");
}

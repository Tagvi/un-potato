extern crate notify_rust;
extern crate tokio;
use notify_rust::{Notification, Urgency};
use std::time::Duration;
use tokio::{join, spawn, time};

#[tokio::main]
async fn main() {
    let posture_notification = RecurringNotification::new(
        "Fix your posture",
        get_seconds_from_minutes(5.0),
        Urgency::Normal,
    );
    let eyes_notification = RecurringNotification::new(
        "Time to let your eyes rest",
        get_seconds_from_minutes(30.0),
        Urgency::Normal,
    );
    let eyedrops_notification = RecurringNotification::new(
        "Time to use eyedrops",
        get_seconds_from_minutes(60.0),
        Urgency::Critical,
    );
    let (r1, r2, r3) = join!(
        spawn(posture_notification.start()),
        spawn(eyes_notification.start()),
        spawn(eyedrops_notification.start())
    );
    r1.unwrap();
    r2.unwrap();
    r3.unwrap();
}

struct RecurringNotification {
    notification: Notification,
    time: Duration,
}

impl RecurringNotification {
    fn new(summary: &str, seconds: u64, urgency: Urgency) -> RecurringNotification {
        let mut notification = Notification::new();
        notification.summary(summary).urgency(urgency);
        RecurringNotification {
            time: Duration::from_secs(seconds),
            notification,
        }
    }
    async fn start(self) {
        let mut interval = time::interval(self.time);
        loop {
            interval.tick().await;
            self.notification.show().unwrap();
        }
    }
}

fn get_seconds_from_minutes(minutes: f64) -> u64 {
    (minutes * 60.0) as u64
}

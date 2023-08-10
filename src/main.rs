use notify_rust::{Notification, Urgency};
use rodio::Sink;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::{task::JoinSet, time};

struct NotificationsManager {
    notifications: Vec<RecurringNotification>,
    threads: JoinSet<()>,
    running_notifications: Arc<Mutex<Vec<()>>>,
    sink: Arc<Mutex<Sink>>,
    file: Arc<&'static [u8]>,
    _handles: Arc<(rodio::OutputStream, rodio::OutputStreamHandle)>,
}
impl NotificationsManager {
    fn new() -> NotificationsManager {
        // Will not work for windows, but idc
        let file = include_bytes!("./bundle/alarm.mp3");
        let stream = Arc::new(rodio::OutputStream::try_default().unwrap());
        let stream_handle = &*Arc::clone(&stream);
        let sink = Sink::try_new(&stream_handle.1).unwrap();
        NotificationsManager {
            notifications: vec![],
            threads: JoinSet::new(),
            running_notifications: Arc::new(Mutex::new(Vec::new())),
            file: Arc::new(file),
            sink: Arc::new(Mutex::new(sink)),
            _handles: stream,
        }
    }
    async fn start(mut self) {
        for notification in self.notifications.clone() {
            self.run_task(notification);
        }
        self.threads.join_next().await;
    }
    fn push_task(&mut self, notification: RecurringNotification) {
        self.notifications.push(notification);
    }
    fn run_task(&mut self, notification: RecurringNotification) {
        let sink = Arc::clone(&self.sink);
        let running_notifications = Arc::clone(&self.running_notifications);
        let file = Arc::clone(&self.file);
        self.threads.spawn(async move {
            loop {
                let file = std::io::Cursor::new(*file);
                let source = rodio::Decoder::new(file).unwrap();
                sink.lock().unwrap().append(source);
                sink.lock().unwrap().play();
                running_notifications.lock().unwrap().push(());
                notification.show(|| {
                    let mut running_notifications = running_notifications.lock().unwrap();
                    running_notifications.pop().unwrap();
                    if running_notifications.len() == 0 {
                        sink.lock().unwrap().stop();
                    }
                });
                time::sleep(notification.time).await;
            }
        });
    }
}

#[tokio::main]
async fn main() {
    let posture_notification = RecurringNotification::new(
        "Fix your posture",
        get_seconds_from_minutes(5.0),
        Urgency::Critical,
    );
    let eyes_notification = RecurringNotification::new(
        "Time to let your eyes rest (5 Min)",
        get_seconds_from_minutes(60.0),
        Urgency::Critical,
    );
    let eyedrops_notification = RecurringNotification::new(
        "Time to use eyedrops",
        get_seconds_from_minutes(90.0),
        Urgency::Critical,
    );
    let mut manager = NotificationsManager::new();
    manager.push_task(posture_notification);
    manager.push_task(eyes_notification);
    manager.push_task(eyedrops_notification);
    manager.start().await
}

#[derive(Clone)]
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
    fn show<F: FnMut() -> ()>(&self, mut on_close: F) {
        self.notification
            .show()
            .unwrap()
            .wait_for_action(|action| match action {
                "__closed" => on_close(),
                _ => (),
            });
    }
}

fn get_seconds_from_minutes(minutes: f64) -> u64 {
    (minutes * 60.0) as u64
}

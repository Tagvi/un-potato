use clap::Parser;
use config::Config;
use tokio::task::JoinSet;
mod config;

#[derive(Parser)] // requires `derive` feature
#[command(name = "un-potato")]
#[command(bin_name = "un-potato")]
enum Cli {
    /// Add a notification
    Add(AddArgs),
    /// Remove a notification
    Remove(RemoveArgs),
    /// List notifications
    List,
    /// Run notifications according to their intervals
    Run,
}

#[derive(clap::Args)]
#[command(author, version, about, long_about = None)]
struct AddArgs {
    /// Interval between notifications (int x)(ms|s|m|h|d|w) e.g. 1h,2s,300ms, etc
    interval: String,
    /// Notification text
    text: String,
}

#[derive(clap::Args)]
#[command(author, version, about, long_about = None)]
struct RemoveArgs {
    /// Notification id, can be found by listing the notifications
    id: usize,
}

#[tokio::main]
async fn main() {
    let stuff = Cli::parse();
    Config::ensure_config_dir_exists().unwrap();
    match stuff {
        Cli::Add(args) => {
            Config::write_notification_to_config(
                Config::parse_interval(&args.interval).unwrap().1,
                args.text.clone(),
            )
            .unwrap();
        }
        Cli::Remove(args) => {
            Config::remove_notification_from_config(args.id).unwrap();
        }
        Cli::List => {
            let mut config = Config::new();
            config.load().unwrap();
            for (i, notification) in config.notifications.iter().enumerate() {
                println!("* {i}: {} {}", notification.interval, notification.text);
            }
        }
        Cli::Run => {
            let mut config = config::Config::new();
            config.load().unwrap();
            let mut set = JoinSet::new();
            for notification in config.notifications.into_iter() {
                set.spawn(async move {
                    let mut interval = tokio::time::interval(notification.interval.into());
                    interval.tick().await;
                    loop {
                        interval.tick().await;
                        notification.show()
                    }
                });
            }
            while let Some(_) = set.join_next().await {}
        }
    };
}

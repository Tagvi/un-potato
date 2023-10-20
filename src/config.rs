use dirs::config_dir;
use fs::File;
use nom::bytes::complete::take_while;
use nom::character::complete;
use nom::multi::separated_list0;
use nom::IResult;
use notify_rust::{Hint, Notification, Urgency};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug)]
pub struct NotificationConfig {
    pub interval: Time,
    pub text: String,
    notify_rust_notification: Notification,
}
impl NotificationConfig {
    pub fn show(&self) {
        self.notify_rust_notification.show().unwrap();
    }
    pub fn new(interval: Time, text: String) -> Self {
        Self {
            text: text.clone(),
            interval,
            notify_rust_notification: Notification::new()
                .appname("un-potato")
                .summary(&text)
                .urgency(Urgency::Critical)
                .sound_name("alarm-clock-elapsed")
                .finalize(),
        }
    }
}
#[derive(Debug)]
pub struct Config {
    pub notifications: Vec<NotificationConfig>,
    path: PathBuf,
}
impl Config {
    pub fn ensure_config_dir_exists() -> io::Result<()> {
        let path = Self::get_path();
        let path = path.parent().unwrap();
        fs::create_dir_all(path)?;
        Ok(())
    }
    fn get_path() -> PathBuf {
        config_dir()
            .unwrap()
            .join("un-potato")
            .join("notifications")
    }
    pub fn new() -> Self {
        Self {
            path: Self::get_path(),
            notifications: Vec::new(),
        }
    }
    pub fn load(&mut self) -> io::Result<()> {
        let path = self.path.as_path();
        match fs::read_to_string(path) {
            Ok(config) => {
                self.notifications.append(
                    &mut separated_list0(complete::newline, Self::config_line_parser)(&config)
                        .unwrap()
                        .1,
                );
                Ok(())
            }
            Err(e) => {
                if e.kind() == io::ErrorKind::NotFound {
                    File::create(path)?;
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }
    fn config_line_parser(line: &str) -> IResult<&str, NotificationConfig> {
        let (input, interval) = Self::parse_interval(line)?;
        let (input, _) = complete::multispace1(input)?;
        let (input, text) = take_while(|c| c != '\n')(input)?;
        Ok((input, NotificationConfig::new(interval, text.to_owned())))
    }
    pub fn write_notification_to_config(interval: Time, text: String) -> io::Result<()> {
        let path = Self::get_path();
        let path = path.as_path();
        match OpenOptions::new().append(true).open(path) {
            Ok(mut file) => {
                writeln!(file, "{interval} {text}")
            }
            Err(e) => {
                if e.kind() == io::ErrorKind::NotFound {
                    let mut file = File::create(path)?;
                    file.write_all(format!("{interval} {text}\n").as_bytes())
                } else {
                    Err(e)
                }
            }
        }
    }
    pub fn parse_interval(input: &str) -> IResult<&str, Time> {
        let (input, number) = complete::u64(input)?;
        let (input, time_str) = complete::alpha1(input)?;
        Ok((
            input,
            match time_str {
                "ms" => Time::Milliseconds(number),
                "s" => Time::Seconds(number),
                "m" => Time::Minutes(number),
                "h" => Time::Hours(number),
                "d" => Time::Days(number),
                "w" => Time::Weeks(number),
                _ => panic!("Encountered an invalid time unit in the config"),
            },
        ))
    }
    pub fn remove_notification_from_config(index: usize) -> io::Result<()> {
        let path = Self::get_path();
        let path = path.as_path();
        let config = fs::read_to_string(path)?;
        let mut config_write = File::create(path)?;
        for (i, line) in config.lines().enumerate() {
            if i != index {
                writeln!(config_write, "{}", line)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Time {
    Milliseconds(u64),
    Seconds(u64),
    Minutes(u64),
    Hours(u64),
    Days(u64),
    Weeks(u64),
}

use std::fmt::Display;
use std::io;

impl Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Milliseconds(n) => write!(f, "{n}ms"),
            Self::Seconds(n) => write!(f, "{n}s"),
            Self::Minutes(n) => write!(f, "{n}m"),
            Self::Hours(n) => write!(f, "{n}h"),
            Self::Days(n) => write!(f, "{n}d"),
            Self::Weeks(n) => write!(f, "{n}w"),
        }
    }
}

use std::time::Duration;
impl Into<Duration> for &Time {
    fn into(self) -> Duration {
        let minute = 60;
        let hour = minute * 60;
        let day = hour * 24;
        let week = day * 7;
        match *self {
            Time::Milliseconds(n) => Duration::from_millis(n),
            Time::Seconds(n) => Duration::from_secs(n),
            Time::Minutes(n) => Duration::from_secs(n * minute),
            Time::Hours(n) => Duration::from_secs(n * hour),
            Time::Days(n) => Duration::from_secs(n * day),
            Time::Weeks(n) => Duration::from_secs(n * week),
        }
    }
}
impl Into<Duration> for Time {
    fn into(self) -> Duration {
        (&self).into()
    }
}

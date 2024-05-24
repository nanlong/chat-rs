use std::fmt;

#[derive(Debug)]
pub enum Message {
    Joined(String),
    Left(String),
    Chat { username: String, message: String },
}

impl Message {
    pub fn joined(username: impl Into<String>) -> Self {
        Message::Joined(username.into())
    }

    pub fn left(username: impl Into<String>) -> Self {
        Message::Left(username.into())
    }

    pub fn chat(username: impl Into<String>, message: impl Into<String>) -> Self {
        Message::Chat {
            username: username.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Message::Joined(username) => write!(f, "{} has joined the chat", username),
            Message::Left(username) => write!(f, "{} has left the chat", username),
            Message::Chat { username, message } => write!(f, "{}: {}", username, message),
        }
    }
}

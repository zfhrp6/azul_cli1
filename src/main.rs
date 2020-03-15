extern crate azul;
#[macro_use]
extern crate serde_derive;
extern crate toml;

use azul::{
    prelude::*,
    widgets::button::Button,
    widgets::text_input::{TextInput, TextInputState},
};
use chrono::prelude::*;
use oauth_client::Token;
use std::borrow::Borrow;
use std::fmt;
use std::fs;
use std::io::{BufReader, Read};
use tokio;

struct MyDataModel {
    time: DateTime<Utc>,
    text: TextInputState,
    account: User,
}

impl Layout for MyDataModel {
    fn layout(&self, _: LayoutInfo<Self>) -> Dom<Self> {
        Dom::div()
            .with_child(Dom::label(format!("{}", self.time.to_rfc3339())))
            .with_child(
                TextInput::new()
                    .dom(&self.text)
                    .with_id("hoge")
                    .with_callback(On::TextInput, |mut callback: CallbackInfo<Self>| {
                        input(&mut callback)
                    }),
            )
            .with_child(
                Button::with_label("hoge")
                    .dom()
                    .with_callback(On::LeftMouseDown, |mut callback: CallbackInfo<Self>| {
                        update(&mut callback)
                    }),
            )
    }
}

fn main() {
    let mut s = String::new();
    let mut br = fs::File::open("user.toml")
        .map(|f| BufReader::new(f))
        .map_err(|e| e.to_string())
        .unwrap();

    br.read_to_string(&mut s).unwrap();

    let creds: Conf = match toml::from_str(&s) {
        Ok(s) => s,
        Err(e) => panic!("config file cannot be loaded. {}", e),
    };

    let mut app = App::new(
        MyDataModel {
            time: Utc::now(),
            text: TextInputState::new(" "),
            account: creds.account,
        },
        AppConfig::default(),
    )
    .unwrap();
    let window = app
        .create_window(WindowCreateOptions::default(), css::native())
        .unwrap();
    app.run(window).unwrap();
}

fn update(event: &mut CallbackInfo<MyDataModel>) -> UpdateScreen {
    event.state.data.time = get_time();
    hoge(
        &event.state.data.text.borrow().text,
        &event.state.data.account,
    );
    event.state.data.text = TextInputState::new(" ");
    Redraw
}

fn input(event: &mut CallbackInfo<MyDataModel>) -> UpdateScreen {
    event.state.data.text.text = format!(
        "{}{}",
        event.state.data.text.text,
        event
            .get_keyboard_state()
            .current_char
            .expect("none char")
            .to_string()
    );
    Redraw
}

fn hoge(text: &str, account: &User) {
    let ct = Token::new(&account.app_key, &account.app_secret);
    let at = Token::new(&account.oauth_token, &account.oauth_token_secret);

    let _ = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(twitter_api::update_status(
            &ct,
            &at,
            format!("{}", text).borrow(),
        ));
}

fn get_time() -> DateTime<Utc> {
    Utc::now()
}

#[derive(Deserialize, Debug)]
struct User {
    screen_name: Option<String>,
    app_key: String,
    app_secret: String,
    oauth_token: String,
    oauth_token_secret: String,
}

#[derive(Deserialize, Debug)]
struct Conf {
    account: User,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}, {}, {}, {}, {}",
            match &self.screen_name {
                Some(s) => s,
                None => "",
            },
            &self.app_key,
            &self.app_secret,
            &self.oauth_token,
            &self.oauth_token_secret
        )
    }
}

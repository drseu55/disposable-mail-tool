use owo_colors::OwoColorize;
use tokio;

mod mails;

const BANNER: &str = r#"
 _____  _           _____                              _       _     _     _             _             
|_   _| |__   ___  | ____|_ __   ___ _ __ ___  _   _  (_)___  | |   (_)___| |_ ___ _ __ (_)_ __   __ _ 
  | | | '_ \ / _ \ |  _| | '_ \ / _ \ '_ ` _ \| | | | | / __| | |   | / __| __/ _ \ '_ \| | '_ \ / _` |
  | | | | | |  __/ | |___| | | |  __/ | | | | | |_| | | \__ \ | |___| \__ \ ||  __/ | | | | | | | (_| |
  |_| |_| |_|\___| |_____|_| |_|\___|_| |_| |_|\__, | |_|___/ |_____|_|___/\__\___|_| |_|_|_| |_|\__, |
                                               |___/                                             |___/                                                                                                                     
"#;

fn main() {
    println!("{}", BANNER.fg_rgb::<0x2E, 0x31, 0x92>());
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        mails::GuerrillaMail::create_new_email().await;
    })
}

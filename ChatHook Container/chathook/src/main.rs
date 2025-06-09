#![allow(non_snake_case)]

use std::env;
use std::{net::{SocketAddr, UdpSocket}};
use std::collections::HashMap;
use jzon;
use base64::{Engine as _, engine::{general_purpose}};
use anyhow::{Result, anyhow};
use discord_webhook_rs as webhook;
use webhook::{Webhook, Embed, Field};

const AVATAR_URL: &str = "https://apache.neverless.dev/random/dc_logo.jpg";

/*
	Format
	[type] = int
	[server_name] = string
	[content] = object any
*/
struct Message {
	pub m_type: i32,
	pub from_server: String,
	pub add_server_name: bool,
	pub content: jzon::object::Object,
}

/*
	Format
	[player_name] = string
	[chat_message] = string
*/
struct Chat {
	pub player_name: String,
	pub chat_message: String,
}

struct PlayerJoin {
	pub player_name: String,
	pub profile_pic_url: String,
	pub profile_color: u32,
}

struct PlayerLeft {
	pub player_name: String,
}

fn main() -> Result<()> {
	let webhook_url = env::var("WEBHOOK_URL").unwrap();
	let udp_port: u16 = env::var("UDP_PORT").unwrap().parse::<u16>().unwrap();

	println!("Listening to 0.0.0.0:{}", &udp_port);
	println!("Sending to: {}", &webhook_url);

    let mut profile_cache: HashMap<String, String> = HashMap::new();
	let mut last_server_name = String::new();
	let socket = openUdpListener(udp_port, false)?;

	let _ = defaultWebhookHeader(&webhook_url)
		.content(&format!("🌺 ***Hello from BeamMP ChatHook o/***"))
		.send(); // we let it fail

	loop {
		match udpTryReceive(&socket) {
			Ok(receive) => {
				match decodeReceive(&receive) {
					Ok(message) => handleMessage(&webhook_url, message, &mut last_server_name, &mut profile_cache),
					Err(e) => eprintln!("{}", e)
				}
			},
			Err(e) => eprintln!("{}", e)
		}
	}
}

// --------------------------------------------------------------------------------
// Handle and do stuff
fn handleMessage(webhook_url: &str, mut message: Message, last_server_name: &mut String, profile_cache: &mut HashMap<String, String>) {
	if *last_server_name != message.from_server {
		message.add_server_name = true;
		*last_server_name = message.from_server.clone();
	}

	match message.m_type {
		1 => {
			if let Ok(chat) = decodeChatMessage(&message) {
				if let Err(e) = sendChatMessage(webhook_url, &message, chat) {
					eprintln!("{:?}", e);
				}
			} else {
				eprintln!("Invalid format for chat message from {}", &message.from_server);
			}
		},
		2 => {
			if let Err(e) = sendServerOnline(webhook_url, &message) {
				eprintln!("{:?}", e);
			}
		},
		3 => {
			if let Ok(player) = decodePlayerJoin(&message, profile_cache) {
				if let Err(e) = sendPlayerJoin(webhook_url, &message, player) {
					eprintln!("{:?}", e);
				}
			} else {
				eprintln!("Invalid format for player join from {}", &message.from_server);
			}
		}
		4 => {
			if let Ok(player) = decodePlayerLeft(&message) {
				if let Err(e) = sendPlayerLeft(webhook_url, &message, player) {
					eprintln!("{:?}", e);
				}
			} else {
				eprintln!("Invalid format for player left from {}", &message.from_server);
			}
		}
		_ => {
			eprintln!("Invalid Option from {}", &message.from_server)
		}
	}
}

fn serverNameHeader(content: &mut String, server_name: &str) {
	content.push_str(&format!("> ↓↓↓ __**{}**__ ↓↓↓\n", server_name));
}

fn defaultWebhookHeader(webhook_url: &str) -> Webhook {
	Webhook::new(webhook_url)
		.username("BeamMP ChatHook")
		.avatar_url(AVATAR_URL)
}

fn sendPlayerJoin(webhook_url: &str, message: &Message, player: PlayerJoin) -> Result<(), webhook::Error> {
	let mut content = String::new();
	if message.add_server_name {serverNameHeader(&mut content, &message.from_server);}
	defaultWebhookHeader(webhook_url)
		.content(content)
		.add_embed(
			Embed::new()
				.thumbnail(player.profile_pic_url)
				.color(player.profile_color)
				.add_field(
					Field::new()
						.name("🧡 New Player Join!")
						.value(format!("→ [{}](https://forum.beammp.com/u/{})", &player.player_name, &player.player_name))
				)
		).send()?;

	Ok(())
}

fn sendPlayerLeft(webhook_url: &str, message: &Message, player: PlayerLeft) -> Result<(), webhook::Error> {
	let mut content = String::new();
	if message.add_server_name {serverNameHeader(&mut content, &message.from_server);}
	content.push_str(&format!("> - 🕵️ ***{}** left*", &player.player_name));
	defaultWebhookHeader(webhook_url)
		.content(content)
		.send()?;
	
	Ok(())
}

fn sendChatMessage(webhook_url: &str, message: &Message, chat: Chat) -> Result<(), webhook::Error> {
	let mut content = String::new();
	if message.add_server_name {serverNameHeader(&mut content, &message.from_server);}
	content.push_str(&format!("> - 💬 **{}:** {}", &chat.player_name, &chat.chat_message));
	defaultWebhookHeader(webhook_url)
		.content(content)
		.send()?;


    Ok(())
}

fn sendServerOnline(webhook_url: &str, message: &Message) -> Result <(), webhook::Error> {
	let mut content = String::new();
	if message.add_server_name {serverNameHeader(&mut content, &message.from_server);}
	content.push_str(&format!("## ✅ **Server has just (re)started!**"));
	defaultWebhookHeader(webhook_url)
		.content(content)
		.send()?;
	
	Ok(())
}

// --------------------------------------------------------------------------------
// Profile pic cache
fn evalProfilePicture(player_name: &str, profile_cache: &mut HashMap<String, String>) -> String {
	if profile_cache.contains_key(player_name) {return profile_cache.get(player_name).unwrap().to_string()}

	let mut profile_pic_url = String::new();
	let url = "https://forum.beammp.com/u/".to_string() + player_name + ".json";

	if let Ok(client) = get_reqwest_client() {
		if let Ok(body) = client.get(url).send() {
			if let Ok(text) = body.text() {
				if let Ok(decode) = jzon::parse(&text) {
					if decode["user"].is_object() && decode["user"]["avatar_template"].is_string() {
						let url = "https://forum.beammp.com".to_string() + decode["user"]["avatar_template"].as_str().unwrap();
						let url = url.replace("{size}", "144");

						profile_pic_url.insert_str(0, &url);
						profile_cache.insert(player_name.to_string(), url);
					}
				}
			}
		}
	}

	profile_pic_url
}

// --------------------------------------------------------------------------------
// Decode
fn decodeReceive(message: &str) -> Result<Message> {
	let decode = jzon::parse(message)?;
	if !decode.is_object() {return Err(anyhow!("Message is not of type objects"))}
	if !decode["type"].is_number() && !decode["server_name"].is_string() {
		return Err(anyhow!("Invalid format"));
	}

	Ok(Message{
		m_type: decode["type"].as_i32().unwrap(),
		add_server_name: false,
		from_server: cleanseString(decode["server_name"].as_str().unwrap()),
		content: decode["content"].as_object().unwrap_or(&jzon::object::Object::new()).to_owned()
	})
}

fn decodeChatMessage(message: &Message) -> Result<Chat, ()> {
	let content = &message.content;
	if !content["player_name"].is_string() && !content["chat_message"].is_string() {
		return Err(())
	}

	Ok(Chat{
		player_name: content["player_name"].as_str().unwrap().to_string(),
		chat_message: content["chat_message"].as_str().unwrap().replace("@", "")
	})
}

fn decodePlayerJoin(message: &Message, profile_cache: &mut HashMap<String, String>) -> Result<PlayerJoin, ()> {
	let content = &message.content;
	if !content["player_name"].is_string() {return Err(())}

	let player_name = content["player_name"].as_str().unwrap();
	let chars = player_name.as_bytes();
	let mut color: u32 = 0;
	for char in chars {
		let val = (*char as u32) * 10000;
		if color + val >= 16777215 {break}

		color += val;
	}

	Ok(PlayerJoin{
		player_name: player_name.to_string(),
		profile_pic_url: evalProfilePicture(player_name, profile_cache),
		profile_color: color,
	})
}

fn decodePlayerLeft(message: &Message) -> Result<PlayerLeft, ()> {
	let content = &message.content;
	if !content["player_name"].is_string() {return Err(())}

	Ok(PlayerLeft{
		player_name: content["player_name"].as_str().unwrap().to_string()
	})
}

// cleanses ^x stuff from strings
fn cleanseString(string: &str) -> String {
    let mut string = String::from(string);
    while let Some(pos) = string.find("^") {
        let mut new_string = String::from(string.get(..pos).unwrap());
        if let Some(v) = string.get(pos + 2..) {
            // if the found ^ is not the last byte in the string then add everything after that byte
            new_string.push_str(v);
        }
        string = new_string.to_owned();
    }

    string
}

// --------------------------------------------------------------------------------
// UDP Stuff
fn openUdpListener(port: u16, non_blocking: bool) -> Result<UdpSocket> {
	let socket = UdpSocket::bind(
		SocketAddr::from(([0, 0, 0, 0], port))
	)?;
	socket.set_nonblocking(non_blocking)?;
	Ok(socket)
}

fn udpTryReceive(socket: &UdpSocket) -> Result<String> {
	let mut read_buffer = [0; 64000];
	let (number_of_bytes, _) = socket.recv_from(&mut read_buffer)?;

	let content_buffer = &mut read_buffer[..number_of_bytes];
	let to_base64 = str::from_utf8(&content_buffer)?;
	let decode_b64 = general_purpose::STANDARD.decode(to_base64)?;
	let to_string = String::from_utf8(decode_b64)?;

	//println!("{}", &to_string);

	Ok(to_string)
}

fn get_reqwest_client() -> Result<reqwest::blocking::Client> {
	let client = reqwest::blocking::ClientBuilder::new()
		.danger_accept_invalid_certs(true) // temp
		.build()?;
	Ok(client)
}
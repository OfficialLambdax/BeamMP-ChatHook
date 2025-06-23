// Made by Neverless @ BeamMP. Issues? Feel free to ask.
#![allow(non_snake_case)]

mod ipapi;

use std::env;
use std::{net::{SocketAddr, UdpSocket}};
use std::collections::HashMap;

use jzon;
use base64::{Engine as _, engine::{general_purpose}};
use anyhow::{Result, anyhow};
use discord_webhook_rs as webhook;
use webhook::{Webhook, Embed, Field};
use once_cell::sync::OnceCell;

const VERSION: u8 = 1;
const PROTOCOL_VERSION: u8 = 3;

static AVATAR_URL: OnceCell<String> = OnceCell::new();
static WEBHOOK_URL: OnceCell<String> = OnceCell::new();
static UDP_PORT: OnceCell<u16> = OnceCell::new();

struct Messages {
	pub from_server: String,
	pub player_count: i32,
	pub player_max: i32,
	pub player_dif: i32,
	pub contents: Vec<Content>,
}

struct Content {
	pub m_type: i64,
	pub content: jzon::object::Object,
}

struct ScriptMessage {
	pub script_ref: String,
	pub chat_message: String,
}

struct Chat {
	pub player_name: String,
	pub chat_message: String,
}

struct PlayerJoining {
	pub player_name: String,
}

struct PlayerJoin {
	pub player_name: String,
	pub profile_pic_url: String,
	pub profile_color: u32,
	pub country_flag: String,
	pub is_vpn: bool,
}

struct PlayerLeft {
	pub player_name: String,
	pub early: bool,
}

fn main() -> Result<()> {
	let args: Vec<String> = env::args().collect();
	WEBHOOK_URL.set({
		if let Ok(key) = env::var("WEBHOOK_URL") {
			key
		} else if let Some(key) = args.get(1) {
			key.to_string()
		} else {
			panic!("Expected WEBHOOK_URL");
		}
	}).unwrap();

	UDP_PORT.set({
		if let Ok(key) = env::var("UDP_PORT") {
			key
		} else if let Some(key) = args.get(2) {
			key.to_string()
		} else {
			panic!("Expected UDP_PORT");
		}
	}.parse::<u16>().unwrap()).unwrap();

	AVATAR_URL.set({
		if let Ok(key) = env::var("AVATAR_URL") {
			key
		} else if let Some(key) = args.get(3) {
			key.to_string()
		} else {
			panic!("Expected AVATAR_URL")
		}
	}).unwrap();

	println!("Hello from ChatHook! o7\nChatHook Version {}\nProtocol Version {}\nListening on 0.0.0.0:{}\nSending to: {}\nAvatar URL: {}",
		VERSION, PROTOCOL_VERSION, UDP_PORT.get().unwrap(), WEBHOOK_URL.get().unwrap(), AVATAR_URL.get().unwrap()
	);

    let mut profile_cache: HashMap<String, String> = HashMap::new();
	let socket = openUdpListener(UDP_PORT.get().unwrap().clone(), false)?;
	let m_types: HashMap<i64, &str> = HashMap::from([
		(1, "onChatMessage"),
		(2, "onServerOnline"),
		(3, "onPlayerJoin"),
		(4, "onPlayerLeft"),
		(5, "onServerReload"),
		(6, "onScriptMessage"),
		(7, "onPlayerJoining"),
	]);

	let _ = defaultWebhookHeader("BeamMP ChatHook")
		.content(&format!("### 🌺 Hello from [*BeamMP ChatHook*](https://github.com/OfficialLambdax/BeamMP-ChatHook) v{} o/", VERSION))
		.send(); // we let it fail

	loop {
		match udpTryReceive(&socket) {
			Ok(receive) => {
				//println!("{}", &receive);
				match decodeReceiveBuf(&receive) {
					Ok(messages) => {
						let mut script_buf = String::new();
						for content in &messages.contents {
							let m_type = m_types.get(&content.m_type);
							println!("Handling Type {} message for {}", if m_type.is_some() {m_type.unwrap()} else {"Unknown"}, messages.from_server);
							handleMessage(&messages, &content, &mut profile_cache, &mut script_buf);
						}

						if script_buf.len() > 0 {
							script_buf.pop();
							if let Err(e) = defaultWebhookHeader(&messages.from_server).content(script_buf).send() {
								eprintln!("{:?}", e);
							}
						}
					},
					Err(e) => eprintln!("{}", e)
				}
			},
			Err(e) => eprintln!("{}", e)
		}
	}
}

// --------------------------------------------------------------------------------
// Handle and do stuff
fn handleMessage(message: &Messages, content: &Content, profile_cache: &mut HashMap<String, String>, script_buf: &mut String) {
	match content.m_type {
		1 => {
			if let Ok(chat) = decodeChatMessage(&content) {
				if let Err(e) = sendChatMessage(&message, chat) {
					eprintln!("{:?}", e);
				}
			} else {
				eprintln!("Invalid format for chat message from {}", &message.from_server);
			}
		},
		2 => {
			if let Err(e) = sendServerOnline(&message) {
				eprintln!("{:?}", e);
			}
		},
		3 => {
			if let Ok(player) = decodePlayerJoin(&content, profile_cache) {
				if let Err(e) = sendPlayerJoin(&message, player) {
					eprintln!("{:?}", e);
				}
			} else {
				eprintln!("Invalid format for player join from {}", &message.from_server);
			}
		}
		4 => {
			if let Ok(player) = decodePlayerLeft(&content) {
				if let Err(e) = sendPlayerLeft(&message, player) {
					eprintln!("{:?}", e);
				}
			} else {
				eprintln!("Invalid format for player left from {}", &message.from_server);
			}
		}
		5 => {
			if let Err(e) = sendServerReload(&message) {
				eprintln!("{:?}", e);
			}
		}
		6 => {
			if let Ok(chat) = decodeScriptMessage(&content) {
				sendScriptMessage(chat, script_buf)
			} else {
				eprintln!("Invalid format for script message from {}", &message.from_server);
			}
		}
		7 => {
			if let Ok(player) = decodePlayerJoining(&content) {
				if let Err(e) = sendPlayerJoining(&message, player) {
					eprintln!("{:?}", e);
				}
			} else {
				eprintln!("Invalid format for on player joining from {}", &message.from_server);
			}
		}
		_ => {
			eprintln!("Invalid Option from {}", &message.from_server)
		}
	}
}

fn isGuest(player_name: &str) -> bool {
	// guest5165468
	// 5 7
	if player_name.len() != 12 {return false}
	if &player_name[..5] != "guest" {return false}
	
	let split: Vec<char> = player_name[5..12].chars().collect();
	for char in split {
		if !char.is_ascii_digit() {return false}
	}
	true
}

// unicode compatible
fn cutServerName(server_name: &str) -> String {
	if server_name.chars().count() <= 80 {return server_name.to_string()}

	let mut new = String::new();
	let char_vec: Vec<char> = server_name.chars().collect();
	for i in 0..77 {
		new.push_str(&char_vec.get(i).unwrap().to_string());
	}
	new.push_str("...");
	new
}

fn defaultWebhookHeader(server_name: &str) -> Webhook {
	Webhook::new(WEBHOOK_URL.get().unwrap())
		.username(cutServerName(server_name))
		.avatar_url(AVATAR_URL.get().unwrap())
}

fn sendPlayerJoining(message: &Messages, player: PlayerJoining) -> Result<(), webhook::Error> {
	let mut content = String::new();
	content.push_str(&format!("> -# - 🕵️ ***{}** joining ({} Qued)*", player.player_name, &message.player_dif));
	defaultWebhookHeader(&message.from_server)
		.content(content)
		.send()?;

	Ok(())
}

fn sendPlayerJoin(message: &Messages, player: PlayerJoin) -> Result<(), webhook::Error> {
	let content;
	let mut vpn = "";
	if player.is_vpn {vpn = " (VPN)"}

	if isGuest(&player.player_name) {
		content = format!("→ {} {}{}\n\n-# {}/{} Players ♦ {} Qued", &player.player_name, player.country_flag, vpn, &message.player_count, &message.player_max, &message.player_dif);
	} else {
		content = format!("→ [{}](https://forum.beammp.com/u/{}) {}{}\n\n-# {}/{} Players ♦ {} Qued", &player.player_name, &player.player_name, player.country_flag, vpn, &message.player_count, &message.player_max, &message.player_dif);
	}
	defaultWebhookHeader(&message.from_server)
		.content("")
		.add_embed(
			Embed::new()
				.thumbnail(player.profile_pic_url)
				.color(player.profile_color)
				.add_field(
					Field::new()
						.name("🧡 New Player Joined!")
						.value(content)
				)
		).send()?;

	Ok(())
}

fn sendPlayerLeft(message: &Messages, player: PlayerLeft) -> Result<(), webhook::Error> {
	let mut content = String::new();
	if !player.early {
		content.push_str(&format!("> - 🚪 ***{}** left ({}/{})*", &player.player_name, &message.player_count, &message.player_max));
	} else {
		content.push_str(&format!("> -# - 🚪 ***{}** left during download ({}/{})*", &player.player_name, &message.player_count, &message.player_max));
	}
	defaultWebhookHeader(&message.from_server)
		.content(content)
		.send()?;
	
	Ok(())
}

fn sendChatMessage(message: &Messages, chat: Chat) -> Result<(), webhook::Error> {
	let mut content = String::new();
	content.push_str(&format!("> - 💬 **{}:** {}", &chat.player_name, &chat.chat_message));
	defaultWebhookHeader(&message.from_server)
		.content(content)
		.send()?;


    Ok(())
}

fn sendScriptMessage(chat: ScriptMessage, script_buf: &mut String)  {
	if chat.script_ref.len() == 0 {
		script_buf.push_str(&format!("> - ⚙️ **Server:** {}\n", cleanseString(&chat.chat_message)));
	} else {
		script_buf.push_str(&format!("> - ⚙️ **{}:** {}\n", chat.script_ref, cleanseString(&chat.chat_message)));
	}
}

fn sendServerOnline(message: &Messages) -> Result <(), webhook::Error> {
	let mut content = String::new();
	content.push_str(&format!("## ✅ Server has just (re)started!"));
	defaultWebhookHeader(&message.from_server)
		.content(content)
		.send()?;
	
	Ok(())
}

fn sendServerReload(message: &Messages) -> Result<(), webhook::Error> {
	let mut content = String::new();
	content.push_str(&format!("## ♻️ Server side script has reloaded"));
	defaultWebhookHeader(&message.from_server)
		.content(content)
		.send()?;

	Ok(())
}

// --------------------------------------------------------------------------------
// Profile pic cache and ip-api eval
fn evalProfilePicture(player_name: &str, profile_cache: &mut HashMap<String, String>) -> String {
	if isGuest(player_name) {return String::new()}
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
fn decodeReceiveBuf(message: &str) -> Result<Messages> {
	let decode = jzon::parse(message)?;
	if !decode.is_object() {return Err(anyhow!("Message is not of type object"))}
	if !decode["server_name"].is_string() || !decode["player_count"].is_number() || !decode["player_max"].is_number() || !decode["player_dif"].is_number() || !decode["version"].is_number() || !decode["contents"].is_array() {
		return Err(anyhow!("Invalid message pack format: {}", message));
	}

	let version = decode["version"].as_u8().unwrap();
	if version != PROTOCOL_VERSION {
		return Err(anyhow!("Invalid message pack version: {}", message));
	}

	let array = decode["contents"].as_array().unwrap();
	let mut contents: Vec<Content> = Vec::new();
	for message in array {
		if !message.is_object() || !message["type"].is_number() {return Err(anyhow!("Invalid message pack format: {}", message))}
		contents.push(Content
			{
				m_type: message["type"].as_i64().unwrap(),
				content: message["content"].as_object().unwrap_or(&jzon::object::Object::new()).to_owned()
			}
		);
	}

	Ok(Messages{
		from_server: cleanseString(decode["server_name"].as_str().unwrap()),
		player_count: decode["player_count"].as_i32().unwrap(),
		player_max: decode["player_max"].as_i32().unwrap(),
		player_dif: decode["player_dif"].as_i32().unwrap(),
		contents: contents,
	})
}

fn decodeChatMessage(content: &Content) -> Result<Chat, ()> {
	let content = &content.content;
	if !content["player_name"].is_string() && !content["chat_message"].is_string() {
		return Err(())
	}

	Ok(Chat{
		player_name: content["player_name"].as_str().unwrap().to_string(),
		chat_message: content["chat_message"].as_str().unwrap().replace("@", "").replace("http://", "").replace("https://", "").replace("discord.gg", "discord-gg")
	})
}

fn decodeScriptMessage(content: &Content) -> Result<ScriptMessage, ()> {
	let content = &content.content;
	if !content["chat_message"].is_string() || !content["script_ref"].is_string() {return Err(())}

	Ok(ScriptMessage {
		script_ref: content["script_ref"].as_str().unwrap().replace("@", ""),
		chat_message: content["chat_message"].as_str().unwrap().replace("@", "")
	})
}

fn decodePlayerJoining(content: &Content) -> Result<PlayerJoining, ()> {
	let content = &content.content;
	if !content["player_name"].is_string() {return Err(())}

	Ok(PlayerJoining{
		player_name: content["player_name"].as_str().unwrap().to_string()
	})
}

fn decodePlayerJoin(content: &Content, profile_cache: &mut HashMap<String, String>) -> Result<PlayerJoin, ()> {
	let content = &content.content;
	if !content["player_name"].is_string() || !content["ip"].is_string() {return Err(())}

	let player_name = content["player_name"].as_str().unwrap();
	let chars = player_name.as_bytes();
	let mut color: u32 = 0;
	for char in chars {
		let val = (*char as u32) * 10000;
		if color + val >= 16777215 {break}

		color += val;
	}

	let mut country_flag = String::new();
	let mut is_vpn = false;
	let ip_api = ipapi::evalIPApi(content["ip"].as_str().unwrap());
	if ip_api.is_err() {eprintln!("{}", ip_api.unwrap_err())} else {
		let ip_api = ip_api.unwrap();
		if let Some(flag) = country_emoji::flag(&ip_api.country_code) {
			country_flag.push_str(&flag);
		}
		is_vpn = ip_api.hosting || ip_api.proxy;
	}

	Ok(PlayerJoin{
		player_name: player_name.to_string(),
		profile_pic_url: evalProfilePicture(player_name, profile_cache),
		profile_color: color,
		country_flag: country_flag,
		is_vpn: is_vpn,
	})
}

fn decodePlayerLeft(content: &Content) -> Result<PlayerLeft, ()> {
	let content = &content.content;
	if !content["player_name"].is_string() || !content["early"].is_boolean() {return Err(())}

	Ok(PlayerLeft{
		player_name: content["player_name"].as_str().unwrap().to_string(),
		early: content["early"].as_bool().unwrap(),
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
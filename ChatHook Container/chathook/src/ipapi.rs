use serde::{Deserialize};
use anyhow::{Result, anyhow};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IpApiObject {
	pub status: String,
	pub country_code: String,
	pub proxy: bool,
	pub hosting: bool,
}

pub fn evalIPApi(ip: &str) -> Result<IpApiObject> {
	let client = get_reqwest_client();
	if client.is_err() {return Err(anyhow!("Cannot create reqwest client - {}", client.unwrap_err()))}

	let body = client.unwrap().get(format!("http://ip-api.com/json/{}?fields=status,countryCode,proxy,hosting", ip)).send();
	if body.is_err() {return Err(anyhow!("Get request to ip-api failed - {}", body.unwrap_err()))}

	let text = body.unwrap().text();
	if text.is_err() {return Err(anyhow!("ip-api response doesnt contain text - {}", text.unwrap_err()))}

	let parsed = serde_json::from_str::<IpApiObject>(&text.unwrap());
	if parsed.is_err() {return Err(anyhow!("Cannot parse ip-api response - {}", parsed.unwrap_err()))}

	let parsed = parsed.unwrap();
	if parsed.status != "success" {return Err(anyhow!("ip-api status failed"))}

	Ok(parsed)
}

fn get_reqwest_client() -> Result<reqwest::blocking::Client> {
	let client = reqwest::blocking::ClientBuilder::new()
		//.danger_accept_invalid_certs(true) // temp
		.build()?;
	Ok(client)
}
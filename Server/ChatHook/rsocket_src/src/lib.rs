use std::{net::{SocketAddr, UdpSocket}, str::FromStr};

use mlua::prelude::*;
use mlua::UserData;
use mlua::Error;


#[derive(Debug, thiserror::Error)]
enum GenericError {
    #[error("Cannot bind to given address {0}")]
    Bind(String),
    #[error("Cannot connect to given address {0}")]
    Connect(String),
    #[error("Cannot send data")]
    Send,
    #[error("Cannot receive data")]
    Recv,
    #[error("Cannot convert to UTF-8")]
    Utf8,
    #[error("Cannot enable non blocking mode")]
    NonBlocking,
}

struct UdpClient {
    socket: UdpSocket
}

// https://docs.rs/mlua/0.11.5/mlua/trait.UserData.html
impl UserData for UdpClient {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_once("close", |_, this, ()| {
            drop(this);
            Ok(())
        });

        methods.add_method("send", |_, this, data: String| {
            /*if this.socket.send(data.as_bytes()).is_err() {
                return Err(error(GenericError::Send))
            }*/
            if let Err(e) = this.socket.send(data.as_bytes()) {
                dbg!("{}", e.kind());
                match e.kind() {
                    std::io::ErrorKind::WouldBlock => return Ok(()),
                    _ => return Err(error(GenericError::Send)) // this crashes the beammp server in 3.9.1
                }
            }
            Ok(())
        });

        methods.add_method("recvString", |lua, this, ()| {
            let mut buf: [u8; 65507] = [0; 65507];
            let len = match this.socket.recv(&mut buf) {
                Ok(v) => v,
                Err(e) => {
                    match e.kind() {
                        std::io::ErrorKind::WouldBlock => return Ok(mlua::Value::Nil),
                        _ => return Err(error(GenericError::Recv))
                    }
                }
            };

            let str = match std::str::from_utf8(&buf[..len]) {
                Ok(v) => v,
                Err(_) => return Err(error(GenericError::Utf8))
            };
            
            Ok(toLuaString(lua, str))
        })
    }
}


struct UdpListener {
    socket: UdpSocket
}

impl UserData for UdpListener {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_once("close", |_, this, ()| {
            drop(this);
            Ok(())
        });
        
        methods.add_method("recvString", |lua, this, ()| {
            let mut buf: [u8; 65507] = [0; 65507];
            let (len, src_addr) = match this.socket.recv_from(&mut buf) {
                Ok(v) => v,
                Err(e) => {
                    match e.kind() {
                        std::io::ErrorKind::WouldBlock => return Ok((mlua::Value::Nil, mlua::Value::Nil)),
                        _ => return Err(error(GenericError::Recv))
                    }
                }
            };

            let str = match std::str::from_utf8(&buf[..len]) {
                Ok(v) => v,
                Err(_) => return Err(error(GenericError::Utf8))
            };

            Ok((toLuaString(lua, str), toLuaString(lua, &src_addr.to_string())))
        });

        methods.add_method("sendTo", |_, this, (data, ip, port): (String, String, i32)| {
            let address = format!("{}:{}", &ip, port);
            if this.socket.send_to(data.as_bytes(), address).is_err() {
                return Err(error(GenericError::Send))
            }
            Ok(())
        });
    }
}


fn toLuaString(lua: &Lua, str: &str) -> mlua::Value {
    mlua::Value::String(lua.create_string(str).unwrap())
}

fn error(err: GenericError) -> LuaError {
    Error::ExternalError(std::sync::Arc::new(err))
}

fn create_bind_socket() -> Result<UdpSocket, ()> {
    for i in 3400..3500 {
        let addr = SocketAddr::from_str(&format!("0.0.0.0:{}", i)).unwrap();
        if let Ok(socket) = UdpSocket::bind(addr) {
            return Ok(socket);
        }
    }
    Err(())
}


fn create_udp_client(_: &Lua, (ip, port): (String, i32)) -> LuaResult<UdpClient> {
    let address = format!("{}:{}", &ip, port);
    let socket = match create_bind_socket() {
        Ok(v) => v,
        Err(_) => return Err(error(GenericError::Bind(address)))
    };

    if socket.connect(&address).is_err() {
        return Err(error(GenericError::Connect(address)))
    }

    if socket.set_nonblocking(true).is_err() {
        return Err(error(GenericError::NonBlocking))
    }

    Ok(UdpClient {
        socket: socket
    })
}

fn create_udp_listener(_: &Lua, (ip, port): (String, i32)) -> LuaResult<UdpListener> {
    let address = format!("{}:{}", &ip, port);
    let socket = match UdpSocket::bind(&address) {
        Ok(v) => v,
        Err(_) => return Err(error(GenericError::Bind(address)))
    };

    if socket.set_nonblocking(true).is_err() {
        return Err(error(GenericError::NonBlocking))
    }

    Ok(UdpListener {
        socket: socket
    })
}

#[mlua::lua_module]
fn rsocket(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set("udpClient", lua.create_function(create_udp_client)?)?;
    exports.set("udpListener", lua.create_function(create_udp_listener)?)?;
    Ok(exports)
}
use std::io::Write;
use std::net::TcpStream;
use crate::api::enums::{PlayerDirection, RecvCommand, ServerPlayer, ServerStatus, to_player_direction, to_server_state, to_color, ServerGameStatus, ServerNotification, ServerPlayerNew, ServerPlayerLeft, ServerChangeName, ServerHit, ServerDamage, Scoreboard, Color, ServerScoreboard};
use crate::api::enums::RecvCommand::Scoreboard;

/// Module for server communication
///
/// A thread will take care of all send and receive commands
/// The communication between this thread and the main process will
///     be handled using channels
///
/// These are the following commands to be used on the channel:
///


/// Commands to be sent to the server
pub enum ServerCommand {
    FORWARD,
    BACKWARD,
    LEFT,
    RIGHT,
    GET,
    SHOOT,
    OBSERVATION,
    GAMESTATUS,
    USERSTATUS,
    POSITION,
    SCOREBOARD,
    GOODBYE,
    NAME,
    SAY,
    COLOR,
}

/// Struct to be used in the thread's channel
pub struct SendCommand {
    command: ServerCommand,
    attr: String
}


/// Create a TCP Connection with the server
fn tcp_connect(address: &str, port: Option<i32>) -> TcpStream {
    let full_address = format!(
        "{}:{}", address, port.unwrap_or(8888)
    );

    TcpStream::connect(full_address)
        .expect(format!(
            "Error creating tcp stream for {}", full_address).as_str()
        )
}

/// Send a raw command to the server
///
/// # Arguments:
/// * `stream` - TCP Connection with the server
/// * `msg` - raw command to be sent
fn send_msg(stream: &mut TcpStream, msg: String) {
    stream.write(msg.as_bytes())
        .expect("Error sending raw command to server");
}

/// Send a command to the server
///
/// # Arguments:
/// * `stream` - TCP Connection with the server
/// * `command` - command to be sent
pub fn send_command(stream: &mut TcpStream, command: SendCommand) {
    let mut msg = match command.command {
        ServerCommand::FORWARD => "w".to_string(),
        ServerCommand::BACKWARD => "s".to_string(),
        ServerCommand::LEFT => "a".as_bytes(),
        ServerCommand::RIGHT => "d".to_string(),
        ServerCommand::GET => "t".to_string(),
        ServerCommand::SHOOT => "e".to_string(),
        ServerCommand::OBSERVATION => "o".to_string(),
        ServerCommand::GAMESTATUS => "g".to_string(),
        ServerCommand::USERSTATUS => "q".to_string(),
        ServerCommand::SCOREBOARD => "u".to_string(),
        ServerCommand::GOODBYE => "quit".to_string(),
        ServerCommand::NAME => format!("name;{}", command.attr),
        ServerCommand::SAY => format!("say;{}", command.attr),
        ServerCommand::COLOR => format!("color;{}", command.attr),
        _ => {"".to_string()}
    };

    // colocando o \n e botando em utf-8
    msg = format!("{}\n", msg);
    send_msg(stream, msg);
}

fn parse_buffer(data: String) -> &[String] {

    let to_be_trimmed: &[char] = &['\0', '\r', '\n'];
    let to_be_ignored: &[char] = &['\x01', '\x03'];

    let mut v: Vec<String> = Vec::new();

    for cmd in data.split('\n') {
        let mut c = cmd.to_string();

        // parsing string
        c = c.trim_matches(to_be_trimmed).to_string();

        // checking if is valid
        if !c.contains(to_be_ignored) && c.len() > 0 {
            // valid command, adding to vector
            v.push(c);
        }
    }

    v.as_slice()
}

fn parse_command(cmd_str: String) {
    let to_be_trimmed: &[char] = &['\0', '\r'];

    let cmd: Vec<&str> = cmd_str.trim_matches(to_be_trimmed).split(';').collect();

    match cmd[0] {
        // OBSERVATION
        "o" => {},

        // STATUS
        "s" => {
            if cmd.len() == 7 {
                RecvCommand::Status(
                    ServerStatus {
                        x: cmd[1].parse::<i8>().unwrap_or(-1),
                        y: cmd[2].parse::<i8>().unwrap_or(-1),
                        dir: to_player_direction(cmd[3]),
                        state: to_server_state(cmd[4]),
                        score: cmd[5].parse::<i64>().unwrap_or(0),
                        energy: cmd[5].parse::<i8>().unwrap_or(0),
                    }
                )
            } else { RecvCommand::Invalid }
        }

        // PLAYER
        "player" => {
            if cmd.len() == 7 {
                RecvCommand::Player(
                    ServerPlayer {
                        node: cmd[1].parse::<i64>().unwrap_or(0),
                        name: cmd[2].to_string(),
                        x: cmd[3].parse::<i8>().unwrap_or(-1),
                        y: cmd[4].parse::<i8>().unwrap_or(-1),
                        dir: to_player_direction(cmd[5]),
                        state: to_server_state(cmd[6]),
                        color: to_color(cmd[7])
                    }
                )
            } else { RecvCommand::Invalid }
        },
        "g" => {
            if cmd.len() == 3 {
                RecvCommand::GameStatus(
                    ServerGameStatus {
                        status: to_server_state(cmd[1]),
                        time: cmd[2].parse::<i64>().unwrap_or(-1)
                    }
                )
            } else { RecvCommand::Invalid }
        },
        "u" => {
            let mut v: Vec<Scoreboard> = Vec::new();
            for (i, s) in cmd.iter().enumerate() {
                if i == 0 { continue }

                let ss: Vec<&str> = s.split('#').collect();
                if ss.len() == 4 || ss.len() == 5 {
                    let name = ss[0].to_string();
                    let connected: bool = ss[1] == "connected";
                    let score = ss[2].parse::<i64>().unwrap_or(-1);
                    let energy = ss[3].parse::<i8>().unwrap_or(0);
                    let mut color = Color { r: 0, g: 0, b: 0, a: 0 };
                    if ss.len() == 5 {
                        color = to_color(ss[4]);
                    }
                    let sb = Scoreboard { name, connected, score, energy, color };

                    v.push(sb);
                }
            }
            RecvCommand::Scoreboard(
                ServerScoreboard {
                    scoreboards: v
                }
            )

        },
        "notification" => {
            RecvCommand::Notification(
                ServerNotification {
                    notification: cmd[1].to_string()
                }
            )
        },
        "hello" => {
            RecvCommand::PlayerNew(
                ServerPlayerNew {
                    player: cmd[1].to_string()
                }
            )
        },
        "goodbye" => {
            RecvCommand::PlayerLeft(
                ServerPlayerLeft {
                    player: cmd[1].to_string()
                }
            )
        },
        "changename" => {
            if cmd.len() == 3 {
                RecvCommand::ChangeName(
                    ServerChangeName {
                        old_name: cmd[1].to_string(),
                        new_name: cmd[2].to_string()
                    }
                )
            } else { RecvCommand::Invalid }
        },
        "h" => {
            RecvCommand::Hit(
                ServerHit {
                    target: cmd[1].to_string()
                }
            )
        },
        "d" => {
            RecvCommand::Damage(
                ServerDamage {
                    shooter: cmd[1].to_string()
                }
            )
        },
        _ => {}
    }
}
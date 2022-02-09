/// Module for server communication
///
/// A thread will take care of all send and receive commands
/// The communication between this thread and the main process will
///     be handled using channels
///

use std::io::{Error, ErrorKind, Write};
use std::net::TcpStream;
use std::sync::mpsc;
use std::time::Duration;

use crate::api::comms::ServerCommand::COLOR;
use crate::api::enums::{
    PlayerDirection, ServerState,
};
use crate::api::structs::{
    LastObservation, Color,
    Scoreboard, ServerObservation,
    ServerStatus, ServerPlayer,
    ServerGameStatus, ServerScoreboard,
    ServerNotification, ServerPlayerNew,
    ServerPlayerLeft, ServerChangeName,
    ServerHit, ServerDamage
};

type ClientSendChannel = mpsc::Sender<ClientCommunication>;
type ClientRecvChannel = mpsc::Receiver<ServerCommunication>;
type ServerSendChannel = mpsc::Sender<ServerCommunication>;
type ServerRecvChannel = mpsc::Receiver<ClientCommunication>;


/// Commands to be sent to the server
pub enum ServerCommand {
    FORWARD, BACKWARD,
    LEFT, RIGHT, GET,
    SHOOT, OBSERVATION,
    GAMESTATUS, USERSTATUS,
    POSITION, SCOREBOARD,
    GOODBYE, NAME,
    SAY, COLOR,
}

pub enum ClientCommunication {
    GetLastRecvCommand,             // asking for the last command
    SendThisCommand(SendCommand)    // asking for a command to be send
}

pub enum ServerCommunication {
    Ok,                             // server responded with OK
    Error,                          // server responded with Error
    RecvCommand(RecvCommand)        // server responded with a command
}


/// This struct is used to send a command to the server,
/// via the comms' channel
pub struct SendCommand {
    command: ServerCommand,
    attr: Option<String>
}

/// This struct is used to receive something from the server,
/// and send via channel to the bot's main thread
pub enum RecvCommand {
    Observations(ServerObservation),
    Status(ServerStatus),
    Player(ServerPlayer),
    GameStatus(ServerGameStatus),
    Scoreboard(ServerScoreboard),
    Notification(ServerNotification),
    PlayerNew(ServerPlayerNew),
    PlayerLeft(ServerPlayerLeft),
    ChangeName(ServerChangeName),
    Hit(ServerHit),
    Damage(ServerDamage),
    Invalid
}

/// Main struct of all communications.
///
/// It will handle all comms via its methods.
///
/// # Example:
/// ```
/// // todo
/// ```
pub struct GameServer {
    connected: bool,
    active: bool,
    recv_channel: ServerRecvChannel,
    send_channel: ServerSendChannel,
    server: Option<TcpStream>,
    last_recv_command: Option<RecvCommand>,
    drone_color: Option<Color>,
    drone_name: Option<String>
}

impl GameServer {
    /// Creates a new GameServer, without connecting to the server yet.
    pub fn new(receiver: ServerRecvChannel, sender: ServerSendChannel) -> GameServer {
        GameServer {
            connected: false,
            active: false,
            recv_channel: receiver,
            send_channel: sender,
            server: None,
            last_recv_command: None,
            drone_color: None,
            drone_name: None
        }
    }

    fn check_internal_state(&self) -> Result<(), &str> {
        if self.connected {
            if self.server.is_none() { Err("no tcp connection stored") }
            if self.drone_name.is_none() { Err("no name stored") }
            if self.drone_color.is_none() { Err("no color stored") }
            Ok(())
        } else { Ok(()) }
    }

    /// Keep socket alive - verify current status
    fn keep_alive(&mut self) {
        // checking internal variables
        if self.active != self.connected {
            self.socket_status_change()
        }
        // updating internal variables
        if !self.active | !self.connected {
            self.active = false;
            self.connected = false;
        }
    }

    fn socket_status_change(&mut self) {
        if self.connected {
            println!("GameServer: connected");
            if let Err(e) = self.check_internal_state() {
                println!("GameServer [ERROR]: internal state out of sync: {}", e);
                return
            }

            let server = &mut self.server.unwrap();
            let name = self.drone_name.unwrap().clone();
            let color = self.drone_color.unwrap().clone();

            // sending my name
            send_command(server, SendCommand {
                command: ServerCommand::NAME, attr: Some(name)
            });

            // sending my color
            send_command(server, SendCommand {
                command: ServerCommand::COLOR, attr: Some(color.to_string())
            });

            // requesting game status
            send_command(server, SendCommand {
                command: ServerCommand::GAMESTATUS, attr: None
            });

            // requesting user status
            send_command(server, SendCommand {
                command: ServerCommand::USERSTATUS, attr: None
            });

            // requesting observation
            send_command(server, SendCommand {
                command: ServerCommand::OBSERVATION, attr: None
            });
        } else {
            println!("GameServer: disconnected")
        }
    }

    pub fn start(&mut self, address: &str, port: Option<i32>) {
        if !self.connected {
            self.server = Some(tcp_connect(address, port));
            self.connected = true;
            self.active = true;
            self.keep_alive();

            // starting loop
            self.thread_loop();
        }
    }

    fn thread_loop(&mut self) {}

    pub fn do_this_command(
        send_channel: &mut ClientSendChannel,
        recv_channel: &mut ClientRecvChannel,
        command: SendCommand,
        timeout: Option<Duration>
    ) -> Result<(), Err> {

        if let mpsc::SendError(_) = send_channel.send(ClientCommunication::SendThisCommand(command)) {
            println!("failed to connect with GameServer thread. Is it down?");
            return Err("GameServer connecting ended");
        }

        match recv_channel.recv_timeout(timeout.unwrap_or(Duration::from_millis(10))) {
            Ok(ServerCommunication::Ok) => Ok(()),
            Ok(ServerCommunication::Error) => Err("GameServer returned error on do_this_command"),
            Ok(_) => Err("GameServer returned invalid on do_this_command"),
            Err(mpsc::RecvTimeoutError::Timeout) => Err("GameServer channel timeout on do_this_command"),
            Err(mpsc::RecvTimeoutError::Disconnected) => Err("GameServer channel has disconnected on do_this_command")
        }
    }

    pub fn access_last_recv_command(
        send_channel: &mut ClientSendChannel,
        recv_channel: &mut ClientRecvChannel,
        timeout: Option<Duration>
    ) -> Option<RecvCommand> {

        if let mpsc::SendError(_) = send_channel.send(ClientCommunication::GetLastRecvCommand) {
            println!("failed to connect with GameServer thread. Is it down?");
            return None;
        };

        match recv_channel.recv_timeout(timeout.unwrap_or(Duration::from_millis(10))) {
            Ok(ServerCommunication::RecvCommand(r)) => Some(r),

            Ok(ServerCommunication::Error) => {
                println!("GameServer returned error on access_last_recv_command");
                None
            },

            Ok(ServerCommunication::Ok) => None,

            Err(mpsc::RecvTimeoutError::Timeout) => {
                println!("GameServer channel timeout on access_last_recv_command");
                None
            },

            Err(mpsc::RecvTimeoutError::Disconnected) => {
                println!("GameServer channel has disconnected on access_last_recv_command");
                None
            }
        }
    }
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

/// Send a raw command to the server.
/// 
/// Returns a `Result` containing how many bytes were sent, or an error
///
/// # Arguments:
/// * `stream` - TCP Connection with the server
/// * `msg` - raw command to be sent
///
fn send_msg(stream: &mut TcpStream, msg: String) -> Result<usize, &str>{
    match stream.write(msg.as_bytes()) {
        Ok(u) => Ok(u),
        Err(_) => Err("Error sending raw message")
    }
}

/// Send a command to the server.
///
/// Returns `Ok()` on success, or an `Error`
///
/// # Arguments:
/// * `stream` - TCP Connection with the server
/// * `command` - command to be sent
fn send_command(stream: &mut TcpStream, command: SendCommand) -> Result<(), &str> {

    // checking if the command has its attr correctly
    let attr = match command.command {
        ServerCommand::NAME | ServerCommand::SAY | ServerCommand::COLOR => {
            match command.attr {
                Some(s) => s,
                None => return Err("invalid command without attr")
            }
        },
        _ => {}
    };

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
        ServerCommand::NAME => format!("name;{}", attr),
        ServerCommand::SAY => format!("say;{}", attr),
        ServerCommand::COLOR => format!("color;{}", attr),
        _ => {"".to_string()}
    };

    // colocando o \n e botando em utf-8
    msg = format!("{}\n", msg);
    match send_msg(stream, msg) {
        Ok(_) => Ok(()),
        Err(e) => Err(e)
    }
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

fn parse_observations(observations: String) -> LastObservation {
    let mut last_observation: LastObservation = LastObservation {
        is_enemy_front: false,
        is_blocked: false,
        is_steps: false,
        is_breeze: false,
        is_flash: false,
        is_treasure: false,
        is_powerup: false,
        is_damage: false,
        is_hit: false,
        distance_enemy_front: -1
    };
    if !observations.trim() == "" {
        for o in observations.trim().split(",") {
            let temp: Vec<&str> = o.split("#").collect();
            if temp.len() > 1 {
                last_observation.is_enemy_front = true;
                last_observation.distance_enemy_front = temp[1].parse::<i32>().unwrap_or(0)
            }
            match o {
                "blocked" => last_observation.is_blocked = true,
                "steps" => last_observation.is_steps = true,
                "breeze" => last_observation.is_breeze = true,
                "flash" => last_observation.is_flash = true,
                "blueLight" => last_observation.is_treasure = true,
                "redLight" => last_observation.is_powerup = true,
                _ => {}
            }
        }
    }
    last_observation
}

fn parse_command(cmd_str: String) -> RecvCommand {
    let to_be_trimmed: &[char] = &['\0', '\r'];

    let cmd: Vec<&str> = cmd_str.trim_matches(to_be_trimmed).split(';').collect();

    match cmd[0] {
        // OBSERVATION
        "o" => {
            if cmd.len() > 1 {
                RecvCommand::Observations(
                    ServerObservation {
                        last_observation: parse_observations(cmd[1].to_string())
                    }
                )
            } else { RecvCommand::Invalid }
        },

        // STATUS
        "s" => {
            if cmd.len() == 7 {
                RecvCommand::Status(
                    ServerStatus {
                        x: cmd[1].parse::<i8>().unwrap_or(-1),
                        y: cmd[2].parse::<i8>().unwrap_or(-1),
                        dir: PlayerDirection::from_str(cmd[3]),
                        state: ServerState::from_str(cmd[4]),
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
                        dir: PlayerDirection::from_str(cmd[5]),
                        state: ServerState::from_str(cmd[6]),
                        color: Color::from_str(cmd[7])
                    }
                )
            } else { RecvCommand::Invalid }
        },
        "g" => {
            if cmd.len() == 3 {
                RecvCommand::GameStatus(
                    ServerGameStatus {
                        status: ServerState::from_str(cmd[1]),
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
                        color = Color::from_str(ss[4]);
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
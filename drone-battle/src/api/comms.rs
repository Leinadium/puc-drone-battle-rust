/// Module for server communication
///
/// A thread will take care of all send and receive commands
/// The communication between this thread and the main process will
///     be handled using channels
///

use std::io::{ErrorKind, Read, Write, Error};
use std::net::{TcpStream, Shutdown};
use std::sync::mpsc;
use std::time::Duration;
use std::thread;
use std::fmt::Debug;

use crate::api::enums::{Action, PlayerDirection, ServerState};
use crate::api::structs::{
    LastObservation, Color,
    Scoreboard, ServerObservation,
    ServerStatus, ServerPlayer,
    ServerGameStatus, ServerScoreboard,
    ServerNotification, ServerPlayerNew,
    ServerPlayerLeft, ServerChangeName,
    ServerHit, ServerDamage
};
use crate::api::config::Config;

type SenderChannel = mpsc::Sender<RecvCommand>;
type ReceiverChannel = mpsc::Receiver<SendCommand>;


/// Commands to be sent to the server
#[derive(Debug, PartialEq, Clone)]
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
    // POSITION,    // not used
    SCOREBOARD,
    GOODBYE,
    NAME,
    SAY,
    COLOR,
    NOTHING,
}

/// This struct is used to send a command to the server,
/// via the comms' channel
#[derive(Debug, Clone)]
pub struct SendCommand {
    pub command: ServerCommand,
    pub attr: Option<String>
}

impl SendCommand {
    pub fn from_action(action: &Action) -> SendCommand {
        match action {
            Action::FRONT => SendCommand { command: ServerCommand::FORWARD, attr: None },
            Action::BACK => SendCommand { command: ServerCommand::BACKWARD, attr: None },
            Action::LEFT => SendCommand { command: ServerCommand::LEFT, attr: None },
            Action::RIGHT => SendCommand { command: ServerCommand::RIGHT, attr: None },
            Action::GET => SendCommand { command: ServerCommand::GET, attr: None },
            Action::SHOOT => SendCommand { command: ServerCommand::SHOOT, attr: None },
            Action::NOTHING => SendCommand { command: ServerCommand::NOTHING, attr: None },
        }
    }
}

/// This struct is used to receive something from the server,
/// and send via channel to the bot's main thread
#[derive(Debug)]
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
    Invalid(String)
}

impl RecvCommand {
    pub fn to_string(&self) -> String {
        match self {
            RecvCommand::Observations(so) => {
                format!("ServerObservation({})", so.last_observation)
            },
            rc=> format!("{:?}", rc)
        }
    }
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
    recv_channel: ReceiverChannel,
    send_channel: SenderChannel,
    server: Option<TcpStream>,
    drone_color: Option<Color>,
    drone_name: Option<String>
}

impl GameServer {
    /// Creates a new GameServer, without connecting to the server yet.
    pub fn new(receiver: ReceiverChannel, sender: SenderChannel, config: &Config) -> GameServer {
        GameServer {
            connected: false,
            active: false,
            recv_channel: receiver,
            send_channel: sender,
            server: None,
            drone_color: Some(config.default_color.clone()),
            drone_name: Some(config.name.clone())
        }
    }

    pub fn close(&mut self) -> Result<(), Error>{
        if let Some(stream) = self.server.as_mut() {
            if let Err(e) = stream.shutdown(Shutdown::Both) {
                println!("GameServer [ERROR]: error closing connecting");
                Err(e)
            } else {
                println!("GameServer: connecting closed successfully");
                Ok(())
            }
        } else {
            println!("GameServer [ERROR]: attempt to close a non-existing connection");
            Err(Error::from(ErrorKind::NotConnected))
        }
    }

    fn check_internal_state(&self) -> Result<(), &str> {
        if self.connected {
            if self.server.is_none() { return Err("no tcp connection stored") }
            if self.drone_name.is_none() { return Err("no name stored") }
            if self.drone_color.is_none() { return Err("no color stored") }
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

            let server = self.server.as_mut().unwrap();
            let name = self.drone_name.as_ref().unwrap().clone();
            let color = self.drone_color.as_ref().unwrap().clone();

            // sending my name
            send_command(server, SendCommand {
                command: ServerCommand::NAME, attr: Some(name)
            }).ok();

            // sending my color
            send_command(server, SendCommand {
                command: ServerCommand::COLOR, attr: Some(color.to_string())
            }).ok();

            // requesting game status
            send_command(server, SendCommand {
                command: ServerCommand::GAMESTATUS, attr: None
            }).ok();

            // requesting user status
            send_command(server, SendCommand {
                command: ServerCommand::USERSTATUS, attr: None
            }).ok();

            // requesting observation
            send_command(server, SendCommand {
                command: ServerCommand::OBSERVATION, attr: None
            }).ok();

        } else {
            println!("GameServer: disconnected")
        }
    }

    pub fn start(&mut self, address: &str, port: Option<i32>) {
        if !self.connected {
            self.server = Some(tcp_connect(address, port));

            // reading ip
            let ip = self.server.as_mut()
                .unwrap()
                .peer_addr()
                .expect("could not connect to the server");
            println!("sucessfully connected with {}", ip);

            // setting up
            self.server.as_mut().unwrap()
                .set_nonblocking(true)
                .expect("set_nonblocking call failed");
            println!("connection with server is all set up");

            self.connected = true;
            self.active = true;
            self.keep_alive();

            // starting loop
            println!("starting loop");
            self.thread_loop();
        }
    }

    fn thread_loop(&mut self) {
        // send initial commands
        self.socket_status_change();

        loop {
            self.keep_alive();      // update self

            let mut recv_buffer = [0; 4096];
            if self.connected {
                // checking with server for new commands
                let has_message = match self.server.as_mut().unwrap().read(&mut recv_buffer) {
                    Ok(size) => size > 0,
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => false,
                    Err(e) => panic!("GameServer [ERROR]: thread_loop: error -> {}", e)
                };

                if has_message {
                    // converting commands
                    let recv_string: String = String::from_utf8_lossy(&recv_buffer).to_string();
                    let mut commands: Vec<RecvCommand> = Vec::new();
                    for c in parse_buffer(recv_string) {
                        commands.push(parse_command(&c))
                    }

                    // sending commands to client
                    for cmd in commands {
                        if let Err(e) = self.send_channel.send(cmd) {
                            println!("GameServer [ERROR]: error sending command back to client: {}", e);
                        }
                    }
                }

                // checking with client for something
                match self.recv_channel.recv_timeout(Duration::from_millis(5)) {
                    Ok(sc) => {
                        if let Err(e) = send_command(self.server.as_mut().unwrap(), sc.clone()) {
                            println!("GameServer [ERROR]: error sending command to server: {}", e);
                        }

                        // checking for a goodbye
                        if ServerCommand::GOODBYE == sc.command {
                            self.close().expect("unable to close the connection");
                            return      // exiting thread loop
                        }

                    },
                    Err(mpsc::RecvTimeoutError::Timeout) => {},
                    Err(mpsc::RecvTimeoutError::Disconnected) => panic!("GameServer: client has disconnected")
                };
            }   // endif
            thread::sleep(Duration::from_millis(1));
        }
    }

    pub fn do_this_command(send_channel: &mut mpsc::Sender<SendCommand>, command: SendCommand, ) -> Result<(), &str> {
        match send_channel.send(command) {
            Err(_) => {
                println!("failed to connect with GameServer thread. Is it down?");
                Err("GameServer connection ended")
            },
            Ok(_) => Ok(())
        }
    }
}


/// Create a TCP Connection with the server
fn tcp_connect(address: &str, port: Option<i32>) -> TcpStream {
    let full_address = format!(
        "{}:{}", address, port.unwrap_or(8888)
    );

    TcpStream::connect(full_address.clone())
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
        Err(_) => {
            // println!("error writing to server: {}", e);
            Err("Error sending raw message")
        }
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
            match &command.attr {
                Some(s) => s.clone(),
                None => return Err("invalid command without attr")
            }
        },
        _ => "".to_string()
    };

    let mut msg = match command.command {
        ServerCommand::FORWARD => "w".to_string(),
        ServerCommand::BACKWARD => "s".to_string(),
        ServerCommand::LEFT => "a".to_string(),
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
        _ => return Ok(()),      // skipping
    };

    // println!("GameServer: sending to server -> {:?} (raw message: {})", &command, &msg);

    // colocando o \n e botando em utf-8
    msg = format!("{}\n", msg);
    match send_msg(stream, msg) {
        Ok(_) => Ok(()),
        Err(e) => Err(e)
    }
}

fn parse_buffer(data: String) -> Vec<String> {

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
    v
}

fn parse_observations(observations: String) -> LastObservation {
    let mut last_observation: LastObservation = LastObservation::new();
    if observations.trim() != "" {
        for o in observations.trim().split(",") {
            let temp: Vec<&str> = o.split("#").collect();
            if temp.len() > 1 {
                last_observation.is_enemy_front = true;
                last_observation.distance_enemy_front = temp[1].parse::<i16>().unwrap_or(0)
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

fn parse_command(cmd_str: &String) -> RecvCommand {
    let to_be_trimmed: &[char] = &['\0', '\r'];

    let cmd: Vec<&str> = cmd_str.trim_matches(to_be_trimmed)
        .split(';')
        .collect();

    match cmd[0] {
        // OBSERVATION
        "o" => {
            if cmd.len() > 1 {
                RecvCommand::Observations(
                    ServerObservation {
                        last_observation: parse_observations(cmd[1].to_string())
                    }
                )
            } else { RecvCommand::Invalid(cmd_str.to_string()) }
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
                        score: cmd[5].parse::<i64>().unwrap_or(-123),
                        energy: cmd[6].parse::<i32>().unwrap_or(-123),
                    }
                )
            } else { RecvCommand::Invalid(cmd_str.to_string()) }
        }

        // PLAYER
        "player" => {
            if cmd.len() == 8 {
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
            } else { RecvCommand::Invalid(cmd_str.to_string()) }
        },
        "g" => {
            if cmd.len() == 3 {
                RecvCommand::GameStatus(
                    ServerGameStatus {
                        status: ServerState::from_str(cmd[1]),
                        time: cmd[2].parse::<i64>().unwrap_or(-1)
                    }
                )
            } else { RecvCommand::Invalid(cmd_str.to_string()) }
        },
        "u" => {
            let mut v: Vec<Scoreboard> = Vec::new();
            for (i, s) in cmd.iter().enumerate() {
                if i == 0 { continue }

                let ss: Vec<&str> = s.split('#').collect();
                if ss.len() == 4 || ss.len() == 5 {
                    let name = ss[0].to_string();
                    let connected: bool = ss[1] == "connected";
                    let score = ss[2].parse::<i64>().unwrap_or(-123);
                    let energy = ss[3].parse::<i32>().unwrap_or(-123);
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
            } else { RecvCommand::Invalid(cmd_str.to_string()) }
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
        _ => RecvCommand::Invalid(cmd_str.to_string())
    }
}
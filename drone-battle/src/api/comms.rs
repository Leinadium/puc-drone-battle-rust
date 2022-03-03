use crate::api::{
    enums::{Action, PlayerDirection, ServerState},
    structs::{
        LastObservation, Color,
        Scoreboard, ServerObservation,
        ServerStatus, ServerPlayer,
        ServerGameStatus, ServerScoreboard,
        ServerNotification, ServerPlayerNew,
        ServerPlayerLeft, ServerChangeName,
        ServerHit, ServerDamage
    },
    config::Config
};
use std::{
    io::{Read, Write},
    net::{TcpStream, Shutdown},
    fmt::Debug,
    thread,
};
use crossbeam_channel::{Sender, Receiver};

/// Channel to send a `RecvCommand` back to the bot/client
type SenderChannel = Sender<RecvCommand>;
/// Channel to receive a `SendCommand` from the bot/client
type ReceiverChannel = Receiver<SendCommand>;

/// GameServer struct
///
/// This is struct functions like a middle man between the bot and the server.
/// It receives commands from the bot, parses it and sends to the server.
/// And it receives data from the server, parses it and sends back to the bot.
///
/// on `.run()`, it will split itself in two: *bot_to_server* and *server_to_bot*:
///
/// # bot_to_server:
/// waits for some action from the bot channel. Parses it into a string, and sends it to the server via TCP
///
/// # server_to_bot:
/// waits for some data from the TCP connection with the server. Parses it into a `RecvCommand`, and
/// sends it to the bot via channel.
pub struct GameServer {
    recv_channel: ReceiverChannel,
    send_channel: SenderChannel,
    server: Option<TcpStream>,
    drone_color: Color,
    drone_name: String,
}

impl GameServer {
    /// Generates a new GameServer using arguments
    pub fn new(receiver: ReceiverChannel, sender: SenderChannel, config: &Config) -> GameServer {
        GameServer {
            recv_channel: receiver,
            send_channel: sender,
            server: None,
            drone_color: config.default_color.clone(),
            drone_name: config.name.clone()
        }
    }

    /// Sends all initial commands to the server:
    ///
    /// * name
    /// * color
    /// * gameStatus
    /// * userStatus
    /// * observation
    fn send_config(&mut self) -> Option<()> {
        let server = self.server.as_mut()?;
        // sending my name
        send_command(server, SendCommand {
            command: ServerCommand::NAME, attr: Some(self.drone_name.clone())
        });
        // sending my color
        send_command(server, SendCommand {
            command: ServerCommand::COLOR, attr: Some(self.drone_color.to_string())
        });
        // requesting game status
        send_command(server, SendCommand {
            command: ServerCommand::GAMESTATUS, attr: None
        });
        // requesting user status
        send_command(server, SendCommand {
            command: ServerCommand::USERSTATUS, attr: None
        })?;
        // requesting observation
        send_command(server, SendCommand {
            command: ServerCommand::OBSERVATION, attr: None
        })?;
        Some(())
    }

    /// Closes the TCP connection
    fn close(server: TcpStream) {
        if let Err(e) = server.shutdown(Shutdown::Both) {
            println!("[GAMESERVER ERROR]: error while closing connection: {:?}", e);
        } else {
            println!("[GAMESERVER] shutdown successful");
        }
    }

    /// Runs the game server loop
    ///
    /// Creates the TCP connection with the server. Then configures it.
    /// Later, creates the `server_to_bot` thread, and then runs the `bot_to_server` loop.
    ///
    /// Because it starts an endless loop, it can only be stopped when receiving an *GOODBYE* command
    /// from the bot.
    ///
    /// # Params:
    /// * address: server address
    /// * port: server port
    pub fn run(mut self, address: &str, port: Option<i32>) {
        // creating server
        println!("[GAMESERVER] creating server connection");
        self.server = Some(tcp_connect(address, port));

        // printing ip
        let ip = self.server.as_mut()
            .unwrap()
            .peer_addr()
            .unwrap();
        println!("[GAMESERVER] connected with {}", ip);

        // setting up
        self.server.as_mut()
            .unwrap()
            .set_nodelay(true)
            .expect("set_delay failed");

        println!("[GAMESERVER] sending initial configs");
        self.send_config().expect("could not send configs");

        let server_clone = self.server.as_mut()
            .unwrap()
            .try_clone()
            .expect("server clone failed");

        println!("[GAMESERVER] starting 'server_to_bot' thread");
        let handle = thread::Builder::new()
            .name("GAMESERVER server_to_bot".into())
            .spawn(move || {
                GameServer::loop_server_to_bot(self.send_channel, server_clone);
            })
            .unwrap();

        println!("[GAMESERVER] starting 'bot_to_server' loop");
        GameServer::loop_bot_to_server(self.recv_channel, self.server.unwrap());


        println!("[GAMESERVER] waiting for 'server_to_bot' thread to join");
        handle.join().expect("could not join threads");

        return
    }

    /// Bot to server communication loop.
    ///
    /// It waits for some command from the bot channel. Then parses into a string, and sends to the server.
    /// If the command was a *GOODBYE*, it ends the loop, and closes the TCP connection
    fn loop_bot_to_server(receiver: ReceiverChannel, mut server: TcpStream) {
        let mut sc;
        let mut command;
        loop {
            // getting from bot
            sc = match receiver.recv() {
                Err(_) => {
                    println!("[GAMESERVER 'bot_to_server' ERROR]: client has disconnected. Closing");
                    break;
                },
                Ok(sc) => sc
            };
            command = sc.command.clone();

            // sending to server
            if let None = send_command(&mut server, sc) {
                println!("[GAMESERVER 'bot_to_server' ERROR]: error sending command to server")
            }

            // checking if it needs to shut down
            if command == ServerCommand::GOODBYE { break; }
        }
        GameServer::close(server);
    }

    /// Server to Bot communication loop.
    ///
    /// It waits for some server data. Parses into RecvCommands and sends via channel to the bot/client.
    ///
    /// If something goes wrong with the server TCP connection, it ends the loop. This is also valid
    /// whenever the TCP connection is closed via `GameServer::close(server)`
    fn loop_server_to_bot(sender: SenderChannel, mut server: TcpStream) {
        loop {
            let mut recv_buffer = [0; 4096];
            match server.read(&mut recv_buffer) {
                Ok(s) => {
                    if s == 0 {
                        println!("[GAMESERVER 'server_to_bot'] connection closed. exiting");
                        break;
                    }
                },
                Err(e) => {
                    println!("[GAMESERVER 'server_to_bot' ERROR] error reading from server: {:?}", e);
                    break;
                }
            }

            let recv_string: String = String::from_utf8_lossy(&recv_buffer).to_string();

            let mut commands: Vec<RecvCommand> = Vec::new();
            for c in parse_buffer(recv_string) {
                commands.push(parse_command(&c))
            }
            // sending commands to client
            for cmd in commands {
                if let Err(e) = sender.send(cmd) {
                    println!("[GAMESERVER 'server_to_bot' ERROR]: error sending command back to client: {}", e);
                }
            }
        }
    }

    /// Sends the `SendCommand` to the server.
    pub fn do_this_command(send_channel: &mut Sender<SendCommand>, command: SendCommand, ) -> Result<(), &str> {
        match send_channel.send(command) {
            Err(_) => {
                println!("failed to connect with GameServer thread. Is it down?");
                Err("GameServer connection ended")
            },
            Ok(_) => Ok(())
        }
    }
}

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
    /// Transforms an action into a `SendCommand`
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
/// and send via channel to the main thread of the bot
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
    #[allow(dead_code)]
    pub fn to_string(&self) -> String {
        match self {
            RecvCommand::Observations(so) => {
                format!("ServerObservation({})", so.last_observation)
            },
            rc=> format!("{:?}", rc)
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
fn send_command(stream: &mut TcpStream, command: SendCommand) -> Option<()> {

    // checking if the command has its attr correctly
    let attr = match command.command {
        ServerCommand::NAME | ServerCommand::SAY | ServerCommand::COLOR => {
            match &command.attr {
                Some(s) => s.clone(),
                None => {return None}
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
        _ => return Some(()),      // skipping
    };

    // println!("GameServer: sending to server -> {:?} (raw message: {})", &command, &msg);

    // inserting \n and converting to utf-8
    msg = format!("{}\n", msg);
    match send_msg(stream, msg) {
        Ok(_) => Some(()),
        Err(_) => None
    }
}

/// Parses a string containing commands into a vector of strings,
/// which each one of the string is a single command, yet to be parsed
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

/// Converts a string containing observations into a `LastObservation` object
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

/// Parses a string containing some command into a `RecvCommand`
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
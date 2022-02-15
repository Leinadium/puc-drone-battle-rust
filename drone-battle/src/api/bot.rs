use crate::api::comms::{GameServer, SendCommand, RecvCommand, ServerCommand};
use crate::api::structs::{ServerPlayer, ServerScoreboard, LastObservation};
use crate::api::enums::{PlayerDirection, ServerState, Action};
use crate::api::config::Config;
use crate::api::map::update;
use crate::api::ai::AI;

use std::sync::mpsc::{channel, Sender, Receiver, RecvTimeoutError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime, SystemTimeError};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::fmt::{Display, Formatter};
use std::borrow::Borrow;

type ExitHandlerStruct = Arc<AtomicBool>;


/// Struct containing everything the bot needs to play
///
/// To create a bot, use `Bot::new()` to create a bot,
/// and `.start()` to start a infinite loop.
///
/// # Example
///
/// ```
/// let mut bot = Bot::new(api::config::Config::default());
/// bot.run();
/// ```
pub struct Bot {
    /// AI
    pub ai: AI,
    /// Current configuration of the bot
    pub config: Config,
    /// Server structure, containing the sender and receiver channels
    /// to be able to communicate with the GameServer thread
    server: ServerChannels,
    /// A hashmap mapping all players to its node
    player_list: HashMap<i64, ServerPlayer>,
    /// The complete scoreboard of a game
    score_list: ServerScoreboard,
    /// Current time, provided by the server
    game_time: i64,
    /// Last observation provided by the server
    last_observation: LastObservation,
    /// Current tick of the bot. A tick is considered a complete loop
    pub current_tick: i32,
    /// Current x position of the bot
    x: i16,
    /// Current y position of the bot
    y: i16,
    /// Current direction of the bot
    dir: PlayerDirection,
    /// Current state of the bot
    state: ServerState,
    /// Current score of the bot
    score: i64,
    /// Current energy of the bot
    energy: i32,
    /// Total time thinking for the next action
    thinking_time: Duration,
    /// Name of the last bot to damage the bot
    last_damage: String,
    /// Time of the last damage to the bot
    last_time_damage: SystemTime,
    /// A struct to handle calls to close the bot
    exit_handler: ExitHandlerStruct,
    /// Handle of game server
    thread_handle: JoinHandle<()>

}

impl Bot {
    /// Create a new bot, using the configuration provided.
    ///
    /// Also starts a `api::comms::GameServer` thread to communicate
    /// with the server.
    /// Use `.exit()`, that sends a command to close the thread, to end it.
    pub fn new(config: Config) -> Bot {
        // creating server listener
        let (tx_client, rx_server) = channel::<SendCommand>();
        let (tx_server, rx_client) = channel::<RecvCommand>();
        let mut game_server = GameServer::new(rx_server, tx_server, &config);
        // starting server listener
        let url_copy = config.url.clone();
        let join_handle = thread::spawn(move || {
            game_server.start(url_copy.as_str(), None)
        });

        // creating bot
        Bot {
            ai: AI::new(&config),
            current_tick: 0,
            config,
            server: ServerChannels {tx: tx_client, rx: rx_client},
            player_list: HashMap::new(),
            score_list: ServerScoreboard {scoreboards: Vec::new()},
            game_time: 0,
            last_observation: LastObservation::new(),
            // default values below
            x: 0,
            y: 0,
            dir: PlayerDirection::NORTH,
            state: ServerState::READY,
            score: 0,
            energy: 0,
            thinking_time: Duration::from_secs(0),
            last_damage: "".to_string(),
            last_time_damage: SystemTime::now(),
            exit_handler: create_exit_handler(),
            thread_handle: join_handle
        }
    }

    /// Closes the GameServer thread.
    /// Also consumes itself.
    pub fn exit(mut self) {
        GameServer::do_this_command(
            &mut self.server.tx,
            SendCommand { command: ServerCommand::GOODBYE, attr: None}
        ).ok();
        self.thread_handle.join().expect("Bot [ERROR]: could not join game server thread");
    }

    /// Puts the bot to sleep for some duration. It skips negative durations
    ///
    /// Also updates the field.
    fn sleep(&mut self, duration: Duration) {
        update::do_tick(&mut self.ai.field, duration);
        thread::sleep(duration.clone());
        println!();
        println!("[BOT] sleep: {} ms", duration.as_millis());

    }

    /// Prints the current scoreboard
    fn print_score(&self) {
        println!("====================");
        println!("==== SCOREBOARD ====");
        println!("game_time: {}", self.game_time);
        println!("game_state: {}", self.state);
        println!("====================");
        for sb in &self.score_list.scoreboards {
            println!(
                "{} ({}): score={}, energy={}",
                sb.name, if sb.connected {"online" } else { "offline" },
                sb.score, sb.energy
            );
        }
        println!("====================");
        println!("====================");
    }

    /// Method to be used whenever the bot suffers some damage.
    ///
    /// Checks if the last damage happened too fast by the same bot.
    /// If so, sends a message to everyone:
    ///
    ///`anticheat alert: Bot1 hit me again after XXX ms (allowed: XXX ms)`
    fn anti_cheat(&mut self, current_damage: String) -> Result<(), SystemTimeError>{
        let time = self.last_time_damage.elapsed()?;

        if self.last_damage == current_damage && time < self.config.min_timer {
            self.say_all_chat(format!(
                "anticheat alert: {} hit me again after {} ms (allowed: {} ms)",
                self.last_damage, time.as_millis(), self.config.min_timer.as_millis()
            ));
        }
        Ok(())
    }

    /// Starts the infinite loop. Can only be stopped by a CTRL-C
    pub fn run(&mut self) {
        let mut timer = 0;
        let mut exec_time;
        let mut playing: bool = false;
        let mut action: Action = Action::NOTHING;

        self.ai.field.restart();

        loop {
            // game is running
            if check_exit_handler(self.exit_handler.borrow()) { return }    // early exit

            if self.state == ServerState::GAME && self.energy > 0 {
                // sleeping
                if action == Action::SHOOT {    // only sleep the min time possible
                    self.sleep(
                        self.config.min_timer.clone()
                            .checked_sub(self.thinking_time)
                            .unwrap_or(Duration::from_millis(0)))
                } else {                        // sleep normally
                    self.sleep(self.config.normal_timer.clone()
                        .checked_sub(self.thinking_time)
                        .unwrap_or(Duration::from_millis(0)))
                }
                println!("[BOT] thinking_time: {} ms", self.thinking_time.as_millis());

                // if the game is starting
                if !playing {
                    playing = true;
                    self.restart();
                }

                // update internal state
                self.update_with_server(true);

                // updating variables
                self.current_tick += 1;
                exec_time = SystemTime::now();

                // do the action
                let data = BotData::from_bot(&self);
                println!("[BOT] bot_data: {}", &data);
                action = self.ai.think(data);
                GameServer::do_this_command(
                    &mut self.server.tx,
                    SendCommand::from_action(&action)
                ).ok();

                // after doing the action
                self.after_action();
                self.thinking_time = exec_time.elapsed().unwrap_or(Duration::from_secs(0));
            }
            // game is NOT running
            else {
                self.sleep(self.config.slow_timer);                 // sleep a bit
                self.update_with_server(false);
                if playing { self.say_all_chat("gg".to_string()) }      // say gg once
                playing = false;
                self.ai.field.restart();

                // after some time, ask for scoreboard
                if timer == 5 {
                    GameServer::do_this_command(&mut self.server.tx,
                        SendCommand { command: ServerCommand::SCOREBOARD, attr: None}
                    ).ok();
                    self.update_with_server(false);
                    self.print_score();
                    timer = 0;
                }
                timer += 1;

                // asking for game status
                GameServer::do_this_command(&mut self.server.tx,
                    SendCommand { command: ServerCommand::GAMESTATUS, attr: None}
                ).ok();
            }
        }
    }

    /// Helper method, containing the actions to be done after sending an action
    fn after_action(&mut self) {
        self.last_observation.reset();
        // asking for some observations
        GameServer::do_this_command(
            &mut self.server.tx,
            SendCommand { command: ServerCommand::OBSERVATION, attr: None}
        ).ok();
        // asking for my status
        GameServer::do_this_command(
            &mut self.server.tx,
            SendCommand { command: ServerCommand::USERSTATUS, attr: None}
        ).ok();
        // asking for game status
        GameServer::do_this_command(
            &mut self.server.tx,
            SendCommand { command: ServerCommand::GAMESTATUS, attr: None}
        ).ok();
    }

    /// Resets some variables, and send some initial commands to the server
    fn restart(&mut self) {
        self.current_tick = 0;
        // asking for game status
        GameServer::do_this_command(
            &mut self.server.tx,
            SendCommand { command: ServerCommand::GAMESTATUS, attr: None}
        ).ok();
        // asking for my status
        GameServer::do_this_command(
            &mut self.server.tx,
            SendCommand { command: ServerCommand::USERSTATUS, attr: None}
        ).ok();
        // asking for some observations
        GameServer::do_this_command(
            &mut self.server.tx,
            SendCommand { command: ServerCommand::OBSERVATION, attr: None}
        ).ok();
        self.sleep(self.config.normal_timer.clone());
    }

    /// Say something in the chat
    fn say_all_chat(&mut self, msg: String) {
        GameServer::do_this_command(
            &mut self.server.tx,    // server communication
            SendCommand {
                command: ServerCommand::SAY,    // mode: say something
                attr: Some(msg)                 // msg to be sent
            }
        ).ok();
    }

    /// Get all responses from the server, and updates all internal variables
    fn update_with_server(&mut self, mut needs_checklist: bool) {
        let mut checklist: ServerChecklist = ServerChecklist::new();
        loop {
            let attempt = self.server.rx.recv_timeout(Duration::from_millis(5));
            let rc: RecvCommand = match attempt {
                Err(RecvTimeoutError::Timeout) => if !needs_checklist { break } else { continue },
                Err(RecvTimeoutError::Disconnected) => { println!("[BOT ERROR]: recv error"); break; },
                Ok(x) => x
            };

            // println!("[BOT INFO] received from GameServer -> {}", rc.to_string());
            match rc {
                RecvCommand::Observations(so) => {
                    checklist.observation = true;
                    // checking first if it can overwrite the hit or damage observations
                    let ishit = self.last_observation.is_hit;
                    let isdamage = self.last_observation.is_damage;
                    let hasreaddamage = self.last_observation.has_read_damage;
                    let hasreadhit = self.last_observation.has_read_hit;

                    self.last_observation = so.last_observation.clone();

                    if !hasreaddamage {
                        self.last_observation.is_damage = isdamage;
                    };
                    if !hasreadhit {
                        self.last_observation.is_hit = ishit;
                    }
                },
                RecvCommand::Status(ss) => {
                    checklist.user = true;
                    self.x = ss.x as i16;
                    self.y = ss.y as i16;
                    self.dir = ss.dir.clone();
                    self.state = ss.state.clone();
                    self.score = ss.score;
                    self.energy = ss.energy;
                }
                RecvCommand::Player(sp) => {
                    self.player_list.insert(sp.node, sp.clone());
                }
                RecvCommand::GameStatus(sgs) => {
                    checklist.game = true;
                    self.state = sgs.status.clone();
                    self.game_time = sgs.time;
                    if sgs.status != ServerState::GAME { needs_checklist = false; }  // skip waiting, game is over
                }
                RecvCommand::Scoreboard(ss) => {
                    self.score_list = ss.clone();
                }
                RecvCommand::Notification(sn) => {
                    println!("Bot [INFO]: {}", sn.notification);
                }
                RecvCommand::PlayerNew(spn) => {
                    println!("Bot [INFO]: [{}] has joined the game", spn.player);
                }
                RecvCommand::PlayerLeft(spl) => {
                    println!("Bot [INFO]: [{}] has left the game", spl.player);
                }
                RecvCommand::ChangeName(scn) => {
                    println!("Bot [INFO]: [{}] changed its name to [{}]", scn.old_name, scn.new_name);
                }
                RecvCommand::Hit(sh) => {
                    println!("BOT [HIT]: I hit [{}]", sh.target);
                    self.last_observation.is_hit = true;
                    self.last_observation.has_read_hit = false;
                }
                RecvCommand::Damage(sd) => {
                    self.anti_cheat(sd.shooter.clone()).ok();
                    println!("BOT [DAMAGE]: [{}] damaged me", sd.shooter);
                    self.last_observation.is_damage = true;
                    self.last_observation.has_read_damage = false;
                }
                RecvCommand::Invalid(_) => {}
            }

            // checking checklist
            if needs_checklist && checklist.check() { break; }

        }

    }
}

/// Helper struct, to gather both ways of the channel
pub struct ServerChannels {
    pub tx: Sender<SendCommand>,
    pub rx: Receiver<RecvCommand>
}

/// Generates an AtomicBool, to be checked if the bot needs to be closed
fn create_exit_handler() -> ExitHandlerStruct {
    // copied from https://docs.rs/ctrlc/latest/ctrlc/#example
    let running = Arc::new(AtomicBool::new(false));
    let running_copy = running.clone();

    // starting thread
    ctrlc::set_handler(move || {
        // if CTRL-C, set to TRUE
        running_copy.store(true, Ordering::Relaxed);
    }).expect("Error setting exit Handler");

    running     // return of the copies of the AtomicBool
}

/// Checks if the bot needs to be closed
fn check_exit_handler(eh: &ExitHandlerStruct) -> bool {
    eh.load(Ordering::Relaxed)
}

#[derive(Debug)]
pub struct BotData {
    x: i16,
    y: i16,
    dir: PlayerDirection,
    energy: i32,
    last_observation: LastObservation,
}

impl BotData {
    pub fn from_bot(bot: &Bot) -> BotData {
        BotData {
            x: bot.x.clone(),
            y: bot.y.clone(),
            dir: bot.dir.clone(),
            energy: bot.energy.clone(),
            last_observation: bot.last_observation.clone()
        }
    }

    pub fn get_x(&self) -> i16 { self.x.clone() }

    pub fn get_y(&self) -> i16 { self.y.clone() }

    pub fn get_energy(&self) -> i32 { self.energy.clone() }

    pub fn get_dir(&self) -> PlayerDirection { self.dir.clone() }

    pub fn get_last_observation(&self) -> LastObservation { self.last_observation.clone() }

}

impl Display for BotData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(x: {}, y: {}, dir: {:?}, en: {}, lo: {}",
               self.x, self.y, self.dir, self.energy, self.last_observation.to_string())
    }
}

struct ServerChecklist {
    pub observation: bool,
    pub user: bool,
    pub game: bool,
}

impl ServerChecklist {
    pub fn new() -> ServerChecklist {
        ServerChecklist { observation: false, user: false, game: false }
    }

    pub fn check(&self) -> bool {
        self.observation && self.game && self.user
    }
}
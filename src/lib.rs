use std::{
    io::{self, Read, Write},
    net::TcpStream,
    sync::mpsc,
    thread, time,
};

macro_rules! thrust_and_return {
    // ($Fn:ident($($arg:expr),*)) => {
    ($f:expr) => {
        // match $Fn($($arg),*) {
        match $f {
            Reconnect::Yes => return Reconnect::Yes,
            Reconnect::Quit => return Reconnect::Quit,
            _ => {}
        }
    };
}

macro_rules! doodah {
    ($stream:expr, $msg:expr) => {
        match write($stream, $msg) {
            Reconnect::Yes | Reconnect::Quit => return Reconnect::Yes,
            _ => {}
        }
    };
}

#[derive(Debug, Default)]
pub struct Message {
    pub username: String,
    pub message: String,
    pub user_id: String,
    pub channel: String,
    pub room_id: String,
    pub is_broadcaster: bool,
    pub is_mod: bool,
    pub is_vip: bool,
    pub is_subscriber: bool,
    pub color: String,
    pub timestamp: String,
    pub unique_message_id: String,
    pub is_highlighted: bool,
}

#[derive(Debug)]
pub struct TwitchAnon<'anon> {
    username: &'anon str,
    password: &'anon str,
    channel: String,
    queue: Option<mpsc::Sender<Message>>,
    pub messages: mpsc::Receiver<Message>,
    send_tx: mpsc::Sender<String>,
    send_rx: Option<mpsc::Receiver<String>>,
}

#[derive(Debug)]
struct Life {
    first_try: bool,
    max_attempts: usize,
    attempts: usize,
    pause: std::time::Duration,
}

impl Life {
    fn new() -> Self {
        Life {
            first_try: true,
            max_attempts: 8,
            attempts: 0,
            pause: std::time::Duration::from_secs(1),
        }
    }
}

impl<'anon> TwitchAnon<'anon> {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let (send_tx, send_rx) = mpsc::channel();

        TwitchAnon {
            username: "justinfan1979111",
            password: "BareCoolCowSawsMoomahOnTwitch",
            channel: "barecoolcowsaysmoomah,toggleBit".to_string(),
            queue: Some(tx),
            messages: rx,
            send_tx,
            send_rx: Some(send_rx),
        }
    }

    pub fn add_channel(mut self, channel: &'anon str) -> Self {
        self.channel = match self.channel.len() {
            0 => channel.to_string().to_lowercase(),
            _ => format!("{},{}", self.channel, channel).to_lowercase(),
        };
        self
    }

    pub fn run(mut self) -> Self {
        let mut anon_life = Life::new();

        let queue = self.queue.take().unwrap();

        let password = self.password.to_string();
        let username = self.username.to_string();
        let channels = self
            .channel
            .split(',')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        let _handle = thread::spawn(move || {
            loop {
                // yes? what should this do?
                // fixme: this should
                dbg!("Doing a reconnect.", &anon_life);

                if anon_life.attempts > anon_life.max_attempts {
                    return;
                }

                if !anon_life.first_try {
                    anon_life.attempts += 1;
                    std::thread::sleep(anon_life.pause);
                    anon_life.pause *= 2;
                }

                anon_life.first_try = false;

                if let Ok(mut stream) = TcpStream::connect("irc.chat.twitch.tv:6667") {
                    stream
                        .set_nonblocking(true)
                        .expect("Failed to make socket non-blocking!");

                    // TODO: FIXME: use doodah! here to reduce duplication?
                    if write(&mut stream, &format!("PASS {}", password)) == Reconnect::Yes {
                        continue;
                    }
                    if write(&mut stream, &format!("NICK {}", username)) == Reconnect::Yes {
                        continue;
                    }
                    if write(&mut stream, "CAP REQ :twitch.tv/tags") == Reconnect::Yes {
                        continue;
                    }
                    // if write(&mut stream, "CAP REQ :twitch.tv/commands\r\nCAP REQ :twitch.tv/membership\r\nCAP REQ :twitch.tv/tags\r\nCAP REQ :twitch.tv/tags twitch.tv/commands\r\n") { return; }
                    for channel in &channels {
                        if write(&mut stream, &format!("JOIN #{}", channel)) == Reconnect::Yes {
                            continue;
                        }
                    }

                    anon_life = Life::new();

                    match circle_check(&mut stream, &queue ) {
                        Reconnect::No | Reconnect::Quit => break,
                        Reconnect::Yes => {}
                    }
                }
            }
        });

        self
    }
}

fn circle_check(
    stream: &mut TcpStream,
    queue: &mpsc::Sender<Message>,
) -> Reconnect {
    loop {
        let heart = &mut Heart::new();

        thrust_and_return!(reader(stream, queue, heart));
        thrust_and_return!(heartbeat(stream, heart));

        std::thread::yield_now();
    }
}

struct Heart {
    last: time::Instant,
    tried: bool,
}

impl Heart {
    fn new() -> Self {
        Heart {
            last: time::Instant::now(),
            tried: false,
        }
    }

    fn reset(&mut self) {
        self.last = time::Instant::now();
        self.tried = false;
    }

    fn set_tried(&mut self, tried: bool) {
        self.tried = tried;
    }
}

#[derive(PartialEq)]
enum Reconnect {
    Yes,
    No,
    Quit,
}

fn heartbeat(mut stream: &mut TcpStream, heart: &mut Heart) -> Reconnect {
    // if we haven't sent or received in a normal time period we ping twitch
    // to make sure our socket is still alive and the connection is there

    // fixme:: put this timeout in const and/or config
    // todo:: add some sway to the amount of time we wait
    if heart.last.elapsed().as_secs() > 5 * 60 {
        doodah!(&mut stream, "PING :tmi.twitch.tv");
        heart.set_tried(true);
    } else if heart.tried && heart.last.elapsed().as_secs() > (5 * 60) + 10 {
        return Reconnect::Yes;
    }

    Reconnect::No
}

fn reader(
    mut stream: &mut TcpStream,
    queue: &mpsc::Sender<Message>,
    heart: &mut Heart,
) -> Reconnect {
    let mut buffer = String::new(); // fixme:: buffer isn't saved via loop anymore, do this different, read till end

    let mut input = [0; 4096];

    match stream.read(&mut input) {
        Ok(0) => {
            return Reconnect::Yes;
        }
        Err(ref error) if error.kind() == io::ErrorKind::WouldBlock => {}
        Ok(_bytes_read) => {
            heart.reset();

            buffer.push_str(std::str::from_utf8(&input).unwrap().trim_end_matches("\0"));

            'lines: while buffer.contains("\n") {
                dbg!(&buffer);

                if let Some(idx) = buffer.find("\n") {
                    let mut cmd_message = buffer
                        .drain(..idx + 1)
                        .collect::<String>()
                        .trim_end()
                        .to_string();

                    if cmd_message.starts_with("PING") {
                        doodah!(&mut stream, &cmd_message.replace("PING", "PONG"));
                        println!("{:?}", std::time::Instant::now());

                        continue 'lines;
                    } else if cmd_message.starts_with("PONG") {
                        heart.reset();
                        continue 'lines;
                    }

                    // grabs the tags
                    let mut tags: Vec<(String, String)> = Vec::new();
                    if cmd_message.starts_with('@') {
                        let idx = cmd_message.find(" :").unwrap();
                        let mut tmp = cmd_message.drain(..idx + 1).collect::<String>();
                        tmp.drain(..1);
                        tags = tmp
                            .split(';')
                            .filter_map(|keyval| {
                                let idx = keyval.find('=').unwrap();
                                Some((keyval[..idx].to_string(), keyval[idx + 1..].to_string()))
                            })
                            .collect();
                    }

                    let mut user_message = Message::default();

                    if tags.len() > 0 {
                        parse_tags(&tags, &mut user_message);
                    }

                    // grabs the irc part
                    if cmd_message.starts_with(':') {
                        cmd_message.drain(..1);

                        // grabs the message part
                        if let Some(idx) = cmd_message.find(" :") {
                            let message = cmd_message.split_off(idx + 2);
                            cmd_message.pop();

                            let params: Vec<&str> = cmd_message
                                .trim_end()
                                .split_whitespace()
                                .map(|s| s)
                                .collect();

                            if params[1] == "PRIVMSG" {
                                if user_message.username == "" {
                                    let idx = params[0].find('!').unwrap(); // this SHOULD always be here, if it's not, I don't know how you broke it
                                    let username = params[0].get(..idx).unwrap().to_string();
                                    user_message.username = username;
                                }

                                user_message.channel = params[2].to_string();
                                if user_message.channel.starts_with('#') {
                                    user_message.channel.remove(0);
                                }
                                user_message.message = message;

                                // this is the only place we currently send anything over the receiver, we own a single sender,
                                // so if the receiver is dead, we just return and let the user handle the dead receiver on their end. - _Bare 1Sep20
                                if let Err(_) = queue.send(user_message) {
                                    eprintln!("Receiver is dead!");
                                    return Reconnect::Quit;
                                }
                            }
                        }
                    }
                } else {
                    break 'lines;
                }
            }
        }
        Err(_) => {
            return Reconnect::Yes;
        }
    }

    Reconnect::No
}

fn write(stream: &mut TcpStream, message: &str) -> Reconnect {
    if let Err(_) = writeln!(stream, "{}", message) {
        dbg!("Write failed!");
        return Reconnect::Yes;
    }

    Reconnect::No
}

fn parse_tags(tags: &Vec<(String, String)>, message: &mut Message) {
    for (key, val) in tags {
        match key.as_ref() {
            // "subscriber" => message.is_subscriber = val == "1", // deprecated - not using
            // "mod" => message.is_mod = val == "1", // overlaps with badges - not using
            // "badge-info" => {} // subscriber months - not using for now
            "badges" => {
                if val.len() == 0 {
                    continue;
                }

                let badges = val.split(',').collect::<Vec<&str>>();
                for badge in &badges {
                    let idx = badge.find('/').unwrap();
                    match &badge[..idx] {
                        "broadcaster" => message.is_broadcaster = true,
                        "subscriber" | "founder" => message.is_subscriber = true,
                        "moderator" => message.is_mod = true,
                        "vip" => message.is_vip = true,
                        _ => {}
                    }
                }
            }
            // "client-nonce" => {} // apparently this is a thing - not using
            // "bits" => {} // not using for now
            "color" => message.color = val.to_string(),
            "display-name" => message.username = val.to_string(),
            // "emotes" => {} // not using for now
            // "flags" => {}  // not actually used? - not using
            "id" => message.unique_message_id = val.to_string(), // unique message id, maybe it's useful, maybe not, including
            // "message" => {} // not actually used? - not using
            "msg-id" => message.is_highlighted = val == "highlighted-message", // message is a highlighted message
            "room-id" => message.room_id = val.to_string(),                    // to room id
            "user-id" => message.user_id = val.to_string(),                    // from user id
            "tmi-sent-ts" => message.timestamp = val.to_string(),
            // "turbo" => {} // deprecated - not using
            // "user-type" => {} // deprecated - not using
            _ => {}
        }
    }
}

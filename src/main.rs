use twitch_anon::TwitchAnon;

fn main() {
    let anon = TwitchAnon::new()
        .add_channel("BareCoolCowSaysMooMah")
        .run();

    loop {
        if let Ok(t_msg) = anon.messages.recv() {
            if t_msg.message.starts_with('!') {
                dbg!(&t_msg.message);
                let command = t_msg.message.split(' ').collect::<Vec<&str>>();

                match command[0] {
                    "!discord" => match t_msg.channel.as_str() {
                        //     "barecoolcowsaysmoomah" => anon.send("#barecoolcowsaysmoomah", "https://discord.gg/h3UkuQU"),
                        //     "togglebit" => anon.send("#togglebit", "https://discord.gg/fZ4kFnS"),
                        _ => {}
                    },
                    _ => {}
                }
            }
        } else {
            dbg!("Receiver has died.");
            break; // Anything else means the receiver is dead.
        }
    }

    dbg!("Exiting...");
}

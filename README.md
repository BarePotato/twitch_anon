# twitch_anon

A simple twitch lib that allows you to read chat and receive PRIVMSG with tags.  
There is no send as we build upon using an anonymous Twitch user interface.  
The lib will maintain it's connection until you sever it, attempting to reconnect with exponential backoff.  
Shutdown can be triggered by just dropping the receiver or otherwise letting it go out of scope.

in your .env you should have a user(JustinFan####), a password(any string), and the channel(s) you wish to join.  
To join multiple channels separate them with commas as shown below.

```
TWITCHANON_NICK=justinfan1979111
TWITCHANON_PASS=BareDoodahOnTwitch
TWITCHANON_CHANNEL=BareDoodah,ToggleBit
```
If you are not getting messages, double check the channel in your .env and read the next line as these are the likely issues.  
if you get an env var error and this is set, make sure you haven't introduced whitespace or are overriding the .env somewhere. If you cloned the git, and are pointing to it from a different directory, the .env in the lib directory may override the one in your app directory. So you may want to remove it from the lib if presesnt.

New messages are sent on a receiver you can block on with recv or non-block with try_recv. Take a look at the rust docs for mpsc recv and try_recv for more explanation there.

The following data is available in the provided message strutct:

```rust
username: String,
message: String,
user_id: String,
channel: String,
room_id: String,
is_broadcaster: bool,
is_mod: bool,
is_vip: bool,
is_subscriber: bool,
color: String,
timestamp: String,
unique_message_id: String,
is_highlighted: bool, // if the message was sent with channel points event
```

We are not exposing bits, badge-info, or emotes related tags for now, but that may change.  
Any other fields are either deprecated or just not useful.

## Example:

```rust
use twitch_anon;

fn main() {
    let t_anon = twitch_anon::run();

    loop {
        if let Ok(t_msg) = t_anon.messages.recv() {
            if t_msg.message.starts_with('!') {
                eprintln!("We found a \'!\', lets do someting neato.\r\n{:#?}", t_msg);
            }
        } else {
            // an error here implies the receiver has died.
            // this means you would need to restart twitch_anon from its run
            // to get a new receiver and make sure all is well.
            // in your loop you might have a restart procedure to act upon
            eprintln!("Receiver has died.");
            break;
        }
    }

    eprintln!("Exiting...");
}
```

<hr>

## Old description - to be removed at some point

Simple lib to allow you to listen to a channel on twitch and do something with its PRIVMSG only.  
Useful for things needing to read chat for other reasons and not needing full bot functionality or sending messages.

You don't need to supply a username or oauth at all.  
You can use the suppplied ones, or if you want to avoid overlap with other potential users, change the numbers in the username to whatever you like.
The password can be whatever string you like, it doesn't matter.  
The only thing that matters is setting the channel. If you aren't getting messages from the lib then the channel may be wrong.

Messages are sent in a simple data struct to the receiver providing the username of the send and the message sent to chat.  
I may expand this later to include some other things like, a formatted username and a color(if chosen). PR if you want.

main.rs is included as a standalone example.  
Basically, though, you just check the mpsc receiver for a Message struct and then do something with it. try_recv for non-blocking.

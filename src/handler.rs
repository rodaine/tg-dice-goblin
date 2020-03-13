use telegram_bot::*;

use crate::parser::parse;
use crate::roll::RollResult;

const START_MSG: &str = "Let *Dice Goblin* roll for you!

Dice Goblin will roll any-sided dice and perform simple arithmetic to reach a total value, appropriate for many tabletop and RPG games. See /help for details on the commands and syntax available.";

const HELP_MSG: &str = "*COMMANDS*

/start
_See introductory information about this bot_

/help
_See this help output_

/roll `[expression]`
_Roll the dice and calculate a total (see expression syntax below)_

/r `[expression]`
_Alias for /roll_

*ROLL EXPRESSION SYNTAX*

Dice rolls are described in the standard `NdS` format, where `N` is the number of rolls and `S` is the number of sides. Each roll is summed together to calculate the overall value.

*Examples:*
`3d10` - Roll a ten-sided die three times
`d6` - Roll a single six-sided die (N defaults to 1 if omitted)
`D2` - flip a coin (The `d` is case-insensitive)

Rolls support basic arithmetic using the operators (+, -, \\*, /) as well as parenthesis. Division always truncates (or rounds towards zero), and division by zero always equals zero.

*Examples:*
`3d10 + 2` - Roll three ten-sided dice and add two to the result
`(d6 - 1) * 2` - Roll a six-sided die, subtract one from the roll, and then double the result
`3 / 2` - Equals 1 (1.5 rounded towards zero)
`1 / 0` - Division by zero always equals zero";

const UNKNOWN_MSG: &str = "Unknown command. Use /help to see available commands";

const ROLL_ERR_MSG: &str = "I don't understand this roll. Rolls should look something like `5d20 + 3`. See /help for the full syntax.";

pub async fn handle(api: &Api, update: Update) -> Result<(), Error> {
    let msg = match update.kind {
        telegram_bot::update::UpdateKind::Message(m) => m,
        _ => return Ok(()),
    };

    let (txt, entities) = match &msg.kind {
        telegram_bot::MessageKind::Text { data, entities } => (data, entities),
        _ => return Ok(()),
    };

    let cmd = match Command::resolve(&txt, &entities) {
        Some(c) => c,
        None => return Ok(()),
    };

    match cmd {
        Command::Start => {
            let mut req = SendMessage::new(msg.chat, START_MSG);
            req.parse_mode(ParseMode::Markdown);
            api.send(req).await
        }

        Command::Help => {
            let mut req = SendMessage::new(msg.chat, HELP_MSG);
            req.parse_mode(ParseMode::Markdown);
            api.send(req).await
        }
        Command::Roll(offset) => {
            match parse(&txt[offset..]) {
                Some(d) => {
                    let res: RollResult = (&d).into();
                    api.send(msg.text_reply(format!("{}", res))).await
                }
                None => {
                    let mut req = msg.text_reply(ROLL_ERR_MSG);
                    req.parse_mode(ParseMode::Markdown);
                    api.send(req).await
                }
            }
        }
        _ => api.send(msg.text_reply(UNKNOWN_MSG)).await,
    }.map(|_| ())
}

#[derive(Debug)]
enum Command {
    Start,
    Help,
    Roll(usize),
    Unknown,
}

impl Command {
    fn resolve(txt: &str, entities: &[MessageEntity]) -> Option<Command> {
        use telegram_bot::MessageEntityKind::BotCommand;
        use Command::*;

        let ent = entities.first()?;

        if ent.kind != BotCommand || ent.offset != 0 {
            return None;
        }

        let slash = Self::isolate_slash(&txt[ent.offset as usize..(ent.offset + ent.length) as usize])?;

        let cmd = match slash.to_lowercase().as_str() {
            "/start" => Start,
            "/help" => Help,
            "/r" | "/roll" => Roll((ent.offset + ent.length) as usize),
            _ => Unknown,
        };

        Some(cmd)
    }

    fn isolate_slash(txt: &str) -> Option<&str> {
        use crate::BOT_NAME;
        let parts: Vec<&str> = txt.split('@').collect();

        match parts.len() {
            1 => Some(parts[0]),
            2 if parts[1] == BOT_NAME => Some(parts[0]),
            _ => None,
        }
    }
}
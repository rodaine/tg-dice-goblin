use grammers_client::{InputMessage, Update};
use log::{info, trace, warn};
use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::multispace1;
use nom::character::streaming::char;
use nom::combinator::{eof, opt, rest};
use nom::IResult;
use nom::sequence::{preceded, tuple};

use crate::{Result, rolls};
use crate::rolls::Roll;

const START_MSG: &str = "Let *Dice Goblin* roll for you!

Dice Goblin will roll any-sided rolls and perform simple arithmetic to reach a total value, appropriate for many tabletop and RPG games. See /help for details on the commands and syntax available.";

const HELP_MSG: &str = "*COMMANDS*

/start\\
_See introductory information about this bot_

/help\\
_See this help output_

/roll `[expression]`\\
_Rolls and calculate a total (see expression syntax below)_

/r `[expression]`\\
_Alias for /roll_

/`[expression]`\\
_Alias for /roll_

*ROLL EXPRESSION SYNTAX*

Dice rolls are described in the standard `NdS` format, where `N` is the number of rolls and `S` is the number of sides. Each roll is summed together to calculate the overall value.

*Examples:*\\
`3d10` - Roll a ten-sided die three times\\
`d6` - Roll a single six-sided die (N defaults to 1 if omitted)\\
`D2` - flip a coin (The `d` is case-insensitive)\\

Rolls support basic arithmetic using the operators (+, -, \\*, /) as well as parenthesis. Division always rounds towards zero, and division by zero always equals zero.

*Examples:*\\
`3d10 + 2` - Roll three ten-sided rolls and add two to the result\\
`(d6 - 1) * 2` - Roll a six-sided die, subtract one from the roll, and then double the result\\
`3 / 2` - Equals 1 (1.5 rounded towards zero)\\
`1 / 0` - Division by zero always equals zero";

const UNKNOWN_MSG: &str = "Unknown command. Use /help to see available commands";

pub(crate) async fn handle(update: Update) -> Result {
    let msg = match update {
        Update::NewMessage(m) if !m.outgoing() && !m.text().is_empty() => m,
        _ => {
            trace!("ignoring: {:?}", update);
            return Ok(());
        }
    };

    let cmd = Command::from(msg.text());
    match msg.sender() {
        Some(user) if user.id() != msg.chat().id() => msg.reply(cmd).await?,
        _ => msg.respond(cmd).await?,
    };

    Ok(())
}

#[derive(Debug)]
enum Command {
    Start,
    Help,
    Roll(Roll),
    Unknown,
}

impl From<&str> for Command {
    fn from(value: &str) -> Self {
        match parse_command(value) {
            Ok((_, cmd)) => cmd,
            Err(e) => {
                warn!("malformed command received: {}", e);
                Command::Unknown
            }
        }
    }
}

impl Into<InputMessage> for Command {
    fn into(self) -> InputMessage {
        use Command::*;
        match self {
            Start => InputMessage::markdown(START_MSG),
            Help => InputMessage::markdown(HELP_MSG),
            Roll(r) => {
                let result = format!("{} = {}", r.value(), r);
                info!("roll: {}", result);
                InputMessage::markdown(result)
            }
            Unknown => InputMessage::markdown(UNKNOWN_MSG),
        }
    }
}

fn parse_command(input: &str) -> IResult<&str, Command> {
    preceded(
        opt(char('/')),
        alt((
            parse_start,
            parse_help,
            parse_roll,
        )),
    )(input)
}

fn parse_start(input: &str) -> IResult<&str, Command> {
    let (input, _) = tuple((
        tag_no_case("start"),
        alt((multispace1, eof)),
        rest,
    ))(input)?;
    Ok((input, Command::Start))
}

fn parse_help(input: &str) -> IResult<&str, Command> {
    let (input, _) = tuple((
        tag_no_case("help"),
        alt((multispace1, eof)),
        rest,
    ))(input)?;
    Ok((input, Command::Help))
}

fn parse_roll(input: &str) -> IResult<&str, Command> {
    let (input, _) = opt(alt((
        tag_no_case("roll"),
        tag_no_case("r"),
    )))(input)?;
    let roll = rolls::parse(input)?;
    Ok(("", Command::Roll(roll)))
}
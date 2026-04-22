use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::Validator;
use rustyline::Context;
use rustyline::Helper;

const COMMANDS: &[&str] = &[
    "autostart",
    "battery",
    "brightness",
    "device",
    "diagnosis",
    "exit",
    "findmyiqos",
    "flexbattery",
    "flexpuff",
    "help",
    "info",
    "lock",
    "quit",
    "smartgesture",
    "unlock",
    "vibration",
];
const AUTOSTART_ARGS: &[&str] = &["on", "off", "enable", "disable", "status"];
const BRIGHTNESS_ARGS: &[&str] = &["high", "low"];
const DEVICE_ARGS: &[&str] = &["list", "save", "remove"];
const FLEXBATTERY_ARGS: &[&str] = &["performance", "eco", "pause"];
const FLEXPUFF_ARGS: &[&str] = &["enable", "disable", "status"];
const SMART_GESTURE_ARGS: &[&str] = &["enable", "disable"];
const VIBRATION_ARGS: &[&str] = &["charge", "heating", "starting", "terminated", "puffend"];
const ON_OFF_ARGS: &[&str] = &["on", "off"];

pub struct IqosHelper {
    highlighter: MatchingBracketHighlighter,
    hinter: HistoryHinter,
}

impl IqosHelper {
    pub fn new() -> Self {
        IqosHelper {
            highlighter: MatchingBracketHighlighter::new(),
            hinter: HistoryHinter {},
        }
    }
}

fn matching_pairs(values: &[&str], prefix: &str) -> Vec<Pair> {
    values
        .iter()
        .filter(|value| value.starts_with(prefix))
        .map(|value| Pair {
            display: (*value).to_string(),
            replacement: (*value).to_string(),
        })
        .collect()
}

impl Completer for IqosHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Self::Candidate>), ReadlineError> {
        let input = &line[..pos];
        let mut args: Vec<&str> = input.split_whitespace().collect();
        if input
            .chars()
            .last()
            .map(char::is_whitespace)
            .unwrap_or(false)
        {
            args.push("");
        }

        if args.is_empty() {
            return Ok((0, matching_pairs(COMMANDS, "")));
        }

        if args.len() == 1 {
            let current = args[0];
            let start = pos - current.len();

            return Ok((start, matching_pairs(COMMANDS, current)));
        }

        if args.len() == 2 {
            let cmd = args[0];
            let subcmd = args[1];
            let start = pos - subcmd.len();

            let candidates = match cmd {
                "autostart" => matching_pairs(AUTOSTART_ARGS, subcmd),
                "brightness" => matching_pairs(BRIGHTNESS_ARGS, subcmd),
                "device" => matching_pairs(DEVICE_ARGS, subcmd),
                "flexbattery" => matching_pairs(FLEXBATTERY_ARGS, subcmd),
                "flexpuff" => matching_pairs(FLEXPUFF_ARGS, subcmd),
                "smartgesture" => matching_pairs(SMART_GESTURE_ARGS, subcmd),
                "vibration" => matching_pairs(VIBRATION_ARGS, subcmd),
                _ => vec![],
            };

            return Ok((start, candidates));
        }

        if args.len() == 3 && (args[0] == "vibration" || args[..2] == ["flexbattery", "pause"]) {
            let option_value = args[2];
            let start = pos - option_value.len();

            return Ok((start, matching_pairs(ON_OFF_ARGS, option_value)));
        }

        Ok((pos, vec![]))
    }
}

impl Helper for IqosHelper {}

impl Hinter for IqosHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<Self::Hint> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for IqosHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> std::borrow::Cow<'b, str> {
        self.highlighter.highlight_prompt(prompt, default)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> std::borrow::Cow<'h, str> {
        self.highlighter.highlight_hint(hint)
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> std::borrow::Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}

impl Validator for IqosHelper {}

#[cfg(test)]
mod tests {
    use super::*;
    use rustyline::completion::Completer;
    use rustyline::history::DefaultHistory;

    fn complete(line: &str) -> (usize, Vec<String>) {
        let helper = IqosHelper::new();
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);
        let (start, candidates) = helper.complete(line, line.len(), &ctx).unwrap();
        let replacements = candidates
            .into_iter()
            .map(|candidate| candidate.replacement)
            .collect();

        (start, replacements)
    }

    #[test]
    fn completes_registered_commands() {
        let (start, candidates) = complete("flex");

        assert_eq!(start, 0);
        assert_eq!(candidates, vec!["flexbattery", "flexpuff"]);
    }

    #[test]
    fn completes_device_subcommands() {
        let (start, candidates) = complete("device");

        assert_eq!(start, 0);
        assert_eq!(candidates, vec!["device"]);

        let (start, candidates) = complete("device ");

        assert_eq!(start, "device ".len());
        assert_eq!(candidates, vec!["list", "save", "remove"]);
    }

    #[test]
    fn completes_only_supported_brightness_values() {
        let (start, candidates) = complete("brightness ");

        assert_eq!(start, "brightness ".len());
        assert_eq!(candidates, vec!["high", "low"]);
    }

    #[test]
    fn completes_vibration_on_off_values() {
        let (start, candidates) = complete("vibration heating o");

        assert_eq!(start, "vibration heating ".len());
        assert_eq!(candidates, vec!["on", "off"]);
    }

    #[test]
    fn completes_flexbattery_pause_values() {
        let (start, candidates) = complete("flexbattery pause ");

        assert_eq!(start, "flexbattery pause ".len());
        assert_eq!(candidates, vec!["on", "off"]);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Action {
    SubnegotiationBegin,
    Will,
    Wont,
    Do,
    Dont,
}

impl TryFrom<u8> for Action {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            250 => Ok(Action::SubnegotiationBegin),
            251 => Ok(Action::Will),
            252 => Ok(Action::Wont),
            253 => Ok(Action::Do),
            254 => Ok(Action::Dont),
            _ => Err(()),
        }
    }
}

impl Into<u8> for Action {
    fn into(self) -> u8 {
        match self {
            Action::SubnegotiationBegin => 250,
            Action::Will => 251,
            Action::Wont => 252,
            Action::Do => 253,
            Action::Dont => 254,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TelnetOption {
    Echo,
    SuppressGoAhead,
    TerminalType,
    NegotiateAboutWindowSize,
    Unknown(u8),
}

impl From<u8> for TelnetOption {
    fn from(value: u8) -> Self {
        match value {
            1 => TelnetOption::Echo,
            3 => TelnetOption::SuppressGoAhead,
            24 => TelnetOption::TerminalType,
            31 => TelnetOption::NegotiateAboutWindowSize,
            _ => TelnetOption::Unknown(value),
        }
    }
}

impl Into<u8> for TelnetOption {
    fn into(self) -> u8 {
        match self {
            TelnetOption::Echo => 1,
            TelnetOption::SuppressGoAhead => 3,
            TelnetOption::TerminalType => 24,
            TelnetOption::NegotiateAboutWindowSize => 31,
            TelnetOption::Unknown(v) => v,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TelnetSubnegotiation {
    WindowSize(u16, u16),
    TerminalType(TerminalTypeOption),
    Unknown(u8, Vec<u8>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TerminalTypeOption {
    Is(String),
    Send,
}

impl From<Vec<u8>> for TelnetSubnegotiation {
    fn from(mut value: Vec<u8>) -> Self {
        match value.remove(0) {
            24 => match value.remove(0) {
                0 => TelnetSubnegotiation::TerminalType(TerminalTypeOption::Is(
                    String::from_utf8(value).unwrap(),
                )),
                1 => TelnetSubnegotiation::TerminalType(TerminalTypeOption::Send),
                _ => TelnetSubnegotiation::Unknown(24, value),
            },
            31 => {
                let width = u16::from(value.remove(0)) << 8 | u16::from(value.remove(0));
                let height = u16::from(value.remove(0)) << 8 | u16::from(value.remove(0));
                TelnetSubnegotiation::WindowSize(width, height)
            }
            v => TelnetSubnegotiation::Unknown(v, value),
        }
    }
}

impl Into<Vec<u8>> for TelnetSubnegotiation {
    fn into(self) -> Vec<u8> {
        match self {
            TelnetSubnegotiation::TerminalType(TerminalTypeOption::Send) => vec![24, 1],
            TelnetSubnegotiation::TerminalType(TerminalTypeOption::Is(value)) => {
                let mut data = vec![24, 0];
                data.extend(value.into_bytes());
                data
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TelnetFrame {
    Data(Vec<u8>),
    Command {
        action: Action,
        option: TelnetOption,
    },
    Subnegotiation(TelnetSubnegotiation),
}

use direction::CardinalDirection;

pub enum Input {
    Direction(CardinalDirection),
    ActiveMeterSelect(ActiveMeterIdentifier),
    MeterDeselect,
    Wait,
}

#[derive(Debug, Clone, Copy)]
pub enum ActiveMeterIdentifier {
    _1 = 0,
    _2 = 1,
    _3 = 2,
    _4 = 3,
    _5 = 4,
    _6 = 5,
    _7 = 6,
    _8 = 7,
    _9 = 8,
    _0 = 9,
}

impl ActiveMeterIdentifier {
    pub fn from_char(c: char) -> Self {
        use self::ActiveMeterIdentifier::*;
        match c {
            '0' => _0,
            '1' => _1,
            '2' => _2,
            '3' => _3,
            '4' => _4,
            '5' => _5,
            '6' => _6,
            '7' => _7,
            '8' => _8,
            '9' => _9,
            _ => panic!("Non-digit character"),
        }
    }

    pub fn to_char(self) -> char {
        use self::ActiveMeterIdentifier::*;
        match self {
            _1 => '1',
            _2 => '2',
            _3 => '3',
            _4 => '4',
            _5 => '5',
            _6 => '6',
            _7 => '7',
            _8 => '8',
            _9 => '9',
            _0 => '0',
        }
    }

    pub fn to_index(self) -> usize {
        self as usize
    }

    pub fn from_index(index: usize) -> Self {
        use self::ActiveMeterIdentifier::*;
        match index {
            0 => _1,
            1 => _2,
            2 => _3,
            3 => _4,
            4 => _5,
            5 => _6,
            6 => _7,
            7 => _8,
            8 => _9,
            9 => _0,
            _ => panic!("out of bounds"),
        }
    }
}

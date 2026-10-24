//! Décodage du protocole infrarouge NEC.

/// Impulsions plus courtes que cette durée (en µs) sont considérées comme du bruit.
pub(crate) const GLITCH_US: u16 = 150;

const LEADER_MARK_MIN: u16 = 6000;
const LEADER_MARK_MAX: u16 = 12000;
const LEADER_SPACE_MIN: u16 = 3000;
const LEADER_SPACE_MAX: u16 = 6000;
const REPEAT_SPACE_MIN: u16 = 1800;
const REPEAT_SPACE_MAX: u16 = 2800;
const BIT_MARK_MAX: u16 = 1200;
const SPACE_0_MAX: u16 = 1000;
const SPACE_1_MIN: u16 = 1200;
const SPACE_1_MAX: u16 = 2500;

/// Un segment du signal IR, exprimé en microsecondes.
#[derive(Clone, Copy)]
pub(crate) struct Edge {
    pub high: bool,
    pub len_us: u16,
}

impl Edge {
    pub const fn new(high: bool, len_us: u16) -> Self {
        Self { high, len_us }
    }
}

/// Résultat du décodage d'une réception NEC.
pub(crate) enum Decoded {
    /// Code du bouton pressé.
    Command(u8),
    /// La touche reste enfoncée (trame de répétition NEC).
    Repeat,
}

/// Décode une suite de segments en une trame NEC.
pub(crate) fn decode(edges: &[Edge]) -> Option<Decoded> {
    let mut i = 0;

    while i + 1 < edges.len() {
        let mark = edges[i];
        let space = edges[i + 1];

        if !mark.high
            && (LEADER_MARK_MIN..=LEADER_MARK_MAX).contains(&mark.len_us)
            && space.high
            && (LEADER_SPACE_MIN..=LEADER_SPACE_MAX).contains(&space.len_us)
        {
            break;
        }
        i += 1;
    }

    if i + 1 >= edges.len() {
        return None;
    }
    i += 2;

    // Trame de répétition : leader + espace ~2.25 ms + mark final.
    if i + 1 < edges.len()
        && !edges[i].high
        && edges[i].len_us <= BIT_MARK_MAX
        && edges[i + 1].high
        && (REPEAT_SPACE_MIN..=REPEAT_SPACE_MAX).contains(&edges[i + 1].len_us)
    {
        return Some(Decoded::Repeat);
    }

    let mut raw = 0u32;

    for bit in 0..32 {
        if i + 1 >= edges.len() {
            return None;
        }

        let mark = edges[i];
        let space = edges[i + 1];

        if mark.high || mark.len_us > BIT_MARK_MAX || !space.high {
            return None;
        }

        let value = if space.len_us <= SPACE_0_MAX {
            0
        } else if (SPACE_1_MIN..=SPACE_1_MAX).contains(&space.len_us) {
            1
        } else {
            return None;
        };

        raw |= (value as u32) << bit;
        i += 2;
    }

    let command = ((raw >> 16) & 0xFF) as u8;
    let command_inv = ((raw >> 24) & 0xFF) as u8;

    if command ^ command_inv != 0xFF {
        return None;
    }

    Some(Decoded::Command(command))
}

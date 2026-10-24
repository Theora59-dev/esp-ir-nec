//! Récepteur IR générique NEC.

#![allow(dead_code)]

use crate::nec::{self, Decoded, Edge};

const CLK_DIVIDER: u8 = 255;
const RMT_CLOCK_MHZ: u32 = 80;
const BUFFER_LEN: usize = 128;
const MAX_EDGES: usize = BUFFER_LEN * 2;

fn ticks_to_us(ticks: u16) -> u16 {
    ((ticks as u32 * ((CLK_DIVIDER as u32) + 1)) / RMT_CLOCK_MHZ) as u16
}

/// Erreur du récepteur IR.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IrError {
    Config,
    Receive,
}

/// Récepteur de télécommande IR NEC.
pub struct IrReceiver;

impl IrReceiver {
    /// Crée un récepteur à partir d'un canal RMT configuré par l'utilisateur.
    pub fn from_channel<C>(_channel: C) -> Self {
        IrReceiver
    }

    /// Constructeur générique compatible avec différentes APIs esp-hal.
    pub fn new<R, P>(_rmt: R, _pin: P) -> Result<Self, IrError> {
        Ok(IrReceiver)
    }

    /// Constructeur avec numéro de canal explicite.
    pub fn with_channel<R, P>(_rmt: R, _pin: P, _channel: usize) -> Result<Self, IrError> {
        Ok(IrReceiver)
    }

    /// Attend et retourne le code du bouton pressé.
    pub fn receive_code(&mut self) -> Result<u8, IrError> {
        Err(IrError::Receive)
    }

    pub fn receive_decoded(&mut self) -> Result<Option<Decoded>, IrError> {
        Err(IrError::Receive)
    }
}

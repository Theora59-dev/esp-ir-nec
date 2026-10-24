//! Récepteur IR haut niveau basé sur le périphérique RMT d'Espressif.

use esp_hal::{
    Blocking,
    gpio::interconnect::PeripheralInput,
    peripherals::RMT,
    rmt::{
        Channel, ConfigError, Error as RmtError, PulseCode, Rmt, Rx, RxChannelConfig,
        RxChannelCreator,
    },
    time::Rate,
};

use crate::nec::{self, Decoded, Edge};

const CLK_DIVIDER: u8 = 255;
const RMT_CLOCK_MHZ: u32 = 80;
const IDLE_THRESHOLD: u16 = 10000;
const BUFFER_LEN: usize = 128;
const MAX_EDGES: usize = BUFFER_LEN * 2;

/// Convertit des ticks RMT en microsecondes.
fn ticks_to_us(ticks: u16) -> u16 {
    ((ticks as u32 * (CLK_DIVIDER as u32 + 1)) / RMT_CLOCK_MHZ) as u16
}

/// Erreur du récepteur IR.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IrError {
    /// Configuration du périphérique RMT invalide.
    Config(ConfigError),
    /// Erreur pendant la réception RMT.
    Receive(RmtError),
}

/// Récepteur de télécommande IR NEC.
///
/// # Exemple
///
/// ```no_run
/// use esp_hal::peripherals::Peripherals;
/// use test_ir::IrReceiver;
///
/// let peripherals = Peripherals::take().unwrap();
/// let mut remote = IrReceiver::new(peripherals.RMT, peripherals.GPIO5).unwrap();
///
/// loop {
///     if let Ok(code) = remote.receive_code() {
///         esp_println::println!("0x{:02X}", code);
///     }
/// }
/// ```
pub struct IrReceiver<'d> {
    channel: Option<Channel<'d, Blocking, Rx>>,
    buffer: [PulseCode; BUFFER_LEN],
}

impl<'d> IrReceiver<'d> {
    /// Initialise un récepteur IR sur le canal RMT 7.
    pub fn new(rmt: RMT<'d>, pin: impl PeripheralInput<'d>) -> Result<Self, IrError> {
        let rmt = Rmt::new(rmt, Rate::from_mhz(RMT_CLOCK_MHZ)).map_err(IrError::Config)?;

        let config = RxChannelConfig::default()
            .with_clk_divider(CLK_DIVIDER)
            .with_idle_threshold(IDLE_THRESHOLD);

        let channel = rmt
            .channel7
            .configure_rx(&config)
            .map_err(IrError::Config)?
            .with_pin(pin);

        Ok(Self {
            channel: Some(channel),
            buffer: [PulseCode::default(); BUFFER_LEN],
        })
    }

    /// Attend et retourne le code du bouton pressé.
    ///
    /// Les trames invalides (bruit) et les répétitions sont ignorées.
    pub fn receive_code(&mut self) -> Result<u8, IrError> {
        loop {
            if let Some(Decoded::Command(code)) = self.receive_decoded()? {
                return Ok(code);
            }
        }
    }

    fn receive_decoded(&mut self) -> Result<Option<Decoded>, IrError> {
        let channel = self
            .channel
            .take()
            .expect("IrReceiver::receive_code appelé après une erreur fatale");

        let transaction = match channel.receive(&mut self.buffer) {
            Ok(transaction) => transaction,
            Err((error, channel)) => {
                self.channel = Some(channel);
                return Err(IrError::Receive(error));
            }
        };

        let (count, channel) = match transaction.wait() {
            Ok(result) => result,
            Err((error, channel)) => {
                self.channel = Some(channel);
                return Err(IrError::Receive(error));
            }
        };
        self.channel = Some(channel);

        let mut edges = [Edge::new(false, 0); MAX_EDGES];
        let mut n = 0;

        'pulses: for pulse in &self.buffer[..count] {
            for (level, ticks) in [
                (pulse.level1(), pulse.length1()),
                (pulse.level2(), pulse.length2()),
            ] {
                if ticks == 0 {
                    break 'pulses;
                }

                let len_us = ticks_to_us(ticks);
                if len_us < nec::GLITCH_US {
                    continue;
                }

                let high = bool::from(level);
                if n > 0 && edges[n - 1].high == high {
                    edges[n - 1].len_us = edges[n - 1].len_us.saturating_add(len_us);
                } else if n < MAX_EDGES {
                    edges[n] = Edge::new(high, len_us);
                    n += 1;
                }
            }
        }

        Ok(nec::decode(&edges[..n]))
    }
}

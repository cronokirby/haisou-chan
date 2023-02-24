use smol::{channel, Timer};
use std::error::Error;
use std::time::{Duration, Instant};

/// Represents data in an individual packet.
pub type MessageData = Vec<u8>;

/// Measures bandwidth, in bytes / sec.
pub type Bandwidth = u32;

/// These are the settings configuring the performance properties of a channel.
///
/// A channel is constraint by its bandwidth, measuring how fast data can be
/// pushed onto the channel, and its latency, measuring how long bytes
/// take to transit from one end of the channel to the other.
#[derive(Debug, Clone, Copy)]
pub struct ChannelSettings {
    /// The latency indicates how long each byte takes to travel from
    /// the sender to the receiver.
    pub latency: Duration,
    /// How many bytes the channel can transmit per second.
    pub bandwidth: Bandwidth,
}

/// Represents a sender for the channel.
#[derive(Debug)]
pub struct Sender {
    settings: ChannelSettings,
    last_time: Instant,
    chan: channel::Sender<(Instant, MessageData)>,
}

impl Sender {
    pub async fn send(&mut self, msg: MessageData) -> Result<(), Box<dyn Error>> {
        let transmission_delay =
            Duration::from_secs_f64((msg.len() as f64) / (self.settings.bandwidth as f64));
        let arrival_time =
            Instant::now().max(self.last_time) + self.settings.latency + transmission_delay;
        self.chan.send((arrival_time, msg)).await?;
        self.last_time = arrival_time;
        Ok(())
    }
}

/// Represents a receiver for the channel.
#[derive(Debug)]
pub struct Receiver {
    chan: channel::Receiver<(Instant, MessageData)>,
}

impl Receiver {
    /// Receive a message along the channel.
    ///
    /// This function can block if no message is ready, or if the message
    /// is delayed because of the latency or bandwidth constraints of the channel.
    pub async fn recv(&self) -> Result<MessageData, Box<dyn Error>> {
        let (time, msg) = self.chan.recv().await?;
        Timer::at(time).await;
        Ok(msg)
    }
}

/// Creates a delayed channel.
///
/// This channel is intended to have one sender and one receiver,
/// and will also simulate the delay of
pub fn channel(settings: ChannelSettings) -> (Sender, Receiver) {
    let (sender, receiver) = channel::unbounded();
    (
        Sender {
            settings,
            last_time: Instant::now(),
            chan: sender,
        },
        Receiver { chan: receiver },
    )
}

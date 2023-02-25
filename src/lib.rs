use smol::{channel, Timer};
use std::error::Error;
use std::time::{Duration, Instant};

/// Represents data in an individual packet.
pub type MessageData = Vec<u8>;

/// Measures bandwidth, in bytes / sec.
pub type Bandwidth = u32;

/// Represents a sender for the channel.
///
/// This sender has a global bottleneck for everything being sent,
/// representing the transmission delay for pushing bytes onto the network.
///
/// The sender will immediately return without blocking, however.
/// This represents an operating system with an infinite sending buffer.
/// In practice, there's also a delay here---which we do not model---but this setup
/// is also equivalent to having an in library queue, by moving sending to a different
/// thread, with an unbounded buffer in between.
#[derive(Debug)]
pub struct Sender {
    /// The bandwidth limiting our sending ability.
    bandwidth: Option<Bandwidth>,
    /// The next time the channel will be free to send data.
    next_time: Instant,
    chan: channel::Sender<(Instant, MessageData)>,
}

impl Sender {
    /// Send a message along this channel.
    /// 
    /// All messages share the same bandwidth, and will be delayed accordingly.
    /// 
    /// This function will not block though.
    pub async fn send(&mut self, msg: MessageData) -> Result<(), Box<dyn Error>> {
        let transmission_delay = match self.bandwidth {
            None => Duration::new(0, 0),
            Some(bw) => Duration::from_secs_f64((msg.len() as f64) / (bw as f64)),
        };
        // The packet leaves after the channel is free again, and we've
        // managed to push all of the data making up the packet.
        let departure_time = Instant::now().max(self.next_time) + transmission_delay;
        self.chan.send((departure_time, msg)).await?;
        self.next_time = departure_time;
        Ok(())
    }

    /// Return a new sender with a different bandwidth.
    pub async fn with_bandwidth(mut self, bandwidth: Bandwidth) -> Self {
        self.bandwidth = Some(bandwidth);
        self
    }
}

/// Represents a receiver for the channel.
/// 
/// This receiver will be delayed because of the upstream bandwidth constraints,
/// along with its individual latency constraints.
#[derive(Debug)]
pub struct Receiver {
    latency: Option<Duration>,
    chan: channel::Receiver<(Instant, MessageData)>,
}

impl Receiver {
    /// Receive a message along the channel.
    ///
    /// This function can block if no message is ready, or if the message
    /// is delayed because of the latency or bandwidth constraints of the channel.
    pub async fn recv(&self) -> Result<MessageData, Box<dyn Error>> {
        let (time, msg) = self.chan.recv().await?;
        let time = match self.latency {
            None => time,
            Some(l) => time + l,
        };
        Timer::at(time).await;
        Ok(msg)
    }

    /// Create a new receiver with a set amount of latency.
    pub async fn with_latency(mut self, latency: Duration) -> Self {
        self.latency = Some(latency);
        self
    }
}

/// Creates a delayed channel.
///
/// This channel is delayed because of the transmission delay of the sender,
/// bottlenecked by the speed of the link, and because of the latency to
/// the receiver.
///
/// By default, the receiver will have no latency, and the [`Receiver::with_latency`]
/// method can be used to add a latency.
///
/// Similarly, the sender will have no bandwidth constraint by default,
/// and the [`Sender::with_bandwidth`] method can be used to add once.
///
/// These channels are also packet based, in the sense that senders transmit
/// an entire packet
pub fn channel() -> (Sender, Receiver) {
    let (sender, receiver) = channel::unbounded();
    (
        Sender {
            bandwidth: None,
            next_time: Instant::now(),
            chan: sender,
        },
        Receiver {
            latency: None,
            chan: receiver,
        },
    )
}

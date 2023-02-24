use std::time::Duration;

/// Measures bandwidth, in bytes / sec.
pub type Bandwidth = u32;

/// These are the settings configuring the performance properties of a channel.
pub struct ChannelSettings {
    /// The latency indicates how long each byte takes to travel from
    /// the sender to the receiver.
    pub latency: Duration,
    /// How many bytes the channel can transmit per second.
    pub bandwidth: Bandwidth,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    // Identifiers sent to nsqd representing this client
    pub client_id: Option<String>,
    pub short_id: Option<String>,
    pub long_id: Option<String>,
    pub hostname: Option<String>,
    pub user_agent: String,

    // Compression Settings
    pub deflate: bool,
    pub deflate_level: u16,
    pub snappy: bool,

    pub feature_negotiation: bool,

    // Duration of time between heartbeats.
    pub heartbeat_interval: i64,

    // Timeout used by nsqd before flushing buffered writes (set to 0 to disable).
    pub message_timeout: u32,

    // Size of the buffer (in bytes) used by nsqd for buffering writes to this connection
    pub output_buffer_size: u64,
    pub output_buffer_timeout: u32,

    // Integer percentage to sample the channel (requires nsqd 0.2.25+)
    pub sample_rate: u16,

    // tls_v1 - Bool enable TLS negotiation
    pub tls_v1: bool,
}
use hostname::get_hostname;

impl Default for Config {
    fn default() -> Config {
        Config {
            client_id: get_hostname(),
            short_id: get_hostname(),
            long_id: get_hostname(),
            user_agent: String::from("nsqueue"),
            hostname: get_hostname(),
            deflate: false,
            deflate_level: 6,
            snappy: false,
            feature_negotiation: true,
            heartbeat_interval: 30000,
            message_timeout: 0,
            output_buffer_size: 16384,
            output_buffer_timeout: 250,
            sample_rate: 0,
            tls_v1: false,
        }
    }
}

#[allow(dead_code)]
impl Config {
    pub fn new() -> Config {
        Config{ ..Default::default() }
    }

    pub fn client_id(mut self, client_id: String) -> Self {
        self.client_id = Some(client_id);
        self
    }

    pub fn hostname(mut self, hostname: String) -> Self {
        self.hostname = Some(hostname);
        self
    }

    pub fn user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = user_agent;
        self
    }

    pub fn snappy(mut self, snappy: bool) -> Self {
        self.snappy = snappy;
        self
    }    
}

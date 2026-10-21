pub enum OutboundFrames {
    ExchangeClosed,
    Queued(Vec<String>),
}

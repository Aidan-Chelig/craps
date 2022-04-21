pub struct CrapsMessage {
    action: u16,
    message: String,
}

pub enum CrapsAction {
    Join = 0,
}

pub struct SyncSession{
    pub token: String,
    pub url: String,
    pub created_at: String
}

pub struct BatchIdsSyncSession{
    pub uuid: String,
    pub url: String,
    pub created_at: String
}

pub fn open_sync_session(){}

pub fn close_sync_session(){}

pub fn send_batch(){}

pub fn receive_batch(){}

pub fn ack_batch(){}


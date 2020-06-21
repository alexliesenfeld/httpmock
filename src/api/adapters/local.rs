use crate::server::data::MockServerState;
use std::sync::Arc;

pub struct LocalMockServerAdapter {
    shutdown_sender: Option<tokio::sync::oneshot::Sender<()>>,
    local_state: Arc<MockServerState>
}

impl LocalMockServerAdapter {
    pub(crate) fn new(  shutdown_sender: tokio::sync::oneshot::Sender<()>, local_state: Arc<MockServerState>) -> LocalMockServerAdapter {
        LocalMockServerAdapter { shutdown_sender: Some(shutdown_sender), local_state }
    }
}

impl Drop for LocalMockServerAdapter {
    fn drop(&mut self) {
        println!("IN DROP!");
        let shutdown_sender = std::mem::replace(&mut self.shutdown_sender, None);
        let shutdown_sender = shutdown_sender.expect("Cannot get shutdown sender.");
        if let Err(e) = shutdown_sender.send(()) {
            println!("Cannot send mock server shutdown signal.");
            log::warn!("Cannot send mock server shutdown signal: {:?}", e)
        }
    }
}

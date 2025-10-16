use crate::models::StreamFrame;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::io::unix::AsyncFd;

#[derive(Clone)]
pub struct AppState { pub jobs: Arc<RwLock<HashMap<String, JobHandle>>>, pub sessions: Arc<RwLock<HashMap<String, SessionHandle>>> }
impl AppState { pub fn new() -> Self { Self { jobs: Arc::new(RwLock::new(HashMap::new())), sessions: Arc::new(RwLock::new(HashMap::new())) } } }

pub struct JobHandle { pub latest_seq:Arc<Mutex<u64>>, pub tx:broadcast::Sender<StreamFrame>, pub exit_code:Arc<Mutex<Option<i32>>> }

#[derive(Clone)]
pub struct SessionHandle {
    pub latest_seq:Arc<Mutex<u64>>,
    pub tx:broadcast::Sender<StreamFrame>,
    pub exit_code:Arc<Mutex<Option<i32>>>,
    pub reader: Arc<AsyncFd<std::fs::File>>,
    pub writer: Arc<AsyncFd<std::fs::File>>,
    pub pid:i32
}

pub mod ids { use uuid::Uuid; pub fn new_id(prefix:&str)->String{format!("{}_{}",prefix,Uuid::new_v4())} }

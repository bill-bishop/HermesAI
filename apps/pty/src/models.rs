use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[derive(Deserialize)] pub struct ExecRequest { pub cmd: Vec<String>, pub cwd: Option<String>, pub env: Option<HashMap<String,String>>, pub timeout_ms: Option<u64> }
#[derive(Serialize)] pub struct ExecResponse { pub job_id: String, pub stream_url: String, pub status_url: String, pub cancel_url: String }
#[derive(Serialize, Deserialize, Clone)] pub struct StreamFrame { pub t:String, pub seq:u64, pub d:String }
#[derive(Serialize)] pub struct StatusResponse { pub state:String, pub exit_code:Option<i32>, pub seq_latest:u64 }
#[derive(Deserialize)] pub struct SessionRequest { pub shell:Option<String>, pub cols:Option<u16>, pub rows:Option<u16> }
#[derive(Serialize)] pub struct SessionResponse { pub session_id:String, pub stream_url:String, pub write_url:String, pub resize_url:String, pub close_url:String }
#[derive(Deserialize)] pub struct WriteRequest { pub data:String }
#[derive(Deserialize)] pub struct ResizeRequest { pub cols:u16, pub rows:u16 }

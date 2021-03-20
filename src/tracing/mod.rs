use enr::NodeId;
use protobuf::Message;
use chrono::Local;
use protobuf::well_known_types::Timestamp;
use std::io::Write;

pub mod generated;

const PATH: &str = "tracing.log";

pub fn clear_log() {
    if std::path::Path::new(PATH).exists() {
        std::fs::remove_file(PATH).unwrap();
    }
}

pub fn node_started(node_id: NodeId) {
    let mut node_started = generated::tracing::Log_NodeStarted::new();
    node_started.set_node_id(format!("{}", node_id));

    let mut log = generated::tracing::Log::new();
    log.set_timestamp(timestamp());
    log.set_node_started(node_started);

    write(log.write_length_delimited_to_bytes().expect("Must be decoded to bytes"), PATH);
}

fn timestamp() -> Timestamp {
    let time = Local::now();
    let timestamp_nanos = time.timestamp_nanos();
    let seconds = timestamp_nanos / 1_000_000_000;
    let nanos = timestamp_nanos - (seconds * 1_000_000_000);
    // println!("timestamp_nanos: {:?}", timestamp_nanos);
    // println!("seconds: {:?}", seconds);
    // println!("nano: {:?}", nano);

    let mut timestamp = Timestamp::new();
    timestamp.set_seconds(seconds);
    timestamp.set_nanos(std::convert::TryFrom::try_from(nanos).unwrap());
    // println!("timestamp: {:?}", timestamp);

    timestamp
}

fn write(bytes: Vec<u8>, path: &str) {
    println!("bytes: {:?}", bytes);

    let mut file = std::fs::OpenOptions::new()
        .append(true)
        // 書き換える場合
        // .write(true)
        // .truncate(true)
        .create(true)
        .open(path)
        .unwrap();
    file.write_all(&bytes).unwrap();
    file.flush().unwrap();
}

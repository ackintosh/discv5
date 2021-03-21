use enr::NodeId;
use protobuf::Message;
use chrono::Local;
use protobuf::well_known_types::Timestamp;
use std::io::Write;
use crate::rpc::{Request, RequestBody, Response, ResponseBody};
use crate::tracing::generated::tracing::{Log_SendOrdinaryMessage, Log, Log_SendWhoAreYou, Log_SendHandshakeMessage, Log_SendHandshakeMessage_Record, Ping, Pong, FindNode, Nodes};
use crate::packet::IdNonce;
use std::convert::TryFrom;
use crate::Enr;

pub mod generated;

const PATH: &str = "tracing.log";

enum ProtocolMessage {
    Ping(Ping),
    Pong(Pong),
    FindNode(FindNode),
    Nodes(Nodes),
}

impl From<&Request> for ProtocolMessage {
    fn from(request: &Request) -> Self {
        match &request.body {
            RequestBody::Ping {enr_seq} => {
                let mut ping = Ping::new();
                ping.set_request_id(request.id.to_string());
                ping.set_enr_seq(*enr_seq);
                ProtocolMessage::Ping(ping)
            }
            RequestBody::FindNode {distances} => {
                let mut find_node = FindNode::new();
                find_node.set_request_id(request.id.to_string());
                find_node.set_distances(distances.clone());
                ProtocolMessage::FindNode(find_node)
            }
            _ => unreachable!()
        }
    }
}

// impl From<&Response> for ProtocolMessage {
//     fn from(response: &Response) -> Self {
//         match response.body {
//             ResponseBody::Pong {enr_seq, ip, port} => {
//                 let mut pong = Pong::new();
//                 pong.set_request_id(response.id.to_string());
//                 pong.set_enr_seq(enr_seq);
//                 pong.set_recipient_ip(format!("{}", ip));
//                 pong.set_recipient_port(port.into());
//                 ProtocolMessage::Pong(pong)
//             }
//         }
//     }
// }

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

    write(log);
}

pub fn send_rpc_request(sender: NodeId, recipient: NodeId, request: &Request) {
    match request.body {
        RequestBody::Ping {enr_seq} => {
            let mut ping = Ping::new();
            ping.set_request_id(request.id.to_string());
            ping.set_enr_seq(enr_seq);

            let mut send_ordinary_message = Log_SendOrdinaryMessage::new();
            send_ordinary_message.set_sender(format!("{}", sender));
            send_ordinary_message.set_recipient(format!("{}", recipient));
            send_ordinary_message.set_ping(ping);

            let mut log = generated::tracing::Log::new();
            log.set_timestamp(timestamp());
            log.set_send_ordinary_message(send_ordinary_message);

            write(log);
        }
        _ => {}
    };
}

pub fn send_rpc_response(sender: NodeId, recipient: &NodeId, response: &Response) {
    match response.body {
        ResponseBody::Pong {enr_seq, ip, port} => {
            let mut pong = Pong::new();
            pong.set_request_id(response.id.to_string());
            pong.set_enr_seq(enr_seq);
            pong.set_recipient_ip(format!("{}", ip));
            pong.set_recipient_port(port.into());

            let mut send_ordinary_message = Log_SendOrdinaryMessage::new();
            send_ordinary_message.set_sender(format!("{}", sender));
            send_ordinary_message.set_recipient(format!("{}", recipient));
            send_ordinary_message.set_pong(pong);

            let mut log = generated::tracing::Log::new();
            log.set_timestamp(timestamp());
            log.set_send_ordinary_message(send_ordinary_message);

            write(log);
        }
        _ => {}
    }
}

pub fn send_whoareyou(sender: &NodeId, recipient: &NodeId, id_nonce: &IdNonce, enr_seq: u64) {
    let mut whoareyou = Log_SendWhoAreYou::new();
    whoareyou.set_sender(format!("{}", sender));
    whoareyou.set_recipient(format!("{}", recipient));
    whoareyou.set_id_nonce(id_nonce.iter().map(|&n| u32::try_from(n).unwrap()).collect::<Vec<u32>>());
    whoareyou.set_enr_seq(enr_seq);

    let mut log = generated::tracing::Log::new();
    log.set_timestamp(timestamp());
    log.set_send_whoareyou(whoareyou);

    write(log);
}

pub fn send_handshake_message(sender: &NodeId, recipient: &NodeId, updated_enr: &Option<Enr>, request: &Request) {
    let mut handshake_message = Log_SendHandshakeMessage::new();
    handshake_message.set_sender(format!("{}", sender));
    handshake_message.set_recipient(format!("{}", recipient));
    if let Some(enr) = updated_enr {
        let mut record = Log_SendHandshakeMessage_Record::new();
        record.set_enr_seq(enr.seq());
        handshake_message.set_record(record);
    }

    let protocol_message: ProtocolMessage = request.into();
    match protocol_message {
        ProtocolMessage::Ping(ping) => {
            handshake_message.set_ping(ping);
        }
        ProtocolMessage::FindNode(find_node) => {
            handshake_message.set_find_node(find_node);
        }
        _ => unreachable!()
    }

    let mut log = generated::tracing::Log::new();
    log.set_timestamp(timestamp());
    log.set_send_handshake_message(handshake_message);

    write(log);
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

fn write(log: Log) {
    let bytes = log.write_length_delimited_to_bytes().expect("Must be decoded to bytes");
    println!("bytes: {:?}", bytes);

    let mut file = std::fs::OpenOptions::new()
        .append(true)
        // 書き換える場合
        // .write(true)
        // .truncate(true)
        .create(true)
        .open(PATH)
        .unwrap();
    file.write_all(&bytes).unwrap();
    file.flush().unwrap();
}

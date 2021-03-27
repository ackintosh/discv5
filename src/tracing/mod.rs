use enr::NodeId;
use protobuf::{Message, RepeatedField};
use chrono::Local;
use protobuf::well_known_types::Timestamp;
use std::io::Write;
use crate::rpc::{Request, RequestBody, Response, ResponseBody};
use crate::tracing::generated::tracing::{Log_SendOrdinaryMessage, Log, Log_SendWhoAreYou, Log_SendHandshakeMessage, Log_SendHandshakeMessage_Record, Ping, Pong, FindNode, Nodes, Random, Log_Shutdown, Log_Start};
use crate::packet::IdNonce;
use std::convert::TryFrom;
use crate::Enr;

pub mod generated;

const PATH: &str = "tracing.log";

enum ProtocolMessageRequest {
    Ping(Ping),
    FindNode(FindNode),
}

enum ProtocolMessageResponse {
    Pong(Pong),
    Nodes(Nodes),
}

impl From<&Request> for ProtocolMessageRequest {
    fn from(request: &Request) -> Self {
        match &request.body {
            RequestBody::Ping {enr_seq} => {
                let mut ping = Ping::new();
                ping.set_request_id(request.id.to_string());
                ping.set_enr_seq(*enr_seq);
                ProtocolMessageRequest::Ping(ping)
            }
            RequestBody::FindNode {distances} => {
                let mut find_node = FindNode::new();
                find_node.set_request_id(request.id.to_string());
                find_node.set_distances(distances.clone());
                ProtocolMessageRequest::FindNode(find_node)
            }
            RequestBody::Talk { protocol, request } => todo!(),
            RequestBody::RegisterTopic { topic, enr, ticket} => todo!(),
            RequestBody::TopicQuery { topic } => todo!(),
        }
    }
}

impl From<&Response> for ProtocolMessageResponse {
    fn from(response: &Response) -> Self {
        match &response.body {
            ResponseBody::Pong {enr_seq, ip, port} => {
                let mut pong = Pong::new();
                pong.set_request_id(response.id.to_string());
                pong.set_enr_seq(*enr_seq);
                pong.set_recipient_ip(format!("{}", ip));
                pong.set_recipient_port(u32::try_from(*port).unwrap());
                ProtocolMessageResponse::Pong(pong)
            }
            ResponseBody::Nodes {total, nodes} => {
                let mut nodes_message = Nodes::new();
                nodes_message.set_request_id(response.id.to_string());
                nodes_message.set_total(*total);
                let node_ids = nodes.iter()
                    .map(|n| format!("{}", n.node_id()))
                    .collect::<Vec<String>>();
                nodes_message.set_nodes(RepeatedField::from_vec(node_ids));
                ProtocolMessageResponse::Nodes(nodes_message)
            }
            ResponseBody::Talk { response } => todo!(),
            ResponseBody::Ticket { ticket, wait_time } => todo!(),
            ResponseBody::RegisterConfirmation { topic } => todo!(),
        }
    }
}

pub fn clear_log() {
    if std::path::Path::new(PATH).exists() {
        std::fs::remove_file(PATH).unwrap();
    }
}

pub fn start(node_id: NodeId) {
    let mut start = Log_Start::new();
    start.set_node_id(format!("{}", node_id));

    let mut log = Log::new();
    log.set_timestamp(timestamp());
    log.set_start(start);

    write(log);
}

pub fn shutdown(node_id: NodeId) {
    let mut shutdown = Log_Shutdown::new();
    shutdown.set_node_id(format!("{}", node_id));

    let mut log = Log::new();
    log.set_timestamp(timestamp());
    log.set_shutdown(shutdown);

    write(log);
}

pub fn send_random_packet(sender: &NodeId, recipient: &NodeId) {
    let mut send_ordinary_message = Log_SendOrdinaryMessage::new();
    send_ordinary_message.set_sender(format!("{}", sender));
    send_ordinary_message.set_recipient(format!("{}", recipient));
    send_ordinary_message.set_random(Random::new());

    let mut log = generated::tracing::Log::new();
    log.set_timestamp(timestamp());
    log.set_send_ordinary_message(send_ordinary_message);

    write(log);
}

pub fn send_rpc_request(sender: &NodeId, recipient: &NodeId, request: &Request) {
    let mut send_ordinary_message = Log_SendOrdinaryMessage::new();
    send_ordinary_message.set_sender(format!("{}", sender));
    send_ordinary_message.set_recipient(format!("{}", recipient));

    let protocol_message: ProtocolMessageRequest = request.into();
    match protocol_message {
        ProtocolMessageRequest::Ping(ping) => {
            send_ordinary_message.set_ping(ping);
        }
        ProtocolMessageRequest::FindNode(find_node) => {
            send_ordinary_message.set_find_node(find_node);
        }
    }

    let mut log = Log::new();
    log.set_timestamp(timestamp());
    log.set_send_ordinary_message(send_ordinary_message);

    write(log);
}

pub fn send_rpc_response(sender: &NodeId, recipient: &NodeId, response: &Response) {
    let mut send_ordinary_message = Log_SendOrdinaryMessage::new();
    send_ordinary_message.set_sender(format!("{}", sender));
    send_ordinary_message.set_recipient(format!("{}", recipient));

    let protocol_message: ProtocolMessageResponse = response.into();
    match protocol_message {
        ProtocolMessageResponse::Pong(pong) => {
            send_ordinary_message.set_pong(pong);
        }
        ProtocolMessageResponse::Nodes(nodes) => {
            send_ordinary_message.set_nodes(nodes);
        }
    }

    let mut log = Log::new();
    log.set_timestamp(timestamp());
    log.set_send_ordinary_message(send_ordinary_message);

    write(log);
}

pub fn send_whoareyou(sender: &NodeId, recipient: &NodeId, id_nonce: &IdNonce, enr_seq: u64) {
    let mut whoareyou = Log_SendWhoAreYou::new();
    whoareyou.set_sender(format!("{}", sender));
    whoareyou.set_recipient(format!("{}", recipient));
    whoareyou.set_id_nonce(id_nonce.iter().map(|&n| u32::try_from(n).unwrap()).collect::<Vec<u32>>());
    whoareyou.set_enr_seq(enr_seq);

    let mut log = Log::new();
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

    let protocol_message: ProtocolMessageRequest = request.into();
    match protocol_message {
        ProtocolMessageRequest::Ping(ping) => {
            handshake_message.set_ping(ping);
        }
        ProtocolMessageRequest::FindNode(find_node) => {
            handshake_message.set_find_node(find_node);
        }
    }

    let mut log = Log::new();
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

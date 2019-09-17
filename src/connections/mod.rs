
mod server;
mod client;

pub use server::Server;
pub use client::Client;

use crate::error::{Result, BenchError};

const INIT_PORT: usize = 30000;

#[derive(Clone, Copy)]
pub enum Action {
    Loading,
    Benchmarking
}

impl Action {
    pub fn as_int(&self) -> i32 {
        match self {
            Action::Loading => 1,
            Action::Benchmarking => 2
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct ConnectionInfo {
    pub id: usize,
    pub ip: String,
    pub port: usize
}

impl ConnectionInfo {
    pub fn generate_connection_list(ip_list: &Vec<String>,
        conn_count: usize, max_conn_per_ip: usize)
        -> Result<Vec<ConnectionInfo>> {
        
        let mut list = Vec::new();
        let mut id: usize = 0;
        let mut conn_per_node = 1;
        'out: loop {

            for ip in ip_list {
                list.push(ConnectionInfo {
                    id,
                    ip: ip.to_owned(),
                    port: INIT_PORT + conn_per_node - 1
                });

                id += 1;
                if id >= conn_count {
                    break 'out;
                }
            }

            conn_per_node += 1;
            if conn_per_node > max_conn_per_ip {
                return Err(BenchError::Message(
                    format!("The number of machines is not enough.")));
            }
        }

        Ok(list)
    }

    pub fn to_string(&self) -> String {
        format!("{} {} {}", self.id, self.ip, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_list() {
        let ip_list = vec![
            "192.168.1.1".to_owned(),
            "192.168.1.2".to_owned(),
            "192.168.1.3".to_owned()
        ];
        let list = ConnectionInfo::generate_connection_list(
            &ip_list, 5, 2).unwrap();

        let expected = vec![
            ConnectionInfo {
                id: 0,
                ip: "192.168.1.1".to_owned(),
                port: 30000
            },
            ConnectionInfo {
                id: 1,
                ip: "192.168.1.2".to_owned(),
                port: 30000
            },
            ConnectionInfo {
                id: 2,
                ip: "192.168.1.3".to_owned(),
                port: 30000
            },
            ConnectionInfo {
                id: 3,
                ip: "192.168.1.1".to_owned(),
                port: 30001
            },
            ConnectionInfo {
                id: 4,
                ip: "192.168.1.2".to_owned(),
                port: 30001
            },
        ];

        assert_eq!(&list, &expected);
    }
}
// Copyright 2023 tison <wander4096@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::time::Duration;

use crossbeam::sync::WaitGroup;
use mephistio_etcd::raft::{node::RaftNode, service::start_raft_service};
use mephisto_raft::Peer;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use tracing::Level;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let peers = vec![
        Peer {
            id: 1,
            address: "127.0.0.1:10386".to_string(),
        },
        Peer {
            id: 2,
            address: "127.0.0.1:10486".to_string(),
        },
        Peer {
            id: 3,
            address: "127.0.0.1:10586".to_string(),
        },
    ];

    let mut shutdowns = vec![];
    for peer in peers.iter() {
        let (runtime, tx_outbound, rx_inbound) = start_raft_service(peer.clone(), peers.clone())?;
        let node = RaftNode::new(peer.clone(), peers.clone(), rx_inbound, tx_outbound)?;
        let shutdown_node = node.tx_shutdown();
        node.run();
        shutdowns.push((runtime, shutdown_node));
    }

    std::thread::sleep(Duration::from_secs(5));

    let wg = WaitGroup::new();
    let wgs = (0..shutdowns.len()).map(|_| wg.clone()).collect::<Vec<_>>();
    shutdowns
        .into_par_iter()
        .zip(wgs)
        .for_each(|((rt, tx), wg)| {
            let _ = tx.send(());
            drop(rt);
            drop(wg);
        });
    wg.wait();

    std::thread::sleep(Duration::from_secs(5));

    Ok(())
}

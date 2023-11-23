use std::{env::temp_dir, sync::Arc};

use mephistio_etcd::etcd::{
    pb::etcdserverpb::kv_server::KvServer, service::KvService, state::EtcdState,
};
use tokio::sync::Mutex;
use tonic::transport::Server;

fn main() -> anyhow::Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let state = EtcdState::new(dbg!(temp_dir()));
    let service = KvService {
        state: Arc::new(Mutex::new(state)),
    };

    runtime.block_on(async move {
        let addr = "127.0.0.1:2379".parse()?;
        Server::builder()
            .add_service(KvServer::new(service))
            .serve(addr)
            .await?;
        Ok(())
    })
}

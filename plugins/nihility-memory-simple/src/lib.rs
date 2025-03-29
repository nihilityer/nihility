mod graph;
mod retrieval;

use anyhow::Result;
use graph::KnowledgeGraph;
use lazy_static::lazy_static;
use nihility_common::config::{NihilityConfigType, get_config};
use nihility_common::idea::Idea;
use nihility_common::inspiration::Inspiration;
use nihility_common::model::get_chat_completion;
use nihility_common::{register_idea_receiver_plugin, register_inspiration_plugin};
use retrieval::HippoRAGRetriever;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use tokio::sync::broadcast::Sender;
use tokio::sync::{Mutex, broadcast, mpsc};
use tracing::{debug, error, info};

lazy_static! {
    static ref CORE: Mutex<Option<NihilitySimpleMemory>> = Mutex::new(None);
}

#[derive(Clone)]
pub struct NihilitySimpleMemory {
    config: NihilitySimpleMemoryConfig,
    graph: KnowledgeGraph,
    retriever: HippoRAGRetriever,
    memory_sender: Sender<Inspiration>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NihilitySimpleMemoryConfig {
    damping_factor: f32,
    store_path: String,
    refine_prompt: String,
}

impl Default for NihilitySimpleMemoryConfig {
    fn default() -> Self {
        Self {
            damping_factor: 0.85,
            store_path: "knowledge_graph.json".to_string(),
            refine_prompt: String::new(),
        }
    }
}

impl NihilitySimpleMemory {
    pub async fn init() -> Result<()> {
        info!("Initializing NihilitySimpleMemory");
        let config = get_config::<NihilitySimpleMemoryConfig>(
            env!("CARGO_PKG_NAME").to_string(),
            NihilityConfigType::Base,
        )
        .await?;
        let file_path = Path::new(config.store_path.as_str());

        let graph = if file_path.exists() {
            let mut graph_store = File::open(file_path)?;
            let mut buffer = Vec::new();
            graph_store.read_to_end(&mut buffer)?;
            if let Ok(graph) = serde_json::from_slice::<KnowledgeGraph>(&buffer) {
                graph
            } else {
                KnowledgeGraph::new()
            }
        } else {
            KnowledgeGraph::new()
        };
        let retriever = HippoRAGRetriever::new(0.85);

        let (tx, _) = broadcast::channel(10);
        let (core_rx, mut core_tx) = mpsc::channel::<String>(10);
        let core = Self {
            config: config.clone(),
            graph,
            retriever,
            memory_sender: tx,
        };
        register_inspiration_plugin(core.memory_sender.subscribe()).await?;
        CORE.lock().await.replace(core);

        tokio::spawn(async move {
            while let Some(idea) = core_tx.recv().await {
                // TODO 模块内部判断想法为什么
                debug!("Received idea from {}", idea);
            }
        });

        tokio::spawn(async move {
            let mut idea_receiver = register_idea_receiver_plugin().await;
            while let Ok(idea) = idea_receiver.recv().await {
                if let Idea::Memory(memory_idea) = idea {
                    if let Err(e) = core_rx.send(memory_idea).await {
                        error!("Failed to send memory query: {:?}", e);
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn query_knowledge<T: Into<String>>(&self, text: T) -> Result<Vec<String>> {
        self.graph.query(text, &self.retriever).await
    }

    pub async fn add_knowledge<T: Into<String>>(&mut self, text: T) -> Result<()> {
        let refine_result =
            get_chat_completion(self.config.refine_prompt.clone(), text.into()).await?;
        if let Some(refine_text) = refine_result.get("text") {
            info!("Adding knowledge: {}", refine_text.to_string());
            self.graph.add_knowledge(refine_text.to_string()).await?;
            self.update_store()?;
        }
        Ok(())
    }

    pub fn update_store(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.graph)?;
        let file_path = Path::new(self.config.store_path.as_str());
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(file_path)?;
        file.write_all(json.as_bytes())?;
        file.sync_all()?;
        Ok(())
    }
}

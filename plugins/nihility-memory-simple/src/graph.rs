use crate::retrieval::HippoRAGRetriever;
use anyhow::Result;
use nihility_common::model::{get_chat_completion, get_embedding};
use petgraph::{graph::NodeIndex, stable_graph::StableDiGraph};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

static SYSTEM_PROMPT: &str = "You are a precise information extraction system. Extract relationships with confidence scores.";

#[derive(Debug, Clone)]
pub struct KnowledgeNode {
    pub text: String,
    pub embedding: Vec<f32>,
    pub specificity: f32,
}

#[derive(Debug, Clone)]
pub struct KnowledgeEdge {
    pub relation: String,
    pub confidence: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KnowledgeTriple {
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub confidence: f32,
}

#[derive(Clone)]
pub struct KnowledgeGraph {
    pub graph: StableDiGraph<KnowledgeNode, KnowledgeEdge>,
    pub node_indices: HashMap<String, NodeIndex>,
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self {
            graph: StableDiGraph::new(),
            node_indices: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, text: String, embedding: Vec<f32>) -> NodeIndex {
        let node = KnowledgeNode {
            text: text.clone(),
            embedding,
            specificity: 1.0,
        };
        let idx = self.graph.add_node(node);
        self.node_indices.insert(text, idx);
        idx
    }

    pub async fn add_knowledge<T: Into<String>>(&mut self, text: T) -> Result<()> {
        // Step 1: 信息抽取
        let triples = extract_triples(text).await?;
        debug!("triples: {:?}", triples);

        // Step 2: 处理三元组
        for triple in triples {
            let subject_embed = get_embedding(&triple.subject).await?;
            let object_embed = get_embedding(&triple.object).await?;

            // 添加或更新节点
            let subj_idx = self.get_or_create_node(&triple.subject, subject_embed);
            let obj_idx = self.get_or_create_node(&triple.object, object_embed);

            // 添加关系边
            self.graph.add_edge(
                subj_idx,
                obj_idx,
                KnowledgeEdge {
                    relation: triple.relation,
                    confidence: triple.confidence,
                },
            );
        }

        // Step 3: 更新特异性
        self.update_specificity();
        Ok(())
    }

    fn get_or_create_node(&mut self, text: &str, embedding: Vec<f32>) -> NodeIndex {
        if let Some(&idx) = self.node_indices.get(text) {
            idx
        } else {
            self.add_node(text.to_string(), embedding)
        }
    }

    fn update_specificity(&mut self) {
        let total_nodes = self.graph.node_count() as f32;
        for node in self.graph.node_weights_mut() {
            node.specificity = 1.0 / total_nodes;
        }
    }

    pub async fn query<T: Into<String>>(
        &self,
        question: T,
        retriever: &HippoRAGRetriever,
    ) -> Result<Vec<String>> {
        // Step 1: 问题解析
        let entities = self.extract_query_entities(question).await?;
        debug!("entities: {:?}", entities);

        // Step 2: 获取嵌入
        let mut query_nodes = Vec::new();
        for entity in entities {
            let embed = get_embedding(&entity).await?;
            if let Some(closest) = self.find_closest_node(&embed) {
                query_nodes.push(closest.text.clone());
            }
        }

        // Step 3: 执行检索
        let results = retriever.personalized_pagerank(self, &query_nodes)?;
        debug!("results: {:?}", results);

        Ok(results
            .into_iter()
            .take(5)
            .map(|(node, _)| node.clone())
            .collect())
    }

    async fn extract_query_entities<T: Into<String>>(&self, question: T) -> Result<Vec<String>> {
        let prompt = format!("Extract key entities from: {}", question.into());
        let triples = extract_triples(&prompt).await?;
        debug!("triples: {:?}", triples);

        Ok(triples
            .into_iter()
            .flat_map(|t| vec![t.subject, t.object])
            .collect())
    }

    fn find_closest_node(&self, embedding: &[f32]) -> Option<&KnowledgeNode> {
        self.graph.node_weights().min_by_key(|node| {
            // 使用余弦相似度简化计算
            let dot_product: f32 = node
                .embedding
                .iter()
                .zip(embedding)
                .map(|(a, b)| a * b)
                .sum();
            (dot_product * -1.0e6) as i32
        })
    }
}
pub async fn extract_triples<T: Into<String>>(text: T) -> Result<Vec<KnowledgeTriple>> {
    let prompt = format!(
        r#"
        Analyze the following text and extract all factual relationships in JSON format.
        Use strict schema: [{{"subject": "...", "relation": "...", "object": "...", "confidence": 0.0}}]
        Text: {}
        "#,
        text.into()
    );
    Ok(serde_json::from_value(
        get_chat_completion(SYSTEM_PROMPT.to_string(), prompt).await?,
    )?)
}

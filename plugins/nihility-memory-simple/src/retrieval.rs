use petgraph::algo::{page_rank};
use super::graph::{KnowledgeGraph};
use anyhow::Result;
use petgraph::visit::NodeIndexable;

#[derive(Clone)]
pub struct HippoRAGRetriever {
    damping_factor: f32,
}

impl HippoRAGRetriever {
    pub fn new(damping_factor: f32) -> Self {
        Self {
            damping_factor,
        }
    }

    pub fn personalized_pagerank(
        &self,
        graph: &KnowledgeGraph,
        query_nodes: &[String],
    ) -> Result<Vec<(String, f32)>> {
        // 构建个性化向量
        let mut personalization = vec![0.0; graph.graph.node_count()];
        let query_count = query_nodes.len() as f32;

        for node_text in query_nodes {
            if let Some(&idx) = graph.node_indices.get(node_text) {
                personalization[graph.graph.to_index(idx)] = 1.0 / query_count;
            }
        }

        // 使用petgraph的page_rank函数
        let base_ranks = page_rank(&graph.graph, self.damping_factor, 20);

        // 应用个性化调整 (个性化PageRank近似)
        let adjusted_ranks: Vec<f32> = base_ranks.iter()
            .zip(personalization.iter())
            .map(|(base, pers)| base * self.damping_factor + pers * (1.0 - self.damping_factor))
            .collect();

        // 组合结果
        let mut results: Vec<_> = graph.graph.node_indices()
            .map(|idx| {
                let node = &graph.graph[idx];
                (node.text.clone(), adjusted_ranks[graph.graph.to_index(idx)])
            })
            .collect();

        // 使用标准库排序
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(results)
    }
}
use petgraph::dot::{Dot, Config};
use petgraph::graph::{Graph, NodeIndex};
use rustworkx_core::centrality::betweenness_centrality;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use csv::ReaderBuilder;

#[derive(Debug, Clone)]
struct Country {
    // declare fields
    country_name: String,
    country_region: String,
    happiness_score: f64,
    happiness_rank: f64,
    gdp: f64,
    health: f64,
    family: f64,
    gove_corruption: f64,
}

fn read_csv(filename: &str) -> Result<HashMap<String, Country>, Box<dyn Error>> {
    let mut country_map = HashMap::new();

    let file = File::open(filename)?;
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);

    for result in rdr.records() {
        let record = result?;
        let country_name = record.get(0).ok_or("Missing country name")?.to_string();
        let country_region = record.get(1).ok_or("Missing country region")?.to_string();
        let happiness_rank = record.get(2).ok_or("Missing happiness rank")?.parse::<f64>()?;
        let happiness_score = record.get(3).ok_or("Missing happiness score")?.parse::<f64>()?;
        let gdp = record.get(4).ok_or("Missing GDP")?.parse::<f64>()?;
        let health = record.get(5).ok_or("Missing health")?.parse::<f64>()?;
        let family = record.get(6).ok_or("Missing family")?.parse::<f64>()?;
        let gove_corruption = record.get(7).ok_or("Missing government corruption")?.parse::<f64>()?;

        let country = Country {
            country_name: country_name.clone(),
            country_region: country_region.clone(),
            happiness_score,
            happiness_rank,
            gdp,
            health,
            family,
            gove_corruption,
        };

        country_map.insert(country_name, country);
    }

    Ok(country_map)
}

fn build_graph(country_data: &HashMap<String, Country>) -> Graph<Country, ()> {
    let mut graph = petgraph::Graph::<Country, ()>::new();

    // Create a mapping from region names to nodes
    let mut region_nodes: HashMap<&str, Vec<NodeIndex>> = HashMap::new();

    for (_, country) in country_data.iter() {
        let node = graph.add_node(country.clone());

        // Check if the region already has nodes in the graph
        let nodes_in_region = region_nodes.entry(&country.country_region).or_insert(vec![]);
        nodes_in_region.push(node);
    }

    // Add edges between nodes in the same region
    for (_, nodes_in_region) in region_nodes.iter_mut() {
        for &i in nodes_in_region.iter() {
            for &j in nodes_in_region.iter() {
                if i != j {
                    graph.add_edge(i, j, ());
                }
            }
        }
    }

    graph // Return the graph
}

// Add this function to calculate and print the degree of each node
fn print_node_degrees(graph: &Graph<Country, ()>) {
    println!("Node Degrees:");
    for node in graph.node_indices() {
        let degree = graph.neighbors(node).count();
        println!("Node {}: Degree {}", node.index(), degree);
    }
}
fn calculate_betweenness(graph: &Graph<Country, ()>) -> Vec<Option<f64>> {
    betweenness_centrality(graph, false, false, 200)
}

fn visualize_graph(graph: &Graph<Country, ()>) {
    let dot: String = format!("{:?}", Dot::with_config(graph, &[Config::EdgeNoLabel]));
    println!("{}", dot);
}

fn main() -> Result<(), Box<dyn Error>> {
    // Read the CSV file and create a dictionary with country names and happiness scores
    let happiness_data = read_csv("2015.csv")?;

    // Display some information about each country
    for country in happiness_data.values() {
        println!(
            "Country: {}, Region: {}, Happiness Score: {}, Rank: {}, GDP: {}, Health: {}, Family: {}, Corruption: {}",
            country.country_name,
            country.country_region,
            country.happiness_score,
            country.happiness_rank,
            country.gdp,
            country.health,
            country.family,
            country.gove_corruption
        );
    }

    // Build the graph based on Country data
    let mut graph = build_graph(&happiness_data);

    let nodes: Vec<NodeIndex> = happiness_data.iter().map(|(_, country)| {
        graph.add_node(country.clone())
    }).collect();

    // Add edges based on some similarity metric (e.g., difference in happiness score)
    for i in 0..nodes.len() {
        for j in i + 1..nodes.len() {
            let diff = (graph[nodes[i]].happiness_score - graph[nodes[j]].happiness_score).abs();
            if diff < 1.0 {
                graph.add_edge(nodes[i], nodes[j], ());
            }
        }
    }

    let start_node = nodes[0];
    let mut bfs = petgraph::visit::Bfs::new(&graph, start_node);

    // Traverse the graph using BFS
    while let Some(node) = bfs.next(&graph) {
        // Process the node as needed
        let country = &graph[node];
        println!(
            "Node: {} (Happiness Score: {})",
            country.country_name, country.happiness_score
        );
    }

    // Calculate and print betweenness centrality
    let centralities = calculate_betweenness(&graph);
    println!("Betweenness Centrality:");
    for (index, centrality) in centralities.iter().enumerate() {
        if let Some(c) = centrality {
            println!("Node: {}, Centrality: {}", index, c);
        }
    }

    // Visualize the graph
    visualize_graph(&graph);

    // Print the degree of each node in the graph
    print_node_degrees(&graph);

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_csv() {
        // Provide a test CSV file for testing
        let filename = "test_data.csv";

        // Create some test data
        let test_data = "\
            Country,Region,HappinessRank,HappinessScore,GDP,Health,Family,GovernmentCorruption\n\
            Country1,Region1,1,7.5,1.2,0.8,1.5,0.2\n\
            Country2,Region2,2,6.8,1.0,0.9,1.3,0.1\n\
        ";

        // Write test data to a temporary file
        std::fs::write(filename, test_data).expect("Failed to write test data to file");

        // Perform the test
        let result = read_csv(filename);
        assert!(result.is_ok());

        // Clean up the temporary file
        std::fs::remove_file(filename).expect("Failed to remove temporary file");
    }

    #[test]
    fn test_build_graph() {
        // Create a test HashMap with sample data
        let mut test_data = HashMap::new();
        test_data.insert(
            "Country1".to_string(),
            Country {
                country_name: "Country1".to_string(),
                country_region: "Region1".to_string(),
                happiness_score: 7.5,
                happiness_rank: 1.0,
                gdp: 1.2,
                health: 0.8,
                family: 1.5,
                gove_corruption: 0.2,
            },
        );
        test_data.insert(
            "Country2".to_string(),
            Country {
                country_name: "Country2".to_string(),
                country_region: "Region2".to_string(),
                happiness_score: 6.8,
                happiness_rank: 2.0,
                gdp: 1.0,
                health: 0.9,
                family: 1.3,
                gove_corruption: 0.1,
            },
        );

        // Perform the test
        let graph = build_graph(&test_data);

        // You can add more assertions based on your expectations about the graph

        // For example, assert that there are nodes in the graph
        assert!(graph.node_count() > 0);

        // For example, assert that there are edges in the graph
        assert!(graph.edge_count() > 0);
    }
}


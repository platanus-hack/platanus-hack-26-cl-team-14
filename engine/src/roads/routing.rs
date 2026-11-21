use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// Shortest directed travel times in minutes, in origin-major anchor order.
/// Each edge gives its destination node and nonnegative travel time in seconds.
pub(super) fn travel_minutes(adjacency: &[Vec<(usize, f64)>], anchors: &[usize]) -> Vec<f64> {
    let mut is_anchor = vec![false; adjacency.len()];
    for &node in anchors {
        is_anchor[node] = true;
    }
    let destinations = is_anchor.iter().filter(|&&present| present).count();
    let mut minutes = Vec::with_capacity(anchors.len() * anchors.len());
    let mut distance = vec![f64::INFINITY; adjacency.len()];
    let mut queue = BinaryHeap::new();

    for (row, &origin) in anchors.iter().enumerate() {
        if let Some(previous) = anchors[..row].iter().position(|&node| node == origin) {
            let start = previous * anchors.len();
            minutes.extend_from_within(start..start + anchors.len());
            continue;
        }

        distance.fill(f64::INFINITY);
        distance[origin] = 0.0;
        queue.clear();
        // Nonnegative IEEE 754 values sort in the same order as their unsigned bits.
        queue.push(Reverse((0.0_f64.to_bits(), origin)));
        let mut remaining = destinations;
        while let Some(Reverse((bits, node))) = queue.pop() {
            let seconds = f64::from_bits(bits);
            if seconds > distance[node] {
                continue;
            }
            if is_anchor[node] {
                remaining -= 1;
                if remaining == 0 {
                    break;
                }
            }
            for &(next, cost) in &adjacency[node] {
                let arrival = seconds + cost;
                if arrival < distance[next] {
                    distance[next] = arrival;
                    queue.push(Reverse((arrival.to_bits(), next)));
                }
            }
        }
        minutes.extend(anchors.iter().map(|&node| distance[node] / 60.0));
    }
    minutes
}

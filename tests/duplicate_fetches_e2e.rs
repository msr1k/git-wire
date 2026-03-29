use std::collections::HashMap;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}, mpsc};

#[derive(Clone)]
struct Parsed {
    name: Option<String>,
    dsc: Option<String>,
    url: String,
    rev: String,
    src: String,
    dst: String,
    mtd: Option<Method>,
}

#[derive(Clone)]
enum Method {
    Shallow,
    ShallowNoSparse,
    Partial,
}

// This integration-style test simulates the end-to-end grouping + fetch + delivery
#[test]
fn duplicate_fetches_e2e() {
    // Given: three parsed entries where two share the same key
    let p1 = Parsed { name: Some("a".into()), dsc: None, url: "https://example.com/repo.git".into(), rev: "main".into(), src: "src".into(), dst: "dst".into(), mtd: None };
    let p2 = Parsed { name: Some("b".into()), dsc: None, url: "https://example.com/repo.git".into(), rev: "main".into(), src: "src2".into(), dst: "dst2".into(), mtd: None };
    let p3 = Parsed { name: Some("c".into()), dsc: None, url: "https://other/repo.git".into(), rev: "v1".into(), src: "s".into(), dst: "d".into(), mtd: Some(Method::Partial) };

    let parsed = vec![p1, p2, p3];

    // When: group by key
    fn fetch_key(parsed: &Parsed) -> String {
        let mtd_str = match parsed.mtd {
            Some(Method::Partial) => "partial",
            Some(Method::ShallowNoSparse) => "shallow_no_sparse",
            Some(Method::Shallow) => "shallow",
            None => "shallow",
        };
        format!("{}::{}::{}", parsed.url.trim(), parsed.rev.trim(), mtd_str)
    }

    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, p) in parsed.iter().enumerate() {
        groups.entry(fetch_key(p)).or_default().push(i);
    }

    // Simulate fetcher invocations counted
    let counter = AtomicUsize::new(0);

    // Simulate per-parsed consumer receivers (not using real channels here for simplicity)
    let mut delivered_keys = vec![None; 3];

    for (key, indices) in groups.into_iter() {
        // Simulate a fetch operation for this key
        counter.fetch_add(1, Ordering::SeqCst);
        let artifact = Arc::new(format!("artifact_for_{}", key));

        // Deliver artifact to all indices associated with this key
        for idx in indices {
            delivered_keys[idx] = Some(artifact.clone());
        }
    }

    // Then: counter should equal number of unique keys (2 in this scenario)
    assert_eq!(counter.load(Ordering::SeqCst), 2);

    // And: entries that shared the key should have Arc pointers equal
    let a = delivered_keys[0].as_ref().unwrap();
    let b = delivered_keys[1].as_ref().unwrap();
    assert!(Arc::ptr_eq(a, b));
}

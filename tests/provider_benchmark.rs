use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::{Duration, Instant};
use tmpmail::providers::MailTm;

#[test]
#[ignore = "reference-machine performance gate"]
fn loopback_mailtm_client_overhead_p95_is_under_five_ms() {
    const REQUESTS: usize = 250;
    const WARMUPS: usize = 25;
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let address = listener.local_addr().expect("address");
    let server = thread::spawn(move || {
        let body = r#"{"id":"m1","from":{"name":"","address":"sender@example.test"},"subject":"subject","text":"body","html":[],"seen":false,"attachments":[],"createdAt":"2026-08-14T00:00:00Z"}"#;
        for _ in 0..REQUESTS + WARMUPS {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut request = [0_u8; 4096];
            let _ = stream.read(&mut request).expect("read");
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/ld+json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            )
            .expect("respond");
        }
    });
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    let provider = MailTm::with_base_url(format!("http://{address}"), Duration::from_secs(2))
        .expect("provider");
    runtime.block_on(async {
        for _ in 0..WARMUPS {
            provider.message("token", "m1").await.expect("warmup");
        }
        let mut samples = Vec::with_capacity(REQUESTS);
        for _ in 0..REQUESTS {
            let started = Instant::now();
            provider.message("token", "m1").await.expect("message");
            samples.push(started.elapsed());
        }
        samples.sort_unstable();
        let p95 = samples[(REQUESTS * 95 / 100) - 1];
        eprintln!("mailtm_loopback_p95_us={}", p95.as_micros());
        assert!(p95 <= Duration::from_millis(5), "p95 was {p95:?}");
    });
    server.join().expect("server");
}

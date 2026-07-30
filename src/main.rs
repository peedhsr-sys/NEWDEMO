use base64::{engine::general_purpose, Engine as _};
use reqwest::Client;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use urlencoding::encode;

// ============ SETTINGS ============
const BASE_URL: &str = "https://nfsa.up.gov.in/Food/TrackingRationCard/ViewRC.aspx?Details=";
const TARGET_STRING: &str = "119040407269";
const PREFIX: &str = "0919000960801176";
const START: u32 = 1;
const END: u32 = 300;
const CONCURRENCY_LIMIT: usize = 15; // एक साथ 15 लिंक्स चेक करेगा (ताकि ब्लॉक न हो)
// ==================================

#[tokio::main]
async fn main() {
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║         🚀 RUST ASYNC FINDER - SUPER FAST              ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Client बनाने में दिक्कत हुई");

    // यह ट्रैक करेगा कि हमें हमारा टारगेट मिला या नहीं
    let found = Arc::new(AtomicBool::new(false));
    
    // यह एक साथ जाने वाली रिक्वेस्ट को कंट्रोल करेगा (ताकि सर्वर क्रैश न हो)
    let semaphore = Arc::new(Semaphore::new(CONCURRENCY_LIMIT));
    let mut handles = vec![];

    for n in START..=END {
        // अगर मैच मिल चुका है, तो नए टास्क बनाना बंद कर दें
        if found.load(Ordering::Relaxed) {
            break;
        }

        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let client = client.clone();
        let found = Arc::clone(&found);

        let handle = tokio::spawn(async move {
            // अगर किसी और थ्रेड को मैच मिल गया है, तो इसे तुरंत रोक दें
            if found.load(Ordering::Relaxed) {
                drop(permit);
                return;
            }

            // URL बनाना
            let raw_string = format!("{}#{:08}#{}", PREFIX, n, TARGET_STRING);
            let b64_encoded = general_purpose::STANDARD.encode(&raw_string);
            let url_safe = encode(&b64_encoded);
            let final_url = format!("{}{}", BASE_URL, url_safe);

            // रिक्वेस्ट भेजना
            match client.get(&final_url)
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .send()
                .await 
            {
                Ok(response) => {
                    if let Ok(text) = response.text().await {
                        if text.contains(TARGET_STRING) {
                            println!("\n╔════════════════════════════════════════════════════════╗");
                            println!("║  🎉 MATCH FOUND! Page Number: {:<24} ║", n);
                            println!("║  🔗 EXACT LINK: {} ║", final_url);
                            println!("╚════════════════════════════════════════════════════════╝\n");
                            
                            // बाकियों को रोकने का सिग्नल दें
                            found.store(true, Ordering::Relaxed);
                        } else {
                            println!("❌ Mismatch | Serial: {:<5} | Card: {}", n, raw_string);
                        }
                    }
                }
                Err(_) => {
                    println!("⚠️ Error on serial {}: Timeout / Blocked", n);
                }
            }
            drop(permit); // नया काम शुरू करने के लिए जगह खाली करें
        });
        handles.push(handle);
    }

    // सभी थ्रेड्स के पूरा होने का इंतज़ार करें
    for handle in handles {
        let _ = handle.await;
    }

    if !found.load(Ordering::Relaxed) {
        println!("\n❌ पूरी रेंज में कोई मैच नहीं मिला।");
    }
}
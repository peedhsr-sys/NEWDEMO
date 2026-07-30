use base64::{engine::general_purpose, Engine as _};
use reqwest::Client;
use std::time::Duration;
use urlencoding::encode;

// ============ SETTINGS ============
const BASE_URL: &str = "https://nfsa.up.gov.in/Food/TrackingRationCard/ViewRC.aspx?Details=";
const TARGET_STRING: &str = "119040407269";
const PREFIX: &str = "0919000960801176";
const START: u32 = 1;
const END: u32 = 300;
// ==================================

#[tokio::main]
async fn main() {
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║      🚶 STEALTH MODE (HUMAN SPEED) FINDER              ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    let client = Client::builder()
        .timeout(Duration::from_secs(15)) // टाइमआउट बढ़ा दिया
        .build()
        .expect("Client बनाने में दिक्कत हुई");

    let mut found = false;

    for n in START..=END {
        let raw_string = format!("{}#{:08}#{}", PREFIX, n, TARGET_STRING);
        let b64_encoded = general_purpose::STANDARD.encode(&raw_string);
        let url_safe = encode(&b64_encoded);
        let final_url = format!("{}{}", BASE_URL, url_safe);

        // रिक्वेस्ट भेजना
        match client.get(&final_url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8")
            .header("Accept-Language", "hi,en-US;q=0.9,en;q=0.8") // सर्वर को लगे कि कोई भारतीय यूज़र है
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
                        found = true;
                        break;
                    } else {
                        println!("❌ Mismatch | Serial: {:<5} | Card: {}", n, raw_string);
                    }
                }
            }
            Err(_) => {
                println!("⚠️ Error on serial {}: कनेक्शन फेल! सरकारी सर्वर ने जवाब नहीं दिया।", n);
            }
        }
        
        // ✨ जादुई हिस्सा: हर बार चेक करने के बाद 3 सेकंड रुको ✨
        // इससे सर्वर को लगेगा कि कोई इंसान धीरे-धीरे स्टेटस चेक कर रहा है
        tokio::time::sleep(Duration::from_secs(3)).await;
    }

    if !found {
        println!("\n❌ पूरी रेंज में कोई मैच नहीं मिला।");
    }
}

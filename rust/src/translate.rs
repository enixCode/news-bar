// Traduction des titres en francais via l'API Groq (compatible OpenAI).
// Reutilise ureq + serde_json (aucune dependance en plus). La cle vient de la
// variable d'environnement GROQ_API_KEY (jamais du repo). Tout echec (reseau,
// 429/quota, cle invalide, parsing) renvoie None : l'appelant garde alors les
// titres en version originale (anglais). C'est volontaire, pas grave.

use std::time::Duration;

use ureq::config::Config as UreqConfig;
use ureq::tls::{RootCerts, TlsConfig, TlsProvider};

const ENDPOINT: &str = "https://api.groq.com/openai/v1/chat/completions";

// Traduit `titles` en francais. Retourne les traductions dans le meme ordre et
// la meme longueur, ou None si quoi que ce soit echoue.
pub fn translate_titles(
    titles: &[String],
    key: &str,
    model: &str,
    language: &str,
) -> Option<Vec<String>> {
    if titles.is_empty() {
        return Some(Vec::new());
    }

    let agent = UreqConfig::builder()
        // 429 (quota) / 4xx / 5xx ne sont pas des erreurs dures : on lit le statut
        // nous-memes et on retombe sur l'anglais.
        .http_status_as_error(false)
        .timeout_global(Some(Duration::from_secs(30)))
        .tls_config(
            TlsConfig::builder()
                .provider(TlsProvider::NativeTls)
                // Verifie les certificats via le magasin de Windows (SChannel),
                // pas un jeu de racines embarque -> connexions HTTPS fiables.
                .root_certs(RootCerts::PlatformVerifier)
                .build(),
        )
        .build()
        .new_agent();

    let input = serde_json::json!({ "titles": titles }).to_string();
    let system = format!(
        "Tu traduis des titres d'actualite vers cette langue : {language}. Style naturel et concis. Reponds UNIQUEMENT un objet JSON de la forme {{\"t\": [...]}}, ou t est le tableau des traductions, dans le meme ordre et la meme longueur que l'entree. Ne traduis pas les noms propres."
    );
    let payload = serde_json::json!({
        "model": model,
        "temperature": 0.2,
        "response_format": { "type": "json_object" },
        "messages": [
            { "role": "system", "content": system },
            { "role": "user", "content": input }
        ]
    })
    .to_string();

    let mut resp = agent
        .post(ENDPOINT)
        .header("Authorization", &format!("Bearer {}", key))
        .header("Content-Type", "application/json")
        .send(payload.as_str())
        .ok()?;

    // Quota depasse (429), cle invalide, service indisponible... -> on garde l'anglais.
    if resp.status().as_u16() != 200 {
        return None;
    }
    let text = resp.body_mut().read_to_string().ok()?;

    // Reponse OpenAI : choices[0].message.content contient la string JSON {"t":[...]}.
    let envelope: serde_json::Value = serde_json::from_str(&text).ok()?;
    let content = envelope["choices"][0]["message"]["content"].as_str()?;
    let parsed: serde_json::Value = serde_json::from_str(content).ok()?;
    let arr = parsed["t"].as_array()?;
    if arr.len() != titles.len() {
        return None;
    }
    let out: Vec<String> = arr
        .iter()
        .map(|x| x.as_str().unwrap_or("").to_string())
        .collect();
    if out.iter().any(|s| s.is_empty()) {
        return None;
    }
    Some(out)
}

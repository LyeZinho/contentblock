use std::net::{SocketAddr, Ipv4Addr};
use trust_dns_client::client::{Client, SyncClient};
use trust_dns_client::udp::UdpClientConnection;
use trust_dns_client::rr::{DNSClass, Name, RecordType};
use std::str::FromStr;

fn main() {
    println!("🧪 Testando servidor DNS local...\n");

    // Conecta ao servidor DNS local na porta 53
    let address = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 53);
    let conn = UdpClientConnection::new(address).unwrap();
    let client = SyncClient::new(conn);

    // Testa alguns domínios que devem estar bloqueados
    let blocked_domains = vec![
        "pornhub.com",
        "1337x.to", 
        "00webcams.com",
        "www.pornhub.com"
    ];

    // Testa alguns domínios que NÃO devem estar bloqueados
    let allowed_domains = vec![
        "google.com",
        "github.com",
        "stackoverflow.com",
        "rust-lang.org"
    ];

    println!("📋 Testando domínios que DEVEM estar bloqueados:");
    for domain in blocked_domains {
        test_domain(&client, domain, true);
    }

    println!("\n📋 Testando domínios que NÃO devem estar bloqueados:");
    for domain in allowed_domains {
        test_domain(&client, domain, false);
    }
}

fn test_domain(client: &SyncClient<UdpClientConnection>, domain: &str, should_be_blocked: bool) {
    let name = Name::from_str(domain).unwrap();
    
    match client.query(&name, DNSClass::IN, RecordType::A) {
        Ok(response) => {
            let answers = response.answers();
            if let Some(record) = answers.first() {
                if let Some(rdata) = record.data() {
                    match rdata {
                        trust_dns_client::rr::RData::A(ip) => {
                            let is_blocked = *ip == Ipv4Addr::new(0, 0, 0, 0);
                            let status = if should_be_blocked {
                                if is_blocked { "✅ BLOQUEADO" } else { "❌ NÃO BLOQUEADO" }
                            } else {
                                if is_blocked { "❌ INCORRETAMENTE BLOQUEADO" } else { "✅ PERMITIDO" }
                            };
                            
                            println!("  {} {} -> {}", status, domain, ip);
                        }
                        _ => {
                            println!("  ❓ {} -> Tipo de resposta inesperado", domain);
                        }
                    }
                }
            } else {
                println!("  ❌ {} -> Sem resposta", domain);
            }
        }
        Err(e) => {
            println!("  ❌ {} -> Erro: {}", domain, e);
        }
    }
}
